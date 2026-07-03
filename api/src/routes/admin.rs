use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use crate::AppState;

#[derive(Serialize)]
pub struct ReindexResponse {
    pub status: String,
    pub videos_indexed: u64,
    pub lives_indexed: u64,
}

pub async fn reindex(
    State(state): State<AppState>,
) -> Result<Json<ReindexResponse>, StatusCode> {
    let videos = sqlx::query_as::<_, (String, String, i32, Option<String>, String, String)>(
        "SELECT id::text, title, duration, upload_date::text, uploader, platform FROM videos",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let lives = sqlx::query_as::<_, (String, String, Option<i32>, Option<String>, String, String)>(
        "SELECT id::text, title, duration, live_date::text, uploader, platform FROM lives WHERE is_active = true",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = reqwest::Client::new();

    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(key) = &state.config.meilisearch_api_key {
        headers.insert(
            "Authorization",
            format!("Bearer {}", key).parse().unwrap(),
        );
    }

    let index_url = format!("{}/indexes/videos/documents", state.config.meilisearch_url);
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

    let videos_count = video_docs.len() as u64;
    if !video_docs.is_empty() {
        client
            .post(&index_url)
            .headers(headers.clone())
            .json(&video_docs)
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let lives_index_url = format!("{}/indexes/lives/documents", state.config.meilisearch_url);
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

    let lives_count = live_docs.len() as u64;
    if !live_docs.is_empty() {
        client
            .post(&lives_index_url)
            .headers(headers)
            .json(&live_docs)
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(ReindexResponse {
        status: "ok".into(),
        videos_indexed: videos_count,
        lives_indexed: lives_count,
    }))
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new().route("/reindex", axum::routing::post(reindex))
}