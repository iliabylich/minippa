use crate::bash::bash;
use anyhow::Result;
use futures::try_join;

pub(crate) struct SystemDeps;

impl SystemDeps {
    pub(crate) async fn ensure_installed() -> Result<()> {
        try_join!(
            bash!("dpkg-scanpackages --version"),
            bash!("apt-ftparchive --version"),
            bash!("gzip --version"),
            bash!("gpg --version"),
        )?;

        Ok(())
    }
}
