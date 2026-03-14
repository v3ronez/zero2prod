use std::{io::Result, net::TcpListener};

use env_logger::Env;
use sqlx::PgPool;
use zero2prod::{configuration, startup::run};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = configuration::get_configuration().expect("Failed to read configuration");
    let port = configuration.application_port;
    let address = format!("127.0.0.1:{}", port);
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to establish new connection on Postgres.");
    let listener = TcpListener::bind(address).expect("Error to bind the 8000 port");

    run(listener, connection_pool)?.await
}
