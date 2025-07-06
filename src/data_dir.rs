use crate::{bash, config::Config};
use anyhow::{Context as _, Result, bail};
use std::path::PathBuf;

pub(crate) struct DataDir;

impl DataDir {
    pub(crate) async fn mkdir_p() -> Result<()> {
        tokio::fs::create_dir_all(Config::dir())
            .await
            .context("failed to create data dir")?;
        log::info!("Data dir at {} exists", Config::dir());

        Ok(())
    }

    pub(crate) async fn write(filename: String, data: Vec<u8>) -> Result<()> {
        log::info!("Saving {} ({} bytes)", filename, data.len());

        let tmp_file = TmpFile::new(&filename);
        tmp_file.write(data).await?;
        let deb_file = tmp_file.persist().await?;
        deb_file.remove_previous_versions().await?;

        Ok(())
    }
}

struct TmpFile {
    filename: String,
    filepath: PathBuf,
    dest: PathBuf,
}

impl TmpFile {
    fn new(filename: &str) -> Self {
        let filepath = PathBuf::from(Config::dir()).join(format!("{filename}.tmp"));
        let dest = PathBuf::from(Config::dir()).join(filename);

        Self {
            filename: filename.to_string(),
            filepath,
            dest,
        }
    }

    async fn write(&self, data: Vec<u8>) -> Result<()> {
        log::info!("Writing to {:?}", self.filepath);

        tokio::fs::write(&self.filepath, data)
            .await
            .with_context(|| format!("failed to write to {:?}", self.filepath))
    }

    async fn read_package_name(&self) -> Result<String> {
        log::info!("Reading package name...");

        let script = format!("dpkg -I {:?}", self.filepath);
        let stdout = bash::exec(&script).await?;

        stdout
            .lines()
            .filter_map(|line| line.trim().strip_prefix("Package: "))
            .next()
            .map(|s| s.to_string())
            .context("failed to find a line starting with 'Package:' in the output of dpkg -I")
    }

    async fn persist(self) -> Result<DebFile> {
        let package_name = self.read_package_name().await?;
        log::info!("Package name: {}", package_name);

        if !self.filename.contains(&package_name) {
            bail!(
                "filename doesn't contain package name ({} vs {})",
                self.filename,
                package_name
            );
        }

        log::info!("Renaming {:?} -> {:?}", self.filepath, self.dest);
        tokio::fs::rename(&self.filepath, &self.dest).await?;

        Ok(DebFile {
            filename: self.filename,
            package_name,
        })
    }
}

#[derive(Debug)]
struct DebFile {
    filename: String,
    package_name: String,
}

impl DebFile {
    async fn remove_previous_versions(&self) -> Result<()> {
        let mut dir = tokio::fs::read_dir(Config::dir())
            .await
            .context("failed to get files of data dir")?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .context("failed to get next data dir entry")?
        {
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

            let path = entry.path();
            log::info!(
                "Removing previous version of {}: {:?}",
                self.package_name,
                path
            );
            tokio::fs::remove_file(&path)
                .await
                .with_context(|| format!("failed to remove {path:?}"))?;
        }

        Ok(())
    }
}
