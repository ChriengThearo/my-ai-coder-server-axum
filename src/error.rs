use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Custom error type for the application
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("LLM API error: {0}")]
    LlmApiError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Request timeout: {0}")]
    TimeoutError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::RateLimitError(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            AppError::TimeoutError(msg) => (StatusCode::GATEWAY_TIMEOUT, msg),
            AppError::ConnectionError(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::LlmApiError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "detail": error_message,
        }));

        (status, body).into_response()
    }
}

/// Convert reqwest errors to AppError
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::TimeoutError(err.to_string())
        } else if err.is_connect() {
            AppError::ConnectionError(err.to_string())
        } else if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
            AppError::RateLimitError(err.to_string())
        } else {
            AppError::LlmApiError(err.to_string())
        }
    }
}
