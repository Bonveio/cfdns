//! Error types for the Cloudflare DNS CLI.

#![allow(dead_code)]

use serde::Deserialize;

/// Top-level application error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

/// Represents an error returned by the Cloudflare API.
#[derive(Debug, thiserror::Error)]
#[error("Cloudflare API error {code}: {message}")]
pub struct ApiError {
    pub code: i64,
    pub message: String,
    pub errors: Vec<ApiErrorDetail>,
}

/// Individual error detail from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorDetail {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub documentation_url: Option<String>,
    #[serde(default)]
    pub source: Option<ApiErrorSource>,
}

/// Source pointer for an API error.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorSource {
    pub pointer: String,
}

/// A message from the API response.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiMessage {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub documentation_url: Option<String>,
}

pub type Result<T> = std::result::Result<T, AppError>;
