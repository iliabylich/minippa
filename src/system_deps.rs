use anyhow::{Context as _, Result, bail};
use tokio::process::Command;

pub(crate) struct SystemDeps;

impl SystemDeps {
    pub(crate) async fn ensure_installed() -> Result<()> {
        Self::ensure_exists("dpkg-scanpackages", "dpkg-dev").await?;
        Self::ensure_exists("apt-ftparchive", "apt-utils").await?;
        Self::ensure_exists("gzip", "gzip").await?;
        Self::ensure_exists("gpg", "gpg").await?;

        Ok(())
    }

    async fn ensure_exists(bin: &str, suggested_package_name: &str) -> Result<()> {
        let cmd = format!("which {bin}");

        macro_rules! not_found {
            () => {
                bail!("{bin} does not exist, try installing {suggested_package_name}");
            };
        }

        log::info!("Running {cmd}...");
        let output = Command::new("which")
            .arg(bin)
            .output()
            .await
            .with_context(|| format!("failed to run '{cmd}'"))?;

        if !output.status.success() {
            not_found!();
        }

        let stdout = String::from_utf8(output.stdout)
            .with_context(|| format!("non-utf-8 stdout of '{cmd}'"))?;
        let Some(at) = stdout.lines().next() else {
            not_found!();
        };

        log::info!("{bin} exists at {at}");
        Ok(())
    }
}
