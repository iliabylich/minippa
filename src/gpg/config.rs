use crate::config::Config;
use anyhow::{Context as _, Result};
use async_tempfile::TempFile;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt as _;

pub(crate) struct GpgConfig {
    f: TempFile,
}

impl GpgConfig {
    pub(crate) async fn new() -> Result<Self> {
        let mut f = TempFile::new()
            .await
            .context("failed to create config tempfile")?;

        let contents = format!(
            r#"%no-protection
Key-Type: RSA
Key-Length: 4096
Key-Usage: cert,sign
Subkey-Type: RSA
Subkey-Length: 4096
Subkey-Usage: encrypt
Name-Real: {}
Name-Email: {}
Expire-Date: 0
%commit
"#,
            Config::name(),
            Config::email()
        );

        f.write_all(contents.as_bytes())
            .await
            .context("failed to write GPG config to tempfile")?;

        Ok(Self { f })
    }

    pub(crate) fn path(&self) -> &PathBuf {
        self.f.file_path()
    }
}
