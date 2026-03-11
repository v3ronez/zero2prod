use actix_web::{
    HttpResponse,
    web::{Data, Form},
};
use sqlx::{PgPool, types::time::OffsetDateTime};
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct SubscriptionForm {
    email: String,
    name: String,
}

pub async fn subscription(form: Form<SubscriptionForm>, pool: Data<PgPool>) -> HttpResponse {
    let current_time = OffsetDateTime::now_utc();
    match sqlx::query!(
        r#"
    insert into subscriptions (id, email, name, subscribed_at, created_at, updated_at) values ($1, $2, $3, $4, $5, $6)
    "#,
        Uuid::now_v7(),
        form.email,
        form.name,
        current_time,
        current_time,
        current_time
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
