mod auth;
mod videos;
mod lives;
mod channels;
mod events;
mod cookies;
mod transcriptions;
mod admin;
mod search;

use axum::{Router, middleware};
use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/v1/auth", auth::router())
        .nest("/v1/videos", videos::router())
        .nest("/v1/lives", lives::router())
        .nest("/v1/channels", channels::router())
        .nest("/v1/events", events::router())
        .nest("/v1/cookies", cookies::router())
        .nest("/v1/transcriptions", transcriptions::router())
        .nest("/v1/admin", admin::router())
        .nest("/v1/search", search::router())
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth_middleware))
        .with_state(state)
}