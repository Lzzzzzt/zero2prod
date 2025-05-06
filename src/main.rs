use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{config::get_config, startup::run, telemetry::init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_subscriber("zero2prod", "info", std::io::stdout);

    let config = get_config().expect("Failed to parse the configuration.");

    let listener = TcpListener::bind((config.app_settings.host, config.app_settings.port))?;
    let connection = PgPool::connect_lazy_with(config.database_settings.with_database());

    run(listener, connection)?.await
}
