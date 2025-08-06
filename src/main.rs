use std::net::TcpListener;
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
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(&address)?;
    let connection_pool = sqlx::PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Failed to connection to postgres");

    println!("Server running on: {address}");
    run(listener, connection_pool)?.await
}
