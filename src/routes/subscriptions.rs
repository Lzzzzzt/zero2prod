use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[instrument(
    skip_all,
    name = "Adding a new subscriber"
    fields(subsriber_name = %data.name, subscriber_email = %data.email)
)]
pub async fn subscribe(State(pool): State<PgPool>, Form(data): Form<FormData>) -> StatusCode {
    match insert_subscriber(&pool, &data).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[instrument(skip_all, name = "Saving new subscriber into the database")]
pub async fn insert_subscriber(pool: &PgPool, data: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        data.email,
        data.name,
        chrono::Utc::now(),
    )
    .execute(pool)
    .await
    .inspect_err(|e| tracing::error!("Failed to execute query: {e:?}"))?;

    Ok(())
}
