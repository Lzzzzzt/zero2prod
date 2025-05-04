use std::{sync::LazyLock, vec};

use reqwest::Client;
use secrecy::ExposeSecret;
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    config::{DatabaseSettings, get_config},
    telemetry::init_subscriber,
};

pub struct TestApp {
    address: String,
    connection_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = Client::new();

    let body = "name=szyer&email=szyer%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions;")
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to read table.");

    assert_eq!(saved.email, "szyer@gmail.com");
    assert_eq!(saved.name, "szyer");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = Client::new();
    let test_cases = vec![
        ("name=szyer", "missing the email"),
        ("email=szyer%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload as {error_message}"
        )
    }
}

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber_name = "test";
    let filter_level = "debug";

    match std::env::var("TEST_LOG") {
        Ok(_) => init_subscriber(subscriber_name, filter_level, std::io::stdout),
        Err(_) => init_subscriber(subscriber_name, filter_level, std::io::sink),
    }
});

async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let listener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind address with random port.");
    let port = listener.local_addr().unwrap().port();

    let mut config = get_config().expect("Failed to parse configuration.");
    config.database_settings.name = Uuid::new_v4().to_string();
    let connection = setting_up_database(&config.database_settings).await;

    let server =
        zero2prod::startup::run(listener, connection.clone()).expect("Failed to start server");

    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        connection_pool: connection,
    }
}

pub async fn setting_up_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to database.");

    sqlx::raw_sql(&format!(r#"CREATE DATABASE "{}";"#, config.name))
        .execute(&mut connection)
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to database.");

    sqlx::migrate!()
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
