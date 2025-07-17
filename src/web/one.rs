use crate::{index::Package, web::app_error::AppError};
use anyhow::Context as _;
use askama::Template;
use axum::{extract::Path, response::Html};

pub(crate) async fn one(Path(name): Path<String>) -> Result<Html<String>, AppError> {
    let package = Package::one(name).await?;
    let html = One { package }
        .render()
        .context("failed to render template")?;
    Ok(Html(html))
}

#[derive(Template)]
#[template(path = "one.html")]
struct One {
    package: Package,
}
