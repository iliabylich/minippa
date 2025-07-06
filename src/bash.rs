use anyhow::{Context as _, Result, bail};
use tokio::process::Command;

pub(crate) async fn exec(script: &str) -> Result<String> {
    log::info!("Running '{script}'");

    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("set -euo pipefail\n{script}"))
        .output()
        .await
        .with_context(|| format!("failed to spawn bash script '{script}'"))?;

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("non-utf-8 stdout of bash script '{script}'"))?;

    if !output.status.success() {
        log::error!("failed to execute bash script '{script}'");

        log::error!("stdout:");
        log::error!("{stdout}");

        let stderr = String::from_utf8(output.stderr)
            .with_context(|| format!("non-utf-8 stderr of bash script '{script}'"))?;
        log::error!("stderr:");
        log::error!("{stderr}");

        bail!("failed to execute bash script")
    }

    Ok(stdout)
}
