use std::net::TcpListener;

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{self, DatabaseSettings},
    routes::SubscriptionForm,
    startup::run,
};

#[derive(Debug)]
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
        .fetch_one(&app.db_pool)
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
    let mut configurations =
        configuration::get_configuration().expect("Failed to read configuration.");

    configurations.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configurate_database(&configurations.database).await;
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

async fn configurate_database(db_settings: &DatabaseSettings) -> PgPool {
    let pool = PgPool::connect(&db_settings.connection_string_without_dbname())
        .await
        .expect("Failed to connect to Postgres");

    let raw_sql = format!(r#"CREATE DATABASE "{}";"#, &db_settings.database_name);

    sqlx::raw_sql(raw_sql.as_str())
        .execute(&pool)
        .await
        .expect("Failed to execute the sql to create new database");

    let connection = PgPool::connect(&db_settings.connection_string())
        .await
        .expect(
            format!(
                "Failed to create new pool on database: {}",
                &db_settings.database_name
            )
            .as_str(),
        );

    sqlx::migrate!("./migrations")
        .run(&connection)
        .await
        .expect("Failed to run migrations");

    connection
}
