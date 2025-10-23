use std::fmt::format;

use reqwest::Url;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::{drop_database, spawn_app};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;

    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/api/send/2317403"))
        .and(method("post"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    app.post_subscriptions(body).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = app.get_confirmation_link(email_request).await;

    let response = reqwest::get(confirmation_link.html).await.unwrap();
    let status = response.status().as_u16();
    let assert_result = std::panic::catch_unwind(|| assert_eq!(status, 200));
    if assert_result.is_err() {
        drop_database(&app.connection_pool).await;
        std::panic::resume_unwind(assert_result.err().unwrap());
    }

    drop_database(&app.connection_pool).await;
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/api/send/2317403"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = app.get_confirmation_link(email_request).await;

    reqwest::get(&confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    let email_are_equals = std::panic::catch_unwind(|| {
        assert_eq!(saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(saved.name, "le guin");
        assert_eq!(saved.status, "confirmed");
    });
    if email_are_equals.is_err() {
        drop_database(&app.connection_pool).await;
        std::panic::resume_unwind(email_are_equals.err().unwrap());
    }
    drop_database(&app.connection_pool).await;
}
