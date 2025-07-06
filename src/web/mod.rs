mod app_state;
mod upload_error;

use crate::{
    config::Config,
    index::{Index, Upload},
    web::{app_state::AppState, upload_error::UploadError},
};
use anyhow::{Context as _, Result, bail};
use axum::{
    Router,
    extract::{Multipart, State},
    http::HeaderMap,
    routing::{get, post},
};
use std::path::Path;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub(crate) struct Web;

impl Web {
    pub(crate) async fn spawn() -> Result<()> {
        let index = Index::new().await?;

        let app = Router::new()
            .route("/upload", post(upload))
            .route("/status", get(status))
            .fallback_service(ServeDir::new(Config::dir()))
            .with_state(AppState::new(index));

        let listener = TcpListener::bind(("127.0.0.1", Config::port()))
            .await
            .context("failed to bind")?;
        log::info!(
            "Listening on {}",
            listener.local_addr().context("failed to get local addr")?
        );

        axum::serve(listener, app)
            .await
            .context("Failed to spawn web server")?;

        Ok(())
    }
}

#[axum::debug_handler]
async fn upload(
    headers: HeaderMap,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<&'static str, UploadError> {
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

    Ok("Package has been succesfully processed")
}

fn auth(headers: HeaderMap) -> Result<()> {
    let Some(token) = headers.get("Token") else {
        bail!("no Token header");
    };
    let Ok(token) = token.to_str() else {
        bail!("non-utf-8 Token header");
    };

    if token == Config::token() {
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

async fn status() -> &'static str {
    "OK"
}
