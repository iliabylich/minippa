mod entry;
mod upload;

use crate::{bash::bash, config::Config};
use anyhow::{Context as _, Result};
use upload::Upload;

pub(crate) struct Index;

impl Index {
    pub(crate) async fn mkdir_p() -> Result<()> {
        tokio::fs::create_dir_all(Config::dir())
            .await
            .context("failed to create data dir")?;
        log::info!("Data dir at {} exists", Config::dir());

        Ok(())
    }

    pub(crate) async fn write(filename: String, data: Vec<u8>) -> Result<()> {
        log::info!("Saving {} ({} bytes)", filename, data.len());

        let upload = Upload::new_in_tmp(data).await?;
        let entry = upload.persist_as_entry(filename).await?;
        entry.remove_previous_versions().await?;
        reindex().await?;

        Ok(())
    }
}

pub(crate) async fn reindex() -> Result<()> {
    log::info!("Reindexing...");

    bash!(
        r#"cd "{dir}"
rm -f Packages Packages.gz Release Release.gpg InRelease
dpkg-scanpackages --multiversion . > Packages 2> /dev/null
gzip -k -f Packages
apt-ftparchive release . > Release
gpg --default-key "{email}" -abs -o - Release > Release.gpg
gpg --default-key "{email}" --clearsign -o - Release > InRelease
    "#,
        dir = Config::dir(),
        email = Config::email(),
    )
    .await?;

    log::info!("Re-indexing has finished!");
    Ok(())
}
