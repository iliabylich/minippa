use anyhow::{Context as _, Result, bail};
use tokio::process::Command;

pub(crate) async fn exec(script: String) -> Result<String> {
    log::info!("Running:\n{script}");

    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("set -euo pipefail\n{script}"))
        .output()
        .await
        .with_context(|| format!("failed to spawn bash script '{script}'"))?;

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("non-utf-8 stdout of bash script '{script}'"))?;
    log::info!("stdout:");
    log::info!("{stdout}");

    if !output.status.success() {
        log::error!("failed to execute bash script '{script}'");

        let stderr = String::from_utf8(output.stderr)
            .with_context(|| format!("non-utf-8 stderr of bash script '{script}'"))?;
        log::error!("stderr:");
        log::error!("{stderr}");

        bail!("failed to execute bash script")
    }

    Ok(stdout)
}

macro_rules! bash {
    ($($arg:tt)*) => {{
        let script = format!($($arg)*);
        $crate::bash::exec(script)
    }};
}
pub(crate) use bash;
