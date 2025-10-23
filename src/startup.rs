use crate::configuration::{Configurations, DatabaseSettings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, subscribe};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use sqlx::pool::PoolOptions;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    pub async fn build(configurations: Configurations) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configurations.database);

        let sender_email = configurations
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let timeout = configurations.email_client.timeout();

        let email_client = EmailClient::new(
            configurations.email_client.base_url.clone(),
            sender_email,
            configurations.email_client.authorization_token.clone(),
            timeout,
        );

        let address = format!(
            "{}:{}",
            configurations.application.host, configurations.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configurations.application.base_url,
        )?;
        Ok(Self { port, server })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug)]
pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(base_url.clone())
            .app_data(connection_pool.clone()) // each request receive a clone of connection pool
            .app_data(email_client.clone()) // each request receive a clone of email client
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn get_connection_pool(database: &DatabaseSettings) -> PgPool {
    PoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(database.with_db())
}
