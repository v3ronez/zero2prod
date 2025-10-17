use crate::helpers::{drop_database, spawn_app};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    let http_code = response.status().as_u16().clone();
    let text = &response.text().await;
    dbg!(text);
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
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
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
        if assert_result.is_err() {
            drop_database(&app.connection_pool).await;
        }
    }

    drop_database(&app.connection_pool).await;
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in body {
        //Act
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
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

        if assert_result.is_err() {
            drop_database(&app.connection_pool).await;
        }
    }

    drop_database(&app.connection_pool).await;
}
