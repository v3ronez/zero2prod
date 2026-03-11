use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool};
use zero2prod::{
    configuration::{self, get_configuration},
    startup::run,
};

struct TestApp {
    address: String,
    db_pool: PgPool,
}

#[tokio::test]
async fn health_check() {
    let app = spawn_app().await;
    let url = format!("{}/v1/health-check", &app.address);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to execute request ");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscriber_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let url = format!("{}/v1/subscriptions", &app.address);
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to establish new connection");

    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert!(response.status().is_success());
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscriber_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let url = format!("{}/v1/subscriptions", &app.address);
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error_message) in test_cases {
        let response = client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        let http_code = response.status().as_u16();

        assert_eq!(
            400, http_code,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let configurations = configuration::get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configurations.database.connection_string())
        .await
        .expect("Failed to create a pool connection");
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}
