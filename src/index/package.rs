use crate::config::Config;
use anyhow::{Context as _, Result};
use chrono::Utc;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) uploaded_at: String,

    pub(crate) description: String,
}

impl Package {
    async fn parse(text: String) -> Result<Self> {
        let name = line_with_prefix(&text, "Package: ")?;
        let version = line_with_prefix(&text, "Version: ")?;
        let filename = line_with_prefix(&text, "Filename: ")?;
        Ok(Self {
            name,
            version,
            uploaded_at: uploaded_at(filename).await?,
            description: text,
        })
    }
    pub(crate) async fn list() -> Result<Vec<Self>> {
        packages()
            .await?
            .split("\n\n")
            .map(|part| tokio::task::spawn(Self::parse(part.to_string())))
            .collect::<futures::future::JoinAll<_>>()
            .await
            .into_iter()
            .map(|package_or_err| package_or_err?)
            .collect()
    }

    pub(crate) async fn one(name: String) -> Result<Self> {
        let part = packages()
            .await?
            .split("\n\n")
            .find(|part| line_with_prefix(part, "Package: ").is_ok_and(|s| s == name))
            .with_context(|| format!("package {name} does not exist"))?
            .to_string();
        Self::parse(part).await
    }
}

async fn packages() -> Result<String> {
    let path = PathBuf::from(Config::dir()).join("Packages");
    let contents = tokio::fs::read_to_string(path)
        .await
        .context("failed to read Packages file")?;
    Ok(contents.trim().to_string())
}

fn line_with_prefix(text: &str, prefix: &str) -> Result<String> {
    let line = text
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .next()
        .with_context(|| format!("no line with {prefix:?} prefix"))?;
    Ok(line.to_string())
}

async fn uploaded_at(filename: String) -> Result<String> {
    let path = PathBuf::from(Config::dir()).join(filename);
    let metadata = tokio::fs::metadata(&path)
        .await
        .with_context(|| format!("failed to get metadata of {path:?}"))?;

    let mtime = metadata
        .modified()
        .with_context(|| format!("failed to get mtime of {path:?}"))?;

    let uploaded_at = chrono::DateTime::<Utc>::from(mtime)
        .format("%v %T")
        .to_string();

    Ok(uploaded_at)
}
