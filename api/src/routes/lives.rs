use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use shared::models::Live;
use uuid::Uuid;
use crate::AppState;

#[derive(Deserialize)]
pub struct ListParams {
    pub platform: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Json<Vec<Live>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let lives = sqlx::query_as::<_, Live>(
        "SELECT * FROM lives WHERE
            ($1::text IS NULL OR platform = $1)
            AND ($2::bool IS NULL OR is_active = $2)
         ORDER BY live_date DESC NULLS LAST
         LIMIT $3 OFFSET $4",
    )
    .bind(&params.platform)
    .bind(params.is_active)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(lives)
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Live>, axum::http::StatusCode> {
    sqlx::query_as::<_, Live>("SELECT * FROM lives WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(axum::http::StatusCode::NOT_FOUND)
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(list))
        .route("/{id}", axum::routing::get(get))
}