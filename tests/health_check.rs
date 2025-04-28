use reqwest::Client;
use tokio::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let server_address = spawn_app().await;

    let client = Client::new();

    let response = client
        .get(format!("{server_address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind address with random port.");
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(zero2prod::run(listener));

    format!("http://127.0.0.1:{port}")
}
