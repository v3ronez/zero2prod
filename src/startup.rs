use actix_web::web;
use actix_web::{App, HttpServer, dev::Server};
use sqlx::PgPool;
use std::{io::Error, net::TcpListener};

use crate::routes::{health_check, subscription};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, Error> {
    // Web::Data::new ->> Arc<T>
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/v1")
                    .route("/health-check", web::get().to(health_check))
                    .route("/subscriptions", web::post().to(subscription)),
            )
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
