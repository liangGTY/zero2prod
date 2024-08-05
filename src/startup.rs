use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::{Pool, Postgres};

use crate::routes::{health_check, subscriptions};

pub fn run(listener: TcpListener, pool: Pool<Postgres>) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
