use std::io::Error;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, dev::Server, web};

pub fn run() -> Result<Server, Error> {
    let server = HttpServer::new(|| {
        App::new().service(
            web::scope("/v1")
                .route("/", web::get().to(great))
                .route("/health-check", web::get().to(health_check))
                .route("/{name}", web::get().to(great)),
        )
    })
    .bind("0.0.0.0:8000")?
    .run();

    Ok(server)
}

async fn great(request: HttpRequest) -> impl Responder {
    let name = request.match_info().get("name").unwrap_or("ze");
    format!("Hello {name}")
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}
