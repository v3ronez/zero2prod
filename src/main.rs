use sqlx::pool::PoolOptions;
use std::{net::TcpListener, time::Duration};
use zero2prod::{configuration, email_client::EmailClient, startup::run, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //tracing application
    let subscriber = telemetry::get_subscriber(
        String::from("zero2prod"),
        String::from("info"),
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber);

    let settings = configuration::get_configuration().expect("Failed to read configuration");
    let sender_email = settings.email_client.sender().unwrap();
    let email_client = EmailClient::new(
        settings.email_client.base_url,
        sender_email,
        settings.email_client.authorization_token,
    );
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = TcpListener::bind(&address)?;
    let connection_pool = PoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy_with(settings.database.with_db());
    println!("Server running on: {address}");
    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
