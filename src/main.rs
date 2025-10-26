use secrecy::ExposeSecret;
use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::{
    config::get_config,
    telemetry::{create_subscriber, setup_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = create_subscriber("zero2prod", "info", std::io::stdout);
    setup_subscriber(subscriber);

    let config = get_config().await.expect("Failed to read config.");
    let conn_pool = PgPool::connect(config.db_config.connection_url().expose_secret())
        .await
        .expect("Failed to connect to the postgres");
    let listener = TcpListener::bind(("127.0.0.1", config.port)).await?;

    zero2prod::run(listener, conn_pool).await
}
