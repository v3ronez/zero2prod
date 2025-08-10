use std::env;

use config::{Config, File};
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretBox<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn connection_string_without_database(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/postgres",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
    }
}
enum Enviroment {
    Local,
    Production,
}
impl Enviroment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Enviroment::Local => "local",
            Enviroment::Production => "production",
        }
    }
}
impl TryFrom<String> for Enviroment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = env::current_dir().expect("Failed to get base path");
    let config_dir = base_path.join("configurations");
    let base_config = config_dir.join("base.yaml");

    let environment: Enviroment = env::var("APP_ENVIROMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIROMENT.");

    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = Config::builder()
        .add_source(File::from(base_config))
        .add_source(File::from(config_dir.join(environment_filename.as_str())))
        .build()?;

    settings.try_deserialize::<Settings>()
}
