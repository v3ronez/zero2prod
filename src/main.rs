use std::{io::Result, net::TcpListener};

use sqlx::{Connection, PgConnection, PgPool};
use zero2prod::{configuration, startup::run};

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");
    let port = configuration.application_port;
    let address = format!("127.0.0.1:{}", port);
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to establish new connection on Postgres.");
    let listener = TcpListener::bind(address).expect("Error to bind the 8000 port");

    run(listener, connection_pool)?.await
}
