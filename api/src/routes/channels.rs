use axum::{
    extract::{Path, State, Extension},
    Json,
};
use serde::Deserialize;
use shared::models::{Channel, WatchedChannel};
use shared::messages::{Message, ScrapeChannel, RemoveChannel};
use crate::AppState;
use crate::routes::auth::Claims;

#[derive(Deserialize)]
pub struct AddChannelRequest {
    pub platform: String,
    pub platform_id: String,
    pub name: String,
}

pub async fn add(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AddChannelRequest>,
) -> Result<Json<Channel>, axum::http::StatusCode> {
    let client_id = claims.client_id()?;

    sqlx::query(
        "INSERT INTO watched_channels (client_id, platform, platform_id, name)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (client_id, platform, platform_id) DO NOTHING",
    )
    .bind(client_id)
    .bind(&req.platform)
    .bind(&req.platform_id)
    .bind(&req.name)
    .execute(&state.db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let channel = sqlx::query_as::<_, Channel>(
        "INSERT INTO channels (platform, platform_id, name, watcher_count)
         VALUES ($1, $2, $3, 1)
         ON CONFLICT (platform, platform_id) DO UPDATE SET
            watcher_count = channels.watcher_count + 1,
            name = EXCLUDED.name,
            updated_at = NOW()
         RETURNING *",
    )
    .bind(&req.platform)
    .bind(&req.platform_id)
    .bind(&req.name)
    .fetch_one(&state.db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let msg = Message::ScrapeChannel(ScrapeChannel {
        channel_id: channel.id,
        platform: channel.platform.clone(),
        platform_id: channel.platform_id.clone(),
        name: channel.name.clone(),
    });

    let payload = serde_json::to_vec(&msg).unwrap();
    crate::amqp::publish(&state.amqp, "scrape.channel", &payload)
        .await
        .ok();

    Ok(Json(channel))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((platform, platform_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let client_id = claims.client_id()?;

    let deleted = sqlx::query(
        "DELETE FROM watched_channels WHERE client_id = $1 AND platform = $2 AND platform_id = $3",
    )
    .bind(client_id)
    .bind(&platform)
    .bind(&platform_id)
    .execute(&state.db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    .rows_affected();

    if deleted == 0 {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    let remaining = sqlx::query_as::<_, Channel>(
        "UPDATE channels SET watcher_count = watcher_count - 1, updated_at = NOW()
         WHERE platform = $1 AND platform_id = $2
         RETURNING *",
    )
    .bind(&platform)
    .bind(&platform_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(channel) = remaining {
        if channel.watcher_count <= 0 {
            let msg = Message::RemoveChannel(RemoveChannel {
                channel_id: channel.id,
                platform: platform.clone(),
                platform_id: platform_id.clone(),
            });

            let payload = serde_json::to_vec(&msg).unwrap();
            crate::amqp::publish(&state.amqp, "channel.removed", &payload)
                .await
                .ok();

            sqlx::query("DELETE FROM channels WHERE id = $1")
                .bind(channel.id)
                .execute(&state.db)
                .await
                .ok();
        }
    }

    Ok(Json(serde_json::json!({"status": "removed"})))
}

pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Json<Vec<WatchedChannel>> {
    let client_id = match claims.client_id() {
        Ok(id) => id,
        Err(_) => return Json(Vec::new()),
    };

    let channels = sqlx::query_as::<_, WatchedChannel>(
        "SELECT * FROM watched_channels WHERE client_id = $1 ORDER BY name",
    )
    .bind(client_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(channels)
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list).post(add))
        .route("/{platform}/{platform_id}", axum::routing::delete(delete))
}