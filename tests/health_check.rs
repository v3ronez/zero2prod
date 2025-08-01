use std::{io::Bytes, net::TcpListener};

use actix_web::{App, HttpServer, web};
use sqlx::{Connection, PgPool, Postgres};
use zero2prod::configuration::{self, get_configuration};

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let settings = configuration::get_configuration().expect("Failed to read configuration file");
    let connection_pool = sqlx::PgPool::connect(&settings.database.connection_string())
        .await
        .unwrap();

    let server = zero2prod::startup::run(listener, connection_pool.clone()).unwrap();

    let _ = tokio::spawn(server);
    TestApp {
        address,
        connection_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    println!("\ntest server run on address: {}\n", &app.address);
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connect = sqlx::PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect on database");

    let client = reqwest::Client::new();
    let body = "name=Henrique%20Veronez&email=v3ronez.dev%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    //assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connect)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "v3ronez.dev@gmail.com");
    assert_eq!(saved.name, "Henrique Veronez");
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
        //Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

struct TestApp {
    address: String,
    connection_pool: sqlx::PgPool,
}
