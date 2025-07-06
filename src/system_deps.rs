use crate::bash::bash;
use anyhow::Result;

pub(crate) struct SystemDeps;

impl SystemDeps {
    pub(crate) async fn ensure_installed() -> Result<()> {
        bash!("dpkg-scanpackages --version").await?;
        bash!("apt-ftparchive --version").await?;
        bash!("gzip --version").await?;
        bash!("gpg --version").await?;

        Ok(())
    }
}
