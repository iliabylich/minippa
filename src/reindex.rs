use crate::config::Config;
use anyhow::{Context as _, Result, bail};
use tokio::process::Command;

pub(crate) async fn reindex() -> Result<()> {
    log::info!("Reindexing...");

    let script = format!(
        r#"set -euo pipefail
cd "{dir}"
rm -f Packages Packages.gz Release Release.gpg InRelease
dpkg-scanpackages --multiversion . > Packages 2> /dev/null
gzip -k -f Packages
apt-ftparchive release . > Release
gpg --default-key "{email}" -abs -o - Release > Release.gpg
gpg --default-key "{email}" --clearsign -o - Release > InRelease
    "#,
        dir = Config::dir(),
        email = Config::owner_email()
    );

    log::info!("Running '{script}'");

    let output = Command::new("bash")
        .arg("-c")
        .arg(script)
        .output()
        .await
        .context("failed to spawn a script for reindexing")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout = stdout.trim();
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();

        log::error!("\n\nstdout:\n{stdout}\n\nstderr:\n{stderr}");
        bail!("re-indexing failed");
    }

    log::info!("Re-indexing has finished!");
    Ok(())
}
