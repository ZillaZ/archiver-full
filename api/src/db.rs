use sqlx::postgres::PgPoolOptions;

pub async fn connect(database_url: &str) -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .expect("failed to connect to postgres")
}

pub async fn run_migrations(pool: &sqlx::PgPool) {
    sqlx::query(include_str!("../../migrations/001_init.sql"))
        .execute(pool)
        .await
        .expect("failed to run migrations");
}