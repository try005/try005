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
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message, error_type) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, "NotFound"),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, "BadRequest"),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg, "Validation"),
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, "Configuration"),
            AppError::Network(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg, "Network"),
            AppError::Timeout(msg) => (StatusCode::REQUEST_TIMEOUT, msg, "Timeout"),
            AppError::Kube(err) => {
                // Handle specific Kubernetes errors more gracefully
                let (status, msg) = match &err {
                    kube::Error::Api(api_err) => match api_err.code {
                        404 => (StatusCode::NOT_FOUND, format!("Resource not found: {}", api_err.message)),
                        400 => (StatusCode::BAD_REQUEST, format!("Invalid request: {}", api_err.message)),
                        401 => (StatusCode::UNAUTHORIZED, format!("Unauthorized: {}", api_err.message)),
                        403 => (StatusCode::FORBIDDEN, format!("Forbidden: {}", api_err.message)),
                        409 => (StatusCode::CONFLICT, format!("Conflict: {}", api_err.message)),
                        _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Kubernetes error: {}", api_err.message)),
                    },
                    kube::Error::Auth(auth_err) => (StatusCode::UNAUTHORIZED, format!("Authentication error: {}", auth_err)),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Kubernetes error: {}", err)),
                };
                (status, msg, "Kubernetes")
            },
            AppError::Serde(err) => (StatusCode::BAD_REQUEST, format!("Serialization error: {}", err), "Serialization"),
            AppError::Json(err) => (StatusCode::BAD_REQUEST, format!("JSON error: {}", err), "JSON"),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, "Internal"),
        };
        
        // Log the error for debugging
        tracing::error!(
            error_type = error_type,
            error_message = error_message,
            status_code = status.as_u16(),
            "Request failed"
        );
        
        let body = ResponseJson(json!({
            "error": {
                "type": error_type,
                "message": error_message,
                "status": status.as_u16()
            }
        }));
        
        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;