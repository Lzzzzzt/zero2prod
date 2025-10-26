use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::routes::*;
use crate::telemetry::with_request_id;

pub async fn run(listener: TcpListener, conn_pool: PgPool) -> Result<(), std::io::Error> {
    let mut router = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(conn_pool);

    router = with_request_id(router);

    axum::serve(listener, router).await
}
