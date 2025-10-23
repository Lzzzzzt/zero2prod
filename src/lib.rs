use axum::{Router, http::StatusCode, routing::get};
use tokio::net::TcpListener;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn run(listener: std::net::TcpListener) -> Result<(), std::io::Error> {
    let router = Router::new().route("/health_check", get(health_check));

    listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(listener)?;

    axum::serve(listener, router).await
}
