use crate::{bash, config::Config};
use anyhow::{Context as _, Result, bail};
use std::{io::Write, path::Path};
use tempfile::NamedTempFile;

pub(crate) struct GPG;

impl GPG {
    pub(crate) async fn generate_key() -> Result<()> {
        if Self::key_exists().await {
            bail!(
                "GPG key for {} already exists. Either use it as is or remove manually.",
                Config::owner_email()
            );
        }
        let config_tmpfile = Self::write_config()?;
        Self::generate(config_tmpfile.path()).await?;

        Ok(())
    }

    async fn key_exists() -> bool {
        let script = format!("gpg --list-key \"{}\"", Config::owner_email());
        match bash::exec(&script).await {
            Ok(stdout) => stdout.contains(Config::owner_email()),
            Err(_) => false,
        }
    }

    fn write_config() -> Result<NamedTempFile> {
        let config = format!(
            r#"%no-protection
Key-Type: RSA
Key-Length: 4096
Key-Usage: cert,sign
Subkey-Type: RSA
Subkey-Length: 4096
Subkey-Usage: encrypt
Name-Real: {name}
Name-Email: {email}
Expire-Date: 0
%commit
"#,
            name = Config::owner_name(),
            email = Config::owner_email()
        );

        let mut tmpfile = NamedTempFile::new().context("failed to create tempfile")?;
        tmpfile
            .write_all(config.as_bytes())
            .context("failed to write GPG config to tempfile")?;
        Ok(tmpfile)
    }

    async fn generate(path: &Path) -> Result<()> {
        let script = format!(
            "gpg --batch --gen-key {path:?}; gpg --list-key {}",
            Config::owner_email()
        );
        let stdout = bash::exec(&script).await?;
        log::info!("Success:\n{stdout}");
        Ok(())
    }

    pub(crate) async fn export_key() -> Result<()> {
        let script = format!("gpg --armor --export {}", Config::owner_email());
        let stdout = bash::exec(&script).await?;
        tokio::fs::write(Path::new(Config::dir()).join("public.gpg"), stdout)
            .await
            .context("failed to write public.gpg")?;

        Ok(())
    }
}
