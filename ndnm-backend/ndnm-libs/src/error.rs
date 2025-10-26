//! Error types for the NDNM system.
//!
//! This module provides a standardized error handling mechanism used across
//! all NDNM services and nodes.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Main error type for the NDNM system.
///
/// This enum covers common error scenarios across all services:
/// - Bad requests (invalid input, validation failures)
/// - Internal errors (processing failures, system errors)
/// - Configuration errors (invalid config files)
/// - IO errors (file system, network)
#[derive(Error, Debug)]
pub enum AppError {
    /// Bad request error - invalid input or validation failure
    ///
    /// # Example
    /// ```
    /// use ndnm_libs::AppError;
    /// let error = AppError::BadRequest("Invalid node configuration".to_string());
    /// ```
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server error - unexpected processing failure
    ///
    /// # Example
    /// ```
    /// use ndnm_libs::AppError;
    /// let error = AppError::Internal("Failed to execute node".to_string());
    /// ```
    #[error("Internal error: {0}")]
    Internal(String),

    /// Configuration error - invalid or missing configuration
    ///
    /// # Example
    /// ```
    /// use ndnm_libs::AppError;
    /// let error = AppError::ConfigError("Missing required field 'node_id_hash'".to_string());
    /// ```
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO error wrapper
    ///
    /// Wraps standard IO errors for consistent error handling
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// YAML parsing error
    ///
    /// Wraps serde_yaml errors for config file parsing
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// JSON serialization/deserialization error
    ///
    /// Wraps serde_json errors
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Implement Axum's IntoResponse for AppError to enable automatic HTTP error responses
///
/// Maps AppError variants to appropriate HTTP status codes:
/// - BadRequest -> 400 Bad Request
/// - ConfigError -> 400 Bad Request
/// - Internal/IoError/YamlError/JsonError -> 500 Internal Server Error
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ConfigError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::IoError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO error: {}", err),
            ),
            AppError::YamlError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("YAML parsing error: {}", err),
            ),
            AppError::JsonError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON error: {}", err),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_request_error() {
        let error = AppError::BadRequest("test error".to_string());
        assert_eq!(error.to_string(), "Bad request: test error");
    }

    #[test]
    fn test_internal_error() {
        let error = AppError::Internal("test error".to_string());
        assert_eq!(error.to_string(), "Internal error: test error");
    }

    #[test]
    fn test_config_error() {
        let error = AppError::ConfigError("test error".to_string());
        assert_eq!(error.to_string(), "Configuration error: test error");
    }
}
