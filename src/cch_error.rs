use axum::{http::StatusCode, response::IntoResponse};
use color_eyre::eyre::Report;

pub struct ReportError(pub Report);

impl From<Report> for ReportError {
    fn from(err: Report) -> Self {
        ReportError(err)
    }
}

impl IntoResponse for ReportError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error: {:?}", self.0),
        )
            .into_response()
    }
}
