mod health_check;
mod subscriptions;

use std::sync::LazyLock;

use percent_encoding::{NON_ALPHANUMERIC, PercentEncode, utf8_percent_encode};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::{
    App,
    config::{DBConfig, get_config},
    telemetry::{create_subscriber, setup_subscriber},
};

static TRACING_SUBSCRIBER: LazyLock<()> = LazyLock::new(|| {
    match std::env::var("TEST_LOG") {
        Ok(_) => setup_subscriber(create_subscriber("zero2prod-test", "info", std::io::stdout)),
        Err(_) => setup_subscriber(create_subscriber("zero2prod-test", "info", std::io::sink)),
    };
});

pub struct TestApp {
    pub address: String,
    pub conn_pool: PgPool,
}

impl TestApp {
    pub async fn new() -> Self {
        LazyLock::force(&TRACING_SUBSCRIBER);

        let config = {
            let mut c = get_config().await.expect("Failed to read config.");
            c.db_config.db_name = uuid::Uuid::new_v4().to_string();
            c.app_config.port = 0;
            c
        };
        let conn_pool = setup_database(&config.db_config).await;

        let app = App::build(config).await.expect("Failed to build app.");

        let test_app = TestApp {
            address: format!("http://127.0.0.1:{}", app.port()),
            conn_pool,
        };

        // Run the server at background
        tokio::spawn(async { app.run().await });

        test_app
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to send request.")
    }
}

async fn setup_database(config: &DBConfig) -> PgPool {
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

pub fn percent_encode<'a>(input: &'a str) -> PercentEncode<'a> {
    utf8_percent_encode(input, NON_ALPHANUMERIC)
}
