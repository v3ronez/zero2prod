use std::net::TcpListener;

use once_cell::sync::Lazy;
use secrecy::SecretString;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing_log::format_trace;
use uuid::Uuid;
use zero2prod::{
    configuration::{self, DatabaseSettings},
    email_client::EmailClient,
    startup::{Application, get_connection_pool, run},
    telemetry,
};

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

pub struct TestApp {
    pub address: String,
    pub connection_pool: sqlx::PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configurations = {
        let mut c = configuration::get_configuration().expect("Failed to read configuration file");
        c.database.database_name = Uuid::now_v7().to_string();
        c.application.port = 0;
        c
    };
    let app = Application::build(configurations.clone())
        .await
        .expect("Error to init the application on tests");

    let address = format!("http://127.0.0.1:{}", app.port());
    configure_database(&configurations.database).await;

    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address,
        connection_pool: get_connection_pool(&configurations.database),
    }
}

async fn configure_database(
    config: &configuration::DatabaseSettings,
) -> sqlx::Pool<sqlx::Postgres> {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretString::from("postgres".to_string()),
        ..config.clone()
    };
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

pub async fn drop_database(pool: &PgPool) {
    let _ = pool.close().await;
    let opts = pool.connect_options();
    let dbname = opts.get_database().unwrap();
    pool.close().await;
    let default_pool = configuration::get_configuration().unwrap();

    let new_pool = PgPool::connect_with(default_pool.database.without_db())
        .await
        .unwrap();

    //kill all connection before drop the database
    sqlx::query(
        format!(
            r#"SELECT pg_terminate_backend(pid)
                FROM pg_stat_activity
                WHERE datname = '{}';"#,
            dbname
        )
        .as_str(),
    )
    .execute(&new_pool)
    .await
    .unwrap();

    sqlx::query(format!(r#"DROP DATABASE IF EXISTS "{}";"#, dbname).as_str())
        .execute(&new_pool)
        .await
        .unwrap();
}
