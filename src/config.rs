use std::fmt::Display;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    #[serde(rename = "database")]
    pub database_settings: DatabaseSettings,
    #[serde(rename = "application")]
    pub app_settings: AppSettings,
}

#[derive(Deserialize)]
pub struct AppSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.name
            )
            .into(),
        )
    }

    pub fn connection_string_without_db(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port
            )
            .into(),
        )
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let config_directory = base_path.join("config");

    let env: Environment = std::env::var("APP_ENVIRONMENT")
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let env_config = format!("{env}.yaml");

    let settings = config::Config::builder()
        .add_source(config::File::from(config_directory.join("base.yaml")))
        .add_source(config::File::from(config_directory.join(env_config)))
        .build()?;

    settings.try_deserialize()
}

pub enum Environment {
    Development,
    Production,
}

impl TryFrom<Result<String, std::env::VarError>> for Environment {
    type Error = String;

    fn try_from(value: Result<String, std::env::VarError>) -> Result<Self, Self::Error> {
        value.unwrap_or_else(|_| "dev".into()).try_into()
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "dev" => Ok(Self::Development),
            "prod" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `dev` or `prod`."
            )),
        }
    }
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &'static str {
        match self {
            Environment::Development => "dev",
            Environment::Production => "prod",
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
