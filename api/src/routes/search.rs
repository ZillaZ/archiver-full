use axum::{extract::{Query, State}, Json};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub hits: Vec<serde_json::Value>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

async fn meili_search(
    state: &AppState,
    index: &str,
    q: &str,
    limit: usize,
    offset: usize,
) -> Result<SearchResult, axum::http::StatusCode> {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(key) = &state.config.meilisearch_api_key {
        headers.insert("Authorization", format!("Bearer {}", key).parse().unwrap());
    }

    let search_url = format!(
        "{}/indexes/{}/search",
        state.config.meilisearch_url, index
    );

    let resp = client
        .post(&search_url)
        .headers(headers)
        .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
        .json(&serde_json::json!({ "q": q }))
        .send()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let hits = body["hits"].as_array().cloned().unwrap_or_default();
    let total = body["total"].as_u64().unwrap_or(0) as usize;

    Ok(SearchResult { hits, total, limit, offset })
}

pub async fn search_videos(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResult>, axum::http::StatusCode> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    meili_search(&state, "videos", &params.q, limit, offset).await.map(Json)
}

pub async fn search_lives(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResult>, axum::http::StatusCode> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    meili_search(&state, "lives", &params.q, limit, offset).await.map(Json)
}

pub async fn search_all(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResult>, axum::http::StatusCode> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let videos = meili_search(&state, "videos", &params.q, limit, offset).await;
    let lives = meili_search(&state, "lives", &params.q, limit, offset).await;

    let mut all_hits = Vec::new();
    if let Ok(v) = &videos {
        all_hits.extend(v.hits.clone());
    }
    if let Ok(l) = &lives {
        all_hits.extend(l.hits.clone());
    }

    let total = all_hits.len();
    Ok(Json(SearchResult { hits: all_hits, total, limit, offset }))
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/videos", axum::routing::get(search_videos))
        .route("/lives", axum::routing::get(search_lives))
        .route("/all", axum::routing::get(search_all))
}