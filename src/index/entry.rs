use crate::config::Config;
use anyhow::{Context as _, Result};

#[derive(Debug)]
pub(crate) struct Entry {
    filename: String,
    package_name: String,
}

impl Entry {
    pub(crate) fn new(filename: String, package_name: String) -> Self {
        Self {
            filename,
            package_name,
        }
    }

    pub(crate) async fn remove_previous_versions(&self) -> Result<()> {
        let mut dir = tokio::fs::read_dir(Config::dir())
            .await
            .context("failed to list files of data dir")?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .context("failed to get next data dir entry")?
        {
            let filepath = entry.path();
            let filename = entry.file_name();
            let Some(filename) = filename.to_str() else {
                continue;
            };
            if !filename.contains(&self.package_name) {
                // other package
                continue;
            }
            if filename == self.filename {
                // prevent removing uploaded version of the package
                continue;
            }

            log::info!(
                "Removing previous version of {}: {:?}",
                self.package_name,
                filepath
            );
            tokio::fs::remove_file(&filepath)
                .await
                .with_context(|| format!("failed to remove {filepath:?}"))?;
        }

        Ok(())
    }
}
