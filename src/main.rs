use zero2prod::{configuration, startup::Application, telemetry};

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
    let app = Application::build(configurations).await?;
    app.run_until_stopped().await?;
    Ok(())
}
