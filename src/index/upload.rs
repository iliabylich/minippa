use crate::{bash::bash, config::Config, index::entry::Entry};
use anyhow::{Context as _, Result, bail};
use async_tempfile::TempFile;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt as _;

pub(crate) struct Upload {
    tempfile: TempFile,
    package_name: String,
}

impl Upload {
    pub(crate) async fn new_in_tmp(data: Vec<u8>) -> Result<Self> {
        let mut tempfile = TempFile::new().await.context("failed to create tempfile")?;

        tempfile
            .write_all(&data)
            .await
            .context("failed to write contents of DEB file to tempfile")?;

        log::info!("Reading package name...");

        let stdout = bash!("dpkg -I {:?}", tempfile.file_path()).await?;

        let package_name = stdout
            .lines()
            .filter_map(|line| line.trim().strip_prefix("Package: "))
            .next()
            .map(|s| s.to_string())
            .context("failed to find a line starting with 'Package:' in the output of dpkg -I")?;

        log::info!("Package name: {package_name}");

        Ok(Self {
            tempfile,
            package_name,
        })
    }

    pub(crate) async fn persist_as_entry(self, filename: String) -> Result<Entry> {
        let Self {
            package_name,
            tempfile,
        } = self;

        if !filename.contains(&package_name) {
            bail!("filename doesn't contain package name ({filename} vs {package_name})",);
        }

        let src = tempfile.file_path().clone();
        let dst = PathBuf::from(Config::dir()).join(&filename);

        log::info!("Copying {src:?} -> {dst:?}");
        tokio::fs::copy(&src, &dst)
            .await
            .with_context(|| format!("failed to copy {src:?} -> {dst:?}"))?;

        Ok(Entry::new(filename, package_name))
    }
}
