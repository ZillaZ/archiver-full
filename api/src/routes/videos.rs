use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use shared::models::Video;
use uuid::Uuid;
use crate::AppState;

#[derive(Deserialize)]
pub struct ListParams {
    pub platform: Option<String>,
    pub uploader: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Json<Vec<Video>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let videos = sqlx::query_as::<_, Video>(
        "SELECT * FROM videos WHERE
            ($1::text IS NULL OR platform = $1)
            AND ($2::text IS NULL OR uploader = $2)
         ORDER BY upload_date DESC NULLS LAST
         LIMIT $3 OFFSET $4",
    )
    .bind(&params.platform)
    .bind(&params.uploader)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(videos)
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Video>, StatusCode> {
    sqlx::query_as::<_, Video>("SELECT * FROM videos WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn heatmap(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let video = sqlx::query_as::<_, Video>("SELECT * FROM videos WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match video.heatmap_path {
        Some(path) => {
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|_| StatusCode::NOT_FOUND)?;
            let data: serde_json::Value = serde_json::from_str(&content)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(data))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list))
        .route("/{id}", axum::routing::get(get))
        .route("/{id}/heatmap", axum::routing::get(heatmap))
}