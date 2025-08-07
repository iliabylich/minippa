use anyhow::{Context as _, Result};
use serde::Deserialize;
use tokio::sync::OnceCell;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) port: u16,
    pub(crate) token: String,
    pub(crate) dir: String,
}

#[cfg(debug_assertions)]
const CONFIG_PATH: &str = "config.toml";

#[cfg(not(debug_assertions))]
const CONFIG_PATH: &str = "/etc/minippa.toml";

pub(crate) const EMAIL: &str = "owner@this-repo.org";
pub(crate) const NAME: &str = "Owner Name";

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

    pub(crate) fn get() -> &'static Self {
        CONFIG.get().expect("Config is not initialized")
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
