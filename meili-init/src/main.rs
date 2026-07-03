use shared::config::AppConfig;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "meili_init=info".into()),
        )
        .init();

    let config = AppConfig::from_env();

    let db = PgPool::connect(&config.database_url)
        .await
        .expect("failed to connect to postgres");

    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(key) = &config.meilisearch_api_key {
        headers.insert(
            "Authorization",
            format!("Bearer {}", key).parse().unwrap(),
        );
    }

    let videos = sqlx::query_as::<_, (String, String, i32, Option<String>, String, String)>(
        "SELECT id::text, title, duration, upload_date::text, uploader, platform FROM videos",
    )
    .fetch_all(&db)
    .await
    .expect("failed to fetch videos");

    let video_docs: Vec<serde_json::Value> = videos
        .into_iter()
        .map(|(id, title, duration, upload_date, uploader, platform)| {
            serde_json::json!({
                "id": id,
                "title": title,
                "duration": duration,
                "upload_date": upload_date,
                "uploader": uploader,
                "platform": platform,
            })
        })
        .collect();

    if !video_docs.is_empty() {
        let index_url = format!("{}/indexes/videos/documents", config.meilisearch_url);
        let resp = client
            .post(&index_url)
            .headers(headers.clone())
            .json(&video_docs)
            .send()
            .await
            .expect("failed to index videos");
        tracing::info!("indexed {} videos (status: {})", video_docs.len(), resp.status());
    }

    let lives = sqlx::query_as::<_, (String, String, Option<i32>, Option<String>, String, String)>(
        "SELECT id::text, title, duration, live_date::text, uploader, platform FROM lives WHERE is_active = true",
    )
    .fetch_all(&db)
    .await
    .expect("failed to fetch lives");

    let live_docs: Vec<serde_json::Value> = lives
        .into_iter()
        .map(|(id, title, duration, live_date, uploader, platform)| {
            serde_json::json!({
                "id": id,
                "title": title,
                "duration": duration,
                "live_date": live_date,
                "uploader": uploader,
                "platform": platform,
            })
        })
        .collect();

    if !live_docs.is_empty() {
        let index_url = format!("{}/indexes/lives/documents", config.meilisearch_url);
        let resp = client
            .post(&index_url)
            .headers(headers.clone())
            .json(&live_docs)
            .send()
            .await
            .expect("failed to index lives");
        tracing::info!("indexed {} lives (status: {})", live_docs.len(), resp.status());
    }

    tracing::info!("reindex complete");
}