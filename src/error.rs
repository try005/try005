use axum::{http::StatusCode, response::Json as ResponseJson};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_yaml::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Kube(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::Serde(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            AppError::Json(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        let body = ResponseJson(json!({
            "error": error_message
        }));
        
        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;