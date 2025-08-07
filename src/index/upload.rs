use crate::{bash::bash, config::EMAIL};
use anyhow::{Context as _, Result, bail};
use async_tempfile::TempFile;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt as _;

pub(crate) async fn upload(dir: &str, filename: String, data: Vec<u8>) -> Result<()> {
    ensure_filename_is_just_a_filename(&filename)?;
    let tempfile = write_tmpfile(data).await?;
    let package_name = read_package_name(&tempfile).await?;

    if !filename.contains(&package_name) {
        bail!("filename doesn't contain package name ({filename} vs {package_name})",);
    }

    remove_previous_versions(dir, &package_name).await?;
    persist(tempfile, dir, &filename).await?;
    reindex(dir).await?;

    Ok(())
}

fn ensure_filename_is_just_a_filename(filename: &str) -> Result<()> {
    let path = Path::new(filename);
    let components = path.components().collect::<Vec<_>>();
    if components.len() == 1 {
        Ok(())
    } else {
        bail!("expected a clean filename, got {filename:?}")
    }
}

async fn write_tmpfile(data: Vec<u8>) -> Result<TempFile> {
    log::info!("Writing tempfile");

    let mut tempfile = TempFile::new().await.context("failed to create tempfile")?;

    tempfile
        .write_all(&data)
        .await
        .context("failed to write contents of DEB file to tempfile")?;

    Ok(tempfile)
}

async fn read_package_name(tempfile: &TempFile) -> Result<String> {
    log::info!("Reading package name...");

    let stdout = bash!("dpkg -I {:?}", tempfile.file_path()).await?;
    let package_name = stdout
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Package: "))
        .next()
        .map(|s| s.to_string())
        .context("failed to find a line starting with 'Package:' in the output of dpkg -I")?;

    log::info!("Package name: {package_name}");

    Ok(package_name)
}

async fn remove_previous_versions(dir: &str, package_name: &str) -> Result<()> {
    let mut dir = tokio::fs::read_dir(&dir)
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
        if !filename.contains(package_name) {
            // other package
            continue;
        }

        log::info!(
            "Removing previous version of {package_name}: {:?}",
            filepath
        );
        tokio::fs::remove_file(&filepath)
            .await
            .with_context(|| format!("failed to remove {filepath:?}"))?;
    }

    Ok(())
}

async fn persist(tempfile: TempFile, dir: &str, filename: &str) -> Result<()> {
    let src = tempfile.file_path().clone();
    let dst = PathBuf::from(dir).join(filename);

    log::info!("Copying {src:?} -> {dst:?}");
    tokio::fs::copy(&src, &dst)
        .await
        .with_context(|| format!("failed to copy {src:?} -> {dst:?}"))?;
    Ok(())
}

async fn reindex(dir: &str) -> Result<()> {
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
    )
    .await?;

    log::info!("Re-indexing has finished!");
    Ok(())
}
