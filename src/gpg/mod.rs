mod config;

use crate::{bash::bash, config::EMAIL};
use anyhow::{Result, bail};
use config::GpgConfig;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct GPG;

impl GPG {
    pub(crate) async fn generate_key() -> Result<()> {
        let key_exists = bash!("gpg --list-key {EMAIL}")
            .await
            .is_ok_and(|stdout| stdout.contains(EMAIL));
        if key_exists {
            bail!("GPG key for {EMAIL} already exists. Either use it as is or remove manually.",);
        }

        let gpg_config = GpgConfig::new().await?;
        bash!("gpg --batch --gen-key {:?}", gpg_config.path()).await?;

        let key = bash!("gpg --list-key {EMAIL}").await?;
        log::info!("Success:\n{key}");

        Ok(())
    }

    pub(crate) async fn export_key() -> Result<String> {
        bash!("gpg --armor --export {EMAIL}").await
    }
}
