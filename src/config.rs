use std::fmt::Display;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "application")]
    pub app_config: AppConfig,
    #[serde(rename = "database")]
    pub db_config: DBConfig,
}

#[derive(Deserialize)]
pub struct AppConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Clone)]
pub struct DBConfig {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    #[serde(rename = "database_name")]
    pub db_name: String,
    pub ssl: bool,
}

impl DBConfig {
    pub fn connection_options(&self) -> PgConnectOptions {
        let ssl = match self.ssl {
            true => PgSslMode::Require,
            false => PgSslMode::Prefer,
        };

        PgConnectOptions::new()
            .username(&self.username)
            .password(self.password.expose_secret())
            .host(&self.host)
            .port(self.port)
            .database(&self.db_name)
            .ssl_mode(ssl)
    }
}

pub enum Env {
    Dev,
    Prod,
}

impl TryFrom<String> for Env {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            s if s.starts_with("dev") => Ok(Env::Dev),
            s if s.starts_with("prod") => Ok(Env::Prod),
            s => Err(format!("{s} is not a valid `APP_ENV`.")),
        }
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Env::Dev => "dev",
            Env::Prod => "prod",
        })
    }
}

pub async fn get_config() -> Result<Config, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let config_path = base_path.join("config");

    let env: Env = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "dev".into())
        .try_into()
        .expect("Failed to parse `APP_ENV`");

    let env_config_file = format!("{env}.toml");

    let settings = config::Config::builder()
        .add_source(config::File::from(config_path.join("base.toml")))
        .add_source(config::File::from(config_path.join(env_config_file)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize()
}
