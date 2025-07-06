use anyhow::{Context as _, Result};
use serde::Deserialize;
use tokio::sync::OnceCell;

#[derive(Deserialize)]
pub(crate) struct Config {
    port: u16,
    token: String,
    dir: String,
}

#[cfg(debug_assertions)]
const CONFIG_PATH: &str = "config.toml";

#[cfg(not(debug_assertions))]
const CONFIG_PATH: &str = "/etc/minippa.toml";

static CONFIG: OnceCell<Config> = OnceCell::const_new();

impl Config {
    pub(crate) async fn read() -> Result<()> {
        let contents = tokio::fs::read_to_string(CONFIG_PATH)
            .await
            .with_context(|| format!("failed to read config at {CONFIG_PATH}"))?;

        let config: Config = toml::from_str(&contents).context("failed to parse Config")?;
        log::info!("Running with config {:?}", config);

        CONFIG
            .set(config)
            .context("Config::read() must be called once")?;

        Ok(())
    }

    fn get() -> &'static Self {
        CONFIG.get().expect("Config is not initialized")
    }

    pub(crate) fn port() -> u16 {
        Self::get().port
    }

    pub(crate) fn token() -> &'static str {
        Self::get().token.as_str()
    }

    pub(crate) fn dir() -> &'static str {
        Self::get().dir.as_str()
    }

    pub(crate) const fn email() -> &'static str {
        "owner@this-repo.org"
    }

    pub(crate) const fn name() -> &'static str {
        "Owner Name"
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("port", &self.port)
            .field("token", &"*****")
            .field("dir", &self.dir)
            .field("email", &Self::email())
            .field("name", &Self::name())
            .finish()
    }
}
