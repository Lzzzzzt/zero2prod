use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub port: u16,
    #[serde(rename = "database")]
    pub db_config: DBConfig,
}

#[derive(Deserialize, Clone)]
pub struct DBConfig {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    #[serde(rename = "database_name")]
    pub db_name: String,
}

impl DBConfig {
    pub fn connection_url(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.db_name
        )
        .into()
    }
}

pub async fn get_config() -> Result<Config, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new("config.toml", config::FileFormat::Toml))
        .build()?;

    settings.try_deserialize()
}
