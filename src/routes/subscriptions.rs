use actix_web::{HttpResponse, web};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<Subscription>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
