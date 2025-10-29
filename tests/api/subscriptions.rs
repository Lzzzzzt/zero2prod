use crate::{TestApp, percent_encode};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = TestApp::new().await;

    let name = "lzzzt";
    let email = "main@lzzzt.cc";
    let body = format!(
        "name={}&email={}",
        percent_encode(name),
        percent_encode(email)
    );

    let response = app.post_subscriptions(body).await;

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.conn_pool)
        .await
        .expect("Failed to read from Postgres");

    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let app = TestApp::new().await;

    let test_cases = vec![
        ("name=lzzzt", "missing the email"),
        ("email=main%40lzzzt.cc", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.to_string()).await;

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_422_when_fields_are_present_but_empty() {
    let app = TestApp::new().await;

    let test_cases = vec![
        ("name=&email=main%40lzzzt.cc", "empty name"),
        ("name=lzzzt&email=", "empty email"),
        ("name=lzzzt&email=12345", "invalid email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.to_string()).await;

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        )
    }
}
