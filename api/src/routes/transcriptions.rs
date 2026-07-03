use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use shared::models::Video;
use uuid::Uuid;
use crate::AppState;

#[derive(Deserialize)]
pub struct UploadTranscriptionRequest {
    pub transcription: String,
}

pub async fn upload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UploadTranscriptionRequest>,
) -> Result<Json<Video>, axum::http::StatusCode> {
    sqlx::query_as::<_, Video>(
        "UPDATE videos SET transcription_path = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(&req.transcription)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    .map(Json)
    .ok_or(axum::http::StatusCode::NOT_FOUND)
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new().route("/{id}", axum::routing::post(upload))
}