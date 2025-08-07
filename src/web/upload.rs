use std::path::Path;

use crate::{
    config::Config,
    index::Upload,
    web::{app_error::AppError, app_state::AppState},
};
use anyhow::{Context as _, Result, bail};
use axum::{
    extract::{Multipart, State},
    http::HeaderMap,
};

pub(crate) async fn upload(
    headers: HeaderMap,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<&'static str, AppError> {
    log::info!("{:?}", headers);

    auth(headers)?;

    while let Some(field) = multipart
        .next_field()
        .await
        .context("failed to read multipart field")?
    {
        let filename = field.name().context("empty name of the part")?.to_string();

        ensure_filename_is_just_a_filename(&filename)?;

        let data = field
            .bytes()
            .await
            .with_context(|| format!("failed to read bytes of the part {filename:?}"))?
            .to_vec();

        log::info!("Saving {} ({} bytes)", filename, data.len());
        let upload = Upload::new_in_tmp(data).await?;

        state.index.write(filename, upload).await?;
    }

    Ok("Package has been successfully processed")
}

fn auth(headers: HeaderMap) -> Result<()> {
    let Some(token) = headers.get("Token") else {
        bail!("no Token header");
    };
    let Ok(token) = token.to_str() else {
        bail!("non-utf-8 Token header");
    };

    if token == Config::get().token {
        Ok(())
    } else {
        bail!("invalid token")
    }
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
