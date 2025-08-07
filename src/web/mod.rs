mod errors;
mod templates;

use crate::index::IndexCtl;
use anyhow::{Context as _, Result};
use askama::Template as _;
use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::HeaderMap,
    response::Html,
    routing::{get, post},
};
use errors::AppError;
use templates::{List, One};
use tokio::{net::TcpListener, task::JoinHandle};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    index_ctl: IndexCtl,
    token: String,
}

pub(crate) async fn spawn(
    port: u16,
    index_ctl: IndexCtl,
    token: &str,
    static_dir: &str,
) -> Result<JoinHandle<()>> {
    let state = AppState {
        index_ctl,
        token: token.to_string(),
    };

    let app = Router::new()
        .route("/upload", post(upload))
        .layer(DefaultBodyLimit::disable())
        .route("/status", get(async || "OK"))
        .route("/install.sh", get(install_sh))
        .route("/packages", get(list))
        .route("/packages/{name}", get(one))
        .fallback_service(ServeDir::new(static_dir))
        .with_state(state);

    const HOST: &str = "127.0.0.1";
    let listener = TcpListener::bind((HOST, port))
        .await
        .context("failed to bind")?;
    log::info!("Listening on http://{HOST}:{port}",);

    let handle = tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            log::error!("failed to spawn web server: {err:?}");
        }
    });

    Ok(handle)
}

async fn upload(
    headers: HeaderMap,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<&'static str, AppError> {
    let given_token = headers
        .get("Token")
        .context("no Token header")?
        .to_str()
        .context("non-utf-8 Token header")?;

    if given_token != state.token {
        return Err(AppError(anyhow::anyhow!("invalid token")));
    }

    while let Some(field) = multipart
        .next_field()
        .await
        .context("failed to read multipart field")?
    {
        let filename = field.name().context("empty name of the part")?.to_string();

        let data = field
            .bytes()
            .await
            .with_context(|| format!("failed to read bytes of the part {filename:?}"))?
            .to_vec();

        log::info!("Saving {} ({} bytes)", filename, data.len());
        state.index_ctl.upload(filename, data).await?;
    }

    Ok("Package has been successfully processed")
}

async fn install_sh(headers: HeaderMap, State(state): State<AppState>) -> Result<String, String> {
    fn bash_err(message: &str) -> String {
        format!("echo \"{message}\"")
    }

    let host = headers
        .get("Host")
        .ok_or_else(|| bash_err("no Host given"))?
        .to_str()
        .map_err(|_| bash_err("non-utf8 Host header"))?;

    const PROTOCOL: &str = if cfg!(debug_assertions) {
        "http"
    } else {
        "https"
    };

    let base_url = format!("{PROTOCOL}://{host}");
    state
        .index_ctl
        .make_install_script(base_url)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            bash_err("internal error")
        })
}

async fn list(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let packages = state.index_ctl.list().await?;
    let html = List { packages }
        .render()
        .context("failed to render template")?;
    Ok(Html(html))
}

async fn one(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let package = state
        .index_ctl
        .find(name)
        .await?
        .context("package not found")?;

    let html = One { package }
        .render()
        .context("failed to render template")?;
    Ok(Html(html))
}
