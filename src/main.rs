use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{config::get_config, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_config().expect("Failed to parse the configuration.");

    let listener = TcpListener::bind(("0.0.0.0", config.port))?;
    let connection = PgPool::connect(config.database_settings.get_connection_string().as_str())
        .await
        .expect("Failed to connect database.");
    run(listener, connection)?.await
}
