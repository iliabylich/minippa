use anyhow::{Context as _, Result};
use std::path::Path;

pub(crate) async fn write_gpg_key(dir: &str, key: String) -> Result<()> {
    let dst = Path::new(dir).join("public.gpg");
    tokio::fs::write(dst, key)
        .await
        .context("failed to write public.gpg")
}
