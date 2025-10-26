mod utils;

use crate::utils::create_app;

#[tokio::test]
async fn health_check_works() {
    // Create the app
    let address = create_app().await.address;
    // Init the test client
    let client = reqwest::Client::new();
    // Send the test request
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to send request.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}
