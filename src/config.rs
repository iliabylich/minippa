use anyhow::{Context as _, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) port: u16,
    pub(crate) token: String,
    pub(crate) dir: String,
}

const CONFIG_PATH: &str = if cfg!(debug_assertions) {
    "config.toml"
} else {
    "/etc/minippa.toml"
};

pub(crate) const EMAIL: &str = "owner@this-repo.org";
pub(crate) const NAME: &str = "Owner Name";

impl Config {
    pub(crate) async fn read() -> Result<Self> {
        let contents = tokio::fs::read_to_string(CONFIG_PATH)
            .await
            .with_context(|| format!("failed to read config at {CONFIG_PATH}"))?;
        let config: Config = toml::from_str(&contents).context("failed to parse Config")?;
        log::info!("Running with config {:?}", config);
        Ok(config)
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("port", &self.port)
            .field("token", &"*****")
            .field("dir", &self.dir)
            .finish()
    }
}
