#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub rabbitmq_url: String,
    pub meilisearch_url: String,
    pub meilisearch_api_key: Option<String>,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://bigshit:bigshit@localhost:5432/bigshit".into()),
            rabbitmq_url: std::env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://bigshit:bigshit@localhost:5672".into()),
            meilisearch_url: std::env::var("MEILISEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:7700".into()),
            meilisearch_api_key: std::env::var("MEILISEARCH_API_KEY").ok(),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".into()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
        }
    }
}