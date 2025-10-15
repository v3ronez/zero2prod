use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use sqlx::pool::PoolOptions;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
pub fn build(configs: Settings) -> Result<Server, std::io::Error> {
    let connection_pool = PoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configs.database.with_db());

    let sender_email = configs
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = configs.email_client.timeout();

    let email_client = EmailClient::new(
        configs.email_client.base_url,
        sender_email,
        configs.email_client.authorization_token,
        timeout,
    );

    let address = format!("{}:{}", configs.application.host, configs.application.port);
    let listener = TcpListener::bind(&address)?;
    run(listener, connection_pool, email_client)
}
