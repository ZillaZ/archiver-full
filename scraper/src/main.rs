mod fetcher;
mod proxy;
mod cookies;
mod storage;

use shared::config::AppConfig;
use shared::messages::{Message, ScrapeChannel, RemoveChannel};
use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

struct AppState {
    config: AppConfig,
    db: sqlx::PgPool,
    proxy_list: Arc<Mutex<proxy::ProxyManager>>,
    cookies_store: Arc<Mutex<cookies::CookieStore>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scraper=info".into()),
        )
        .init();

    let config = AppConfig::from_env();

    let db = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("failed to connect to postgres");

    let proxy_list = std::env::var("PROXY_LIST")
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let proxy_mgr = Arc::new(Mutex::new(proxy::ProxyManager::new(proxy_list)));

    let initial_cookies = cookies::load_cookies_from_db(&db).await;
    let cookie_store = Arc::new(Mutex::new(cookies::CookieStore::new(initial_cookies)));

    let state = Arc::new(AppState {
        config: config.clone(),
        db,
        proxy_list: proxy_mgr,
        cookies_store: cookie_store,
    });

    let conn = Connection::connect(
        &config.rabbitmq_url,
        ConnectionProperties::default(),
    )
    .await
    .expect("failed to connect to rabbitmq");

    let channel = conn.create_channel().await.unwrap();

    channel
        .exchange_declare(
            "bigshit",
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .unwrap();

    let queue = channel
        .queue_declare(
            "scraper",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .unwrap();

    channel
        .queue_bind(
            queue.name().as_str(),
            "bigshit",
            "scrape.channel",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    channel
        .queue_bind(
            queue.name().as_str(),
            "bigshit",
            "cookies.updated",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    channel
        .queue_bind(
            queue.name().as_str(),
            "bigshit",
            "channel.removed",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "scraper",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    tracing::info!("scraper listening for messages");

    while let Some(delivery) = consumer.next().await {
        let delivery = match delivery {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("consumer error: {e}");
                continue;
            }
        };

        let state = state.clone();
        tokio::spawn(async move {
            let msg: Message = match serde_json::from_slice(&delivery.data) {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!("failed to deserialize message: {e}");
                    delivery.ack(BasicAckOptions::default()).await.ok();
                    return;
                }
            };

            match msg {
                Message::ScrapeChannel(cmd) => {
                    handle_scrape_channel(state, cmd).await;
                }
                Message::RemoveChannel(cmd) => {
                    handle_remove_channel(state, cmd).await;
                }
                Message::CookieUpdate(update) => {
                    let platform = update.platform.clone();
                    let mut store = state.cookies_store.lock().await;
                    store.set(update.platform, update.content);
                    tracing::info!("cookies updated for platform: {platform}");
                }
                _ => {}
            }

            delivery.ack(BasicAckOptions::default()).await.ok();
        });
    }
}

async fn handle_scrape_channel(state: Arc<AppState>, cmd: ScrapeChannel) {
    tracing::info!(
        "scraping channel: {} ({})",
        cmd.name,
        cmd.platform
    );

    let proxy = {
        let mut mgr = state.proxy_list.lock().await;
        mgr.next()
    };

    let cookies = {
        let store = state.cookies_store.lock().await;
        store.get(&cmd.platform).cloned()
    };

    let result = fetcher::fetch_metadata(
        &cmd.platform,
        &cmd.platform_id,
        proxy.as_deref(),
        cookies.as_deref(),
    )
    .await;

    let result = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("failed to fetch metadata for {}: {e}", cmd.name);
            return;
        }
    };

    for video in &result.videos {
        let insert_result = sqlx::query_as::<_, (uuid::Uuid,)>(
            "INSERT INTO videos (platform, platform_id, title, uploader, uploader_id, duration, upload_date, thumbnail_url, description)
             VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, $8, $9)
             ON CONFLICT (platform, platform_id) DO UPDATE SET
                title = EXCLUDED.title,
                duration = EXCLUDED.duration,
                upload_date = EXCLUDED.upload_date,
                thumbnail_url = EXCLUDED.thumbnail_url,
                description = EXCLUDED.description,
                updated_at = NOW()
             RETURNING id",
        )
        .bind(&cmd.platform)
        .bind(&video.platform_id)
        .bind(&video.title)
        .bind(&video.uploader)
        .bind(&video.uploader_id)
        .bind(video.duration)
        .bind(&video.upload_date)
        .bind(&video.thumbnail_url)
        .bind(&video.description)
        .fetch_optional(&state.db)
        .await;

        if let Ok(Some((video_id,))) = insert_result {
            let notification = shared::messages::NewVideoNotification {
                video_id,
                platform: cmd.platform.clone(),
                platform_id: video.platform_id.clone(),
                title: video.title.clone(),
                uploader: video.uploader.clone(),
                duration: video.duration,
            };
            let payload = serde_json::to_vec(&notification).unwrap();
            publish_amqp(&state.config.rabbitmq_url, "video.new", &payload).await;

            if cmd.platform == "youtube" {
                let state_clone = state.clone();
                let pid = video.platform_id.clone();
                let proxy = proxy.clone();
                let cookies = cookies.clone();
                tokio::spawn(async move {
                    if let Ok(Some(heatmap)) = fetcher::fetch_video_heatmap(
                        "youtube", &pid, proxy.as_deref(), cookies.as_deref(),
                    ).await {
                        let path = format!("/tmp/heatmap_{pid}.json");
                        if let Ok(content) = serde_json::to_string(&heatmap) {
                            if tokio::fs::write(&path, &content).await.is_ok() {
                                let _ = sqlx::query(
                                    "UPDATE videos SET heatmap_path = $1, updated_at = NOW() WHERE id = $2"
                                )
                                .bind(&path)
                                .bind(video_id)
                                .execute(&state_clone.db)
                                .await;
                            }
                        }
                    }
                });
            }
        }
    }

    for live in &result.active_lives {
        let insert_result = sqlx::query_as::<_, (uuid::Uuid,)>(
            "INSERT INTO lives (platform, platform_id, title, uploader, uploader_id, is_active, thumbnail_url, description)
             VALUES ($1, $2, $3, $4, $5, true, $6, $7)
             ON CONFLICT (platform, platform_id) DO UPDATE SET
                title = EXCLUDED.title,
                is_active = true,
                thumbnail_url = EXCLUDED.thumbnail_url,
                description = EXCLUDED.description,
                updated_at = NOW()
             RETURNING id",
        )
        .bind(&cmd.platform)
        .bind(&live.platform_id)
        .bind(&live.title)
        .bind(&live.uploader)
        .bind(&live.uploader_id)
        .bind(&live.thumbnail_url)
        .bind(&live.description)
        .fetch_optional(&state.db)
        .await;

        if let Ok(Some((live_id,))) = insert_result {
            let notification = shared::messages::NewLiveNotification {
                live_id,
                platform: cmd.platform.clone(),
                platform_id: live.platform_id.clone(),
                title: live.title.clone(),
                uploader: live.uploader.clone(),
            };
            let payload = serde_json::to_vec(&notification).unwrap();
            publish_amqp(&state.config.rabbitmq_url, "live.new", &payload).await;
        }
    }

    tracing::info!(
        "scraped {}: {} videos, {} lives",
        cmd.name,
        result.videos.len(),
        result.active_lives.len()
    );
}

async fn publish_amqp(url: &str, routing_key: &str, payload: &[u8]) {
    if let Ok(conn) = Connection::connect(url, ConnectionProperties::default()).await {
        if let Ok(channel) = conn.create_channel().await {
            let _ = channel
                .basic_publish(
                    "bigshit",
                    routing_key,
                    BasicPublishOptions::default(),
                    payload,
                    BasicProperties::default(),
                )
                .await;
        }
    }
}

async fn handle_remove_channel(state: Arc<AppState>, cmd: RemoveChannel) {
    tracing::info!(
        "removing channel: {} ({})",
        cmd.platform,
        cmd.platform_id
    );

    sqlx::query("DELETE FROM channels WHERE id = $1")
        .bind(cmd.channel_id)
        .execute(&state.db)
        .await
        .ok();
}