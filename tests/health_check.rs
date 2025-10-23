use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    // Create the app
    let address = create_app();
    // Init the test client
    let client = reqwest::Client::new();
    // Send the test request
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

fn create_app() -> String {
    // Bind the address and listen
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("Failed to bind address");
    // Generate the server address
    let address = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    // Run the server at background
    tokio::spawn(async { zero2prod::run(listener).await });

    address
}
