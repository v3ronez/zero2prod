use std::net::TcpListener;
use zero2prod::{configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = configuration::get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(&address)?;
    println!("Server running on: {address}");
    run(listener)?.await
}
