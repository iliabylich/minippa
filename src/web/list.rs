use crate::{index::Package, web::app_error::AppError};
use anyhow::Context as _;
use askama::Template;
use axum::response::Html;

pub(crate) async fn list() -> Result<Html<String>, AppError> {
    let packages = Package::list().await?;
    let html = List { packages }
        .render()
        .context("failed to render template")?;
    Ok(Html(html))
}

#[derive(Template)]
#[template(path = "list.html")]
struct List {
    packages: Vec<Package>,
}
