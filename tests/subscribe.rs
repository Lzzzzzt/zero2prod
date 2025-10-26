mod utils;

use percent_encoding::NON_ALPHANUMERIC;
use percent_encoding::utf8_percent_encode;

use crate::utils::create_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = create_app().await;
    let client = reqwest::Client::new();

    let address = app.address;
    let name = "lzzzt";
    let email = "main@lzzzt.cc";
    let body = format!(
        "name={}&email={}",
        utf8_percent_encode(name, NON_ALPHANUMERIC),
        utf8_percent_encode(email, NON_ALPHANUMERIC)
    );

    let response = client
        .post(format!("{address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.conn_pool)
        .await
        .expect("Failed to read from Postgres");

    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = create_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=lzzzt", "missing the email"),
        ("email=main%40lzzzt.cc", "missing the name"),
        ("", "missing both name and email"),
    ];
    let address = app.address;

    let url = format!("{address}/subscriptions");
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        )
    }
}
