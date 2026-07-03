mod db;
mod amqp;
mod routes;

use shared::config::AppConfig;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: sqlx::PgPool,
    pub amqp: deadpool_lapin::Pool,
    pub event_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::from_env();
    let db = db::connect(&config.database_url).await;
    let amqp = amqp::connect(&config.rabbitmq_url).await;
    let (event_tx, _) = broadcast::channel(1024);

    db::run_migrations(&db).await;

    let state = AppState { config: config.clone(), db, amqp, event_tx };

    let app = routes::router(state);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}