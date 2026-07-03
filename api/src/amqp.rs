use deadpool_lapin::Config;
use lapin::BasicProperties;

pub async fn connect(url: &str) -> deadpool_lapin::Pool {
    let cfg = Config {
        url: Some(url.into()),
        ..Default::default()
    };
    cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1))
        .expect("failed to create rabbitmq pool")
}

pub async fn publish(
    pool: &deadpool_lapin::Pool,
    routing_key: &str,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.get().await?;
    let channel = conn.create_channel().await?;

    channel
        .exchange_declare(
            "bigshit",
            lapin::ExchangeKind::Topic,
            lapin::options::ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await?;

    channel
        .basic_publish(
            "bigshit",
            routing_key,
            lapin::options::BasicPublishOptions::default(),
            payload,
            BasicProperties::default(),
        )
        .await?;

    Ok(())
}