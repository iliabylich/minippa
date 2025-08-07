use crate::{
    bash::bash,
    config::{Config, EMAIL},
    index::Upload,
};
use anyhow::{Context as _, Result};

#[derive(Debug)]
pub(crate) struct RawIndex;

impl RawIndex {
    pub(crate) async fn new() -> Result<Self> {
        tokio::fs::create_dir_all(&Config::get().dir)
            .await
            .context("failed to create data dir")?;
        log::info!("Data dir at {} exists", Config::get().dir);

        Ok(Self)
    }

    pub(crate) async fn write(&self, filename: String, upload: Upload) -> Result<()> {
        let entry = upload.persist_as_entry(filename).await?;
        entry.remove_previous_versions().await?;
        reindex().await?;
        Ok(())
    }
}

async fn reindex() -> Result<()> {
    log::info!("Reindexing...");

    bash!(
        r#"cd "{dir}"
rm -f Packages Packages.gz Release Release.gpg InRelease
dpkg-scanpackages --multiversion . > Packages 2> /dev/null
gzip -k -f Packages
apt-ftparchive release . > Release
gpg --default-key "{EMAIL}" -abs -o - Release > Release.gpg
gpg --default-key "{EMAIL}" --clearsign -o - Release > InRelease
    "#,
        dir = Config::get().dir,
    )
    .await?;

    log::info!("Re-indexing has finished!");
    Ok(())
}
