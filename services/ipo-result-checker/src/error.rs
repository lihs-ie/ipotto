use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ipo_backend_shared::errors::DomainError;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        let status = match error {
            DomainError::HttpClientError { .. } => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            status,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}
