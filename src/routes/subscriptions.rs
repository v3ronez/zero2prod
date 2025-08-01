use actix_web::{HttpResponse, web};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: web::Form<Subscription>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    let row = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::now_v7(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await;
    match row {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => {
            eprintln!("{}", err.to_string());
            HttpResponse::BadRequest().finish()
        }
    }
}
