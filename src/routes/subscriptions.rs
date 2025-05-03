use actix_web::HttpResponse;
use actix_web::web::{Data, Form};
use chrono::Utc;
use sea_query::{Alias, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(unused)]
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(Form(form): Form<FormData>, connection_pool: Data<PgPool>) -> HttpResponse {
    let (sql, values) = Query::insert()
        .into_table(Alias::new("subscriptions"))
        .columns(["id", "name", "email", "subscribed_at"].map(Alias::new))
        .values_panic([
            Uuid::new_v4().into(),
            form.name.into(),
            form.email.into(),
            Utc::now().into(),
        ])
        .build_sqlx(PostgresQueryBuilder);

    match sqlx::query_with(&sql, values)
        .execute(connection_pool.as_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("{e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
