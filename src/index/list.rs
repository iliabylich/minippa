use crate::index::Package;
use anyhow::{Context as _, Result};
use chrono::Utc;
use futures::future::TryJoinAll;
use std::{collections::HashSet, path::PathBuf};

pub(crate) async fn list(dir: &str, only: Option<HashSet<String>>) -> Result<Vec<Package>> {
    let contents = read_packages_file(dir).await?;
    let packages = parse_many(contents)?;

    let matches = |name: &str| -> bool {
        if let Some(only) = only.as_ref() {
            only.contains(name)
        } else {
            true
        }
    };

    packages
        .into_iter()
        .filter(|info| matches(&info.name))
        .map(|info| package_with_mtime(dir, info))
        .collect::<TryJoinAll<_>>()
        .await
}

async fn read_packages_file(dir: &str) -> Result<String> {
    let path = PathBuf::from(dir).join("Packages");
    let contents = tokio::fs::read_to_string(path)
        .await
        .context("failed to read Packages file")?;
    Ok(contents.trim().to_string())
}

struct Info {
    name: String,
    version: String,
    filename: String,
    full: String,
}

fn parse_many(contents: String) -> Result<Vec<Info>> {
    contents.split("\n\n").map(parse_one).collect()
}

fn parse_one(text: &str) -> Result<Info> {
    let name = line_with_prefix(text, "Package: ")?;
    let version = line_with_prefix(text, "Version: ")?;
    let filename = line_with_prefix(text, "Filename: ")?;
    Ok(Info {
        name,
        version,
        filename,
        full: text.to_string(),
    })
}

fn line_with_prefix(text: &str, prefix: &str) -> Result<String> {
    let line = text
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .next()
        .with_context(|| format!("no line with {prefix:?} prefix"))?;
    Ok(line.to_string())
}

async fn package_with_mtime(dir: &str, info: Info) -> Result<Package> {
    let path = PathBuf::from(dir).join(&info.filename);
    let metadata = tokio::fs::metadata(&path)
        .await
        .with_context(|| format!("failed to get metadata of {path:?}"))?;

    let mtime = metadata
        .modified()
        .with_context(|| format!("failed to get mtime of {path:?}"))?;

    let uploaded_at = chrono::DateTime::<Utc>::from(mtime)
        .format("%v %T")
        .to_string();

    Ok(Package {
        name: info.name,
        version: info.version,
        uploaded_at,
        full: info.full,
    })
}
