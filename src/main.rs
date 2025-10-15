use sqlx::pool::PoolOptions;
use std::{net::TcpListener, time::Duration};
use zero2prod::{
    configuration,
    email_client::EmailClient,
    startup::{build, run},
    telemetry,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //tracing application
    let subscriber = telemetry::get_subscriber(
        String::from("zero2prod"),
        String::from("info"),
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber);

    let configurations = configuration::get_configuration().expect("Failed to read configuration");
    let server = build(configurations)?;
    server.await?;
    Ok(())
}
