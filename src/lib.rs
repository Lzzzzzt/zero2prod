use axum::{Router, http::StatusCode, routing::get};
use tokio::net::TcpListener;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let router = Router::new().route("/health_check", get(health_check));

    axum::serve(listener, router).await
}
