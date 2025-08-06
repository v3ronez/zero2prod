use actix_web::{
    HttpResponse,
    web::{self, Form},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{PgPool, Pool};
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding new subscriber", 
    skip(form, pool),
    fields(
        request_id=%Uuid::new_v4(),
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(form: web::Form<Subscription>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(&form, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_err) => HttpResponse::BadRequest().finish(),
    }
}

#[tracing::instrument(name = "Saving a new subscriber", skip(pool, form))]
async fn insert_subscriber(form: &Subscription, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::now_v7(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(())
}
