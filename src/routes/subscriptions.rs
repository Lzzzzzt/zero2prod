use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;

use crate::domain::Subscriber;

#[derive(Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[instrument(
    skip_all,
    name = "Adding a new subscriber"
    fields(subsriber_name = %data.name, subscriber_email = %data.email)
)]
pub async fn subscribe(State(pool): State<PgPool>, Form(data): Form<FormData>) -> StatusCode {
    let subscriber = match data.try_into() {
        Ok(data) => data,
        Err(_) => return StatusCode::UNPROCESSABLE_ENTITY,
    };

    match insert_subscriber(&pool, &subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[instrument(skip_all, name = "Saving new subscriber into the database")]
pub async fn insert_subscriber(pool: &PgPool, data: &Subscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        data.email.as_ref(),
        data.name.as_ref(),
        chrono::Utc::now(),
    )
    .execute(pool)
    .await
    .inspect_err(|e| tracing::error!("Failed to execute query: {e:?}"))?;

    Ok(())
}
