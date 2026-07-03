use axum::{extract::State, Json};
use serde::Deserialize;
use shared::models::Cookie;
use shared::messages::{Message, CookieUpdate};
use crate::AppState;

#[derive(Deserialize)]
pub struct UploadCookiesRequest {
    pub platform: String,
    pub content: String,
}

pub async fn upload(
    State(state): State<AppState>,
    Json(req): Json<UploadCookiesRequest>,
) -> Result<Json<Cookie>, axum::http::StatusCode> {
    let cookie = sqlx::query_as::<_, Cookie>(
        "INSERT INTO cookies (platform, content)
         VALUES ($1, $2)
         ON CONFLICT (platform) DO UPDATE SET content = $2, updated_at = NOW()
         RETURNING *",
    )
    .bind(&req.platform)
    .bind(&req.content)
    .fetch_one(&state.db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let msg = Message::CookieUpdate(CookieUpdate {
        platform: req.platform,
        content: req.content,
    });

    let payload = serde_json::to_vec(&msg).unwrap();
    crate::amqp::publish(&state.amqp, "cookies.updated", &payload)
        .await
        .ok();

    Ok(Json(cookie))
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new().route("/", axum::routing::post(upload))
}