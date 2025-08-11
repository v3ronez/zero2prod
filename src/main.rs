use sqlx::pool::PoolOptions;
use std::{net::TcpListener, time::Duration};
use zero2prod::{configuration, startup::run, telemetry};

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
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = TcpListener::bind(&address)?;
    let connection_pool = PoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy(&settings.database.connection_string())
        .expect("Failed to connection to postgres");

    println!("Server running on: {address}");
    run(listener, connection_pool)?.await?;
    Ok(())
}
