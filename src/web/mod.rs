mod app_error;
mod app_state;
mod install_sh;
mod list;
mod one;
mod upload;

use crate::{config::Config, index::Index, web::app_state::AppState};
use anyhow::{Context as _, Result};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use install_sh::install_sh;
use list::list;
use one::one;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use upload::upload;

pub(crate) struct Web;

impl Web {
    pub(crate) async fn spawn() -> Result<()> {
        let index = Index::new().await?;

        let app = Router::new()
            .route("/upload", post(upload))
            .layer(DefaultBodyLimit::disable())
            .route("/status", get(status))
            .route("/install.sh", get(install_sh))
            .route("/packages", get(list))
            .route("/packages/{name}", get(one))
            .fallback_service(ServeDir::new(&Config::get().dir))
            .with_state(AppState::new(index));

        let listener = TcpListener::bind(("127.0.0.1", Config::get().port))
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

async fn status() -> &'static str {
    "OK"
}
