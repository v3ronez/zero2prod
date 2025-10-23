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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    let get_link = async |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        let assert_result = std::panic::catch_unwind(|| {
            assert_eq!(links.len(), 1);
        });

        if assert_result.is_err() {
            drop_database(&app.connection_pool).await;
        }
        links[0].as_str().to_owned()
    };
    let raw_confirmation_link = &get_link(&body["html_content"].as_str().unwrap()).await;

    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();

    let assert_result =
        std::panic::catch_unwind(|| assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1"));
    if assert_result.is_err() {
        drop_database(&app.connection_pool).await;
        std::panic::resume_unwind(assert_result.err().unwrap());
    }
    confirmation_link.set_port(Some(app.port)).unwrap();

    let response = reqwest::get(confirmation_link).await.unwrap();
    let status = response.status().as_u16();
    let assert_result = std::panic::catch_unwind(|| assert_eq!(status, 200));
    if assert_result.is_err() {
        drop_database(&app.connection_pool).await;
        std::panic::resume_unwind(assert_result.err().unwrap());
    }

    drop_database(&app.connection_pool).await;
}
