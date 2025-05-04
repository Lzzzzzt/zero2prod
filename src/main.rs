use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{config::get_config, startup::run, telemetry::init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_subscriber("zero2prod", "info", std::io::stdout);

    let config = get_config().expect("Failed to parse the configuration.");

    let listener = TcpListener::bind(("0.0.0.0", config.port))?;
    let connection = PgPool::connect(config.database_settings.connection_string().expose_secret())
        .await
        .expect("Failed to connect database.");
    run(listener, connection)?.await
}
