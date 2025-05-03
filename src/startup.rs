use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::{Data, get, post};
use actix_web::{App, HttpServer};
use sqlx::PgPool;

use crate::routes;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let connection = Data::new(connection_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", get().to(routes::health_check))
            .route("/subscriptions", post().to(routes::subscribe))
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
