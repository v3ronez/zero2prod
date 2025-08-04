use env_logger::Env;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
use zero2prod::{configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(JsonStorageLayer);
    let _ = set_global_default(subscriber);

    let settings = configuration::get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(&address)?;
    let connection_pool = sqlx::PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Failed to connection to postgres");

    println!("Server running on: {address}");
    run(listener, connection_pool)?.await
}
