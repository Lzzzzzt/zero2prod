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
    let conn_pool = PgPool::connect_lazy_with(config.db_config.connection_options());
    let listener = TcpListener::bind((config.app_config.host, config.app_config.port)).await?;

    zero2prod::run(listener, conn_pool).await
}
