use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::email_client::EmailClient;
use crate::routes::*;
use crate::telemetry::with_request_id;

pub struct App {
    listener: TcpListener,
    conn_pool: PgPool,
    email_client: EmailClient,
    port: u16,
}

impl App {
    pub async fn build(config: Config) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind((config.app_config.host, config.app_config.port)).await?;
        let conn_pool = PgPool::connect_lazy_with(config.db_config.connection_options());
        let email_client = EmailClient::from(config.email_client_config);
        let port = listener.local_addr().unwrap().port();

        Ok(Self {
            port,
            listener,
            conn_pool,
            email_client,
        })
    }

    #[inline]
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let mut router = Router::new()
            .route("/health_check", get(health_check))
            .route("/subscriptions", post(subscribe))
            .with_state(self.conn_pool)
            .with_state(Arc::new(self.email_client));

        router = with_request_id(router);

        axum::serve(self.listener, router).await
    }
}
