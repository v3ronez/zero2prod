use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::{drop_database, spawn_app};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/api/send/2317403"))
        .and(method("post"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_client)
        .await;

    // Act
    let response = app.post_subscriptions(body).await;

    // Assert
    let http_code = response.status().as_u16().clone();
    let assert_result = std::panic::catch_unwind(|| assert_eq!(200, http_code));

    //reset db
    match assert_result {
        Ok(_) => drop_database(&app.connection_pool).await,
        Err(msg) => {
            drop_database(&app.connection_pool).await;
            std::panic::resume_unwind(msg)
        }
    }
}

#[tokio::test]
async fn subscriber_returns_400_when_fields_are_empty() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        let response = app.post_subscriptions(body).await;

        //assert
        let http_code = response.status().as_u16().clone();

        let assert_result = std::panic::catch_unwind(|| {
            assert_eq!(
                400, http_code,
                "The api did not return a 200 OK when the payload was {}",
                description
            );
        });

        //reset db
        match assert_result {
            Ok(_) => continue,
            Err(msg) => {
                drop_database(&app.connection_pool).await;
                std::panic::resume_unwind(msg)
            }
        }
    }

    drop_database(&app.connection_pool).await;
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let app = spawn_app().await;
    let body = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in body {
        //Act
        let response = app.post_subscriptions(invalid_body).await;

        let http_code = response.status().as_u16().clone();
        //Assert
        let assert_result = std::panic::catch_unwind(|| {
            assert_eq!(
                400, http_code,
                // Additional customised error message on test failure
                "The API did not fail with 400 Bad Request when the payload was {}.",
                error_message
            )
        });

        match assert_result {
            Ok(_) => continue,
            Err(msg) => {
                drop_database(&app.connection_pool).await;
                std::panic::resume_unwind(msg)
            }
        }
    }
    drop_database(&app.connection_pool).await;
}

#[tokio::test]
async fn subscriber_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=henriquetest%20veronez&email=henriqueteste@gmail.com";

    Mock::given(path("/api/send/2317403"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_client)
        .await;
    app.post_subscriptions(body).await;
    drop_database(&app.connection_pool).await;
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin@gmail.com";

    Mock::given(path("/api/send/2317403"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_client)
        .await;

    app.post_subscriptions(body).await;
    let email_request = &app.email_client.received_requests().await.unwrap()[0];
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
    let html_link = get_link(&body["html_content"].as_str().unwrap()).await;
    let text_link = get_link(&body["text_content"].as_str().unwrap()).await;

    let email_are_equals = std::panic::catch_unwind(|| {
        assert_eq!(html_link, text_link);
    });

    match email_are_equals {
        Ok(_) => drop_database(&app.connection_pool).await,
        Err(msg) => {
            drop_database(&app.connection_pool).await;
            std::panic::resume_unwind(msg)
        }
    }
}
