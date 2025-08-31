use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{configuration, telemetry};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut settings =
        configuration::get_configuration().expect("Failed to read configuration file");
    settings.database.database_name = Uuid::now_v7().to_string();

    let connection_pool = configure_database(&settings.database).await;

    let server = zero2prod::startup::run(listener, connection_pool.clone()).unwrap();

    let _ = tokio::spawn(server);
    TestApp {
        address,
        connection_pool,
    }
}

async fn configure_database(
    config: &configuration::DatabaseSettings,
) -> sqlx::Pool<sqlx::Postgres> {
    let query = format!(r#"CREATE DATABASE "{}";"#, &config.database_name.as_str());
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect(
            format!(
                "Failed to connect on {}",
                config
                    .with_db()
                    .get_database()
                    .expect("Error to get database name")
            )
            .as_str(),
        );

    connection
        .execute(query.as_str())
        .await
        .expect(format!("Error to execute this query: {}", query).as_str());

    let connection_pool = PgPool::connect_with(config.with_db()).await.expect(
        format!(
            "Failed to connect on {}",
            config
                .with_db()
                .get_database()
                .expect("Failed to get database name")
        )
        .as_str(),
    );

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Error to run migration on test");

    connection_pool
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

    //reset db
    drop_database(&app.connection_pool).await;
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
        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not return a 200 OK when the payload was {}",
            description
        );
    }
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");

    //reset db
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
        //Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }

    //reset db
    drop_database(&app.connection_pool).await;
}

async fn drop_database(pool: &PgPool) {
    let opts = pool.connect_options();
    let dbname = opts.get_database().unwrap();
    pool.close().await;
    let default_pool = configuration::get_configuration().unwrap();

    let new_pool = PgPool::connect_with(default_pool.database.without_db())
        .await
        .unwrap();

    sqlx::query(format!(r#"DROP DATABASE "{}";"#, dbname).as_str())
        .execute(&new_pool)
        .await
        .unwrap();
}

struct TestApp {
    address: String,
    connection_pool: sqlx::PgPool,
}
