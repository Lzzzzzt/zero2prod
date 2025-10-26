use axum::http::StatusCode;
use tracing::instrument;

#[instrument]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
