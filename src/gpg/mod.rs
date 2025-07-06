mod config;

use crate::{bash::bash, config::Config};
use anyhow::{Context as _, Result, bail};
use config::GpgConfig;
use std::path::Path;

const EMAIL: &str = Config::email();

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct GPG;

impl GPG {
    pub(crate) async fn generate_key() -> Result<()> {
        if key_exists().await {
            bail!("GPG key for {EMAIL} already exists. Either use it as is or remove manually.",);
        }
        let gpg_config = GpgConfig::new().await?;
        generate(gpg_config).await?;

        Ok(())
    }

    pub(crate) async fn export_key() -> Result<()> {
        let stdout = bash!("gpg --armor --export {EMAIL}").await?;

        let dst = Path::new(Config::dir()).join("public.gpg");
        tokio::fs::write(dst, stdout)
            .await
            .context("failed to write public.gpg")?;

        Ok(())
    }
}

async fn key_exists() -> bool {
    match bash!("gpg --list-key {EMAIL}").await {
        Ok(stdout) => stdout.contains(EMAIL),
        Err(_) => false,
    }
}

async fn generate(gpg_config: GpgConfig) -> Result<()> {
    bash!("gpg --batch --gen-key {:?}", gpg_config.path()).await?;

    let key = bash!("gpg --list-key {EMAIL}").await?;
    log::info!("Success:\n{key}");

    Ok(())
}
