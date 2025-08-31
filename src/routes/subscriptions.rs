use actix_web::{HttpResponse, web};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(Deserialize, Debug)]
pub struct Subscription {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding new subscriber",
    skip(form, pool),
    fields(
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(form: web::Form<Subscription>, pool: web::Data<PgPool>) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_e) => return HttpResponse::BadRequest().finish(),
    };

    let new_subscriver = NewSubscriber {
        name: name,
        email: form.0.email,
    };
    match insert_subscriber(&pool, &new_subscriver).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_err) => HttpResponse::BadRequest().finish(),
    }
}

#[tracing::instrument(name = "Saving a new subscriber", skip(pool, form))]
async fn insert_subscriber(pool: &PgPool, form: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::now_v7(),
        form.email,
        form.name.as_ref(),
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
