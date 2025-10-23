use actix_web::{HttpResponse, web};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ConfirmParams {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: web::Query<ConfirmParams>) -> HttpResponse {
    return HttpResponse::Ok().finish();
}
