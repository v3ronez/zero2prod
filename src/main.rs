use std::io::Result;

use actix_web::{App, HttpRequest, HttpServer, Responder, web};

async fn great(request: HttpRequest) -> impl Responder {
    let name = request.match_info().get("name").unwrap_or("ze");
    format!("Hello {name}")
}

#[tokio::main]
async fn main() -> Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::scope("/v1")
                .route("/", web::get().to(great))
                .route("/{name}", web::get().to(great)),
        )
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
