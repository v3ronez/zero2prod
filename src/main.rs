use env_logger::Env;
use std::net::TcpListener;
use zero2prod::{configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let settings = configuration::get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(&address)?;
    let connection_pool = sqlx::PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Failed to connection to postgres");

    println!("Server running on: {address}");
    run(listener, connection_pool)?.await
}
