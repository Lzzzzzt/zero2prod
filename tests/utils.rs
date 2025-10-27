use std::sync::LazyLock;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::net::TcpListener;
use zero2prod::{
    config::{DBConfig, get_config},
    telemetry::{create_subscriber, setup_subscriber},
};

static SUBSCRIBER: LazyLock<()> = LazyLock::new(|| {
    match std::env::var("TEST_LOG") {
        Ok(_) => setup_subscriber(create_subscriber("zero2prod-test", "info", std::io::stdout)),
        Err(_) => setup_subscriber(create_subscriber("zero2prod-test", "info", std::io::sink)),
    };
});

pub struct TestApp {
    pub address: String,
    #[allow(unused)]
    pub conn_pool: PgPool,
}

pub async fn create_app() -> TestApp {
    LazyLock::force(&SUBSCRIBER);

    // Bind the address and listen
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("Failed to bind address.");
    // Generate the server address
    let address = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());

    let mut config = get_config().await.expect("Failed to read config.");
    config.db_config.db_name = uuid::Uuid::new_v4().to_string();
    let conn_pool = setup_database(&config.db_config).await;

    let app = TestApp {
        address,
        conn_pool: conn_pool.clone(),
    };
    // Run the server at background
    tokio::spawn(async { zero2prod::run(listener, conn_pool).await });

    app
}

pub async fn setup_database(config: &DBConfig) -> PgPool {
    let test_settings = DBConfig {
        db_name: "postgres".into(),
        username: "postgres".into(),
        password: "password".into(),
        ..config.clone()
    };

    let mut conn = PgConnection::connect_with(&test_settings.connection_options())
        .await
        .expect("Failed to connect to Postgres.");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create database.");

    let conn_pool = PgPool::connect_with(config.connection_options())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!()
        .run(&conn_pool)
        .await
        .expect("Failed to migrate database.");

    conn_pool
}
