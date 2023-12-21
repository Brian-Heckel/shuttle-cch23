use axum::{http::StatusCode, response::IntoResponse};
use color_eyre::eyre::Report;

pub struct ReportError(pub Report);

impl<E> From<E> for ReportError
where
    E: Into<Report>,
{
    fn from(err: E) -> Self {
        ReportError(err.into())
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
