use actix_web::HttpResponse;
use actix_web::web::{Data, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{Instrument, instrument};
use uuid::Uuid;

#[allow(unused)]
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[instrument(
    name = "Adding a new subscriber", 
    skip_all,
    fields(subscriber.email = %form.email, subscriber.name = %form.name)
)]
pub async fn subscribe(Form(form): Form<FormData>, connection_pool: Data<PgPool>) -> HttpResponse {
    let query_span = tracing::info_span!("Saving new subscriber details in the database.");

    match sqlx::query!(
        "insert into subscriptions values ($1, $2, $3, $4);",
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection_pool.as_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {e:?}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[instrument(name = "Saving new subscriber details in the database", skip_all)]
pub async fn insert_subscriber(pool: &PgPool, name: &str, email: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "insert into subscriptions values ($1, $2, $3, $4);",
        Uuid::new_v4(),
        email,
        name,
        Utc::now()
    )
    .execute(pool)
    .await
    .inspect_err(|e| tracing::error!("Failed to execute query: {e:?}"))?;

    Ok(())
}
