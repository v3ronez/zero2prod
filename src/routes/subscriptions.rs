use actix_web::{HttpResponse, web};
use serde::Deserialize;
use sqlx::PgConnection;

#[derive(Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

pub async fn subscribe(
    _form: web::Form<Subscription>,
    _connection: web::Data<PgConnection>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}
