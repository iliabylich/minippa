use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub(crate) struct UploadError(anyhow::Error);

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "app error: {}", self.0)
    }
}

impl IntoResponse for UploadError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self:?}")).into_response()
    }
}

impl From<anyhow::Error> for UploadError {
    fn from(error: anyhow::Error) -> Self {
        UploadError(error)
    }
}
