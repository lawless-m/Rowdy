use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Voice not found: {0}")]
    VoiceNotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("TTS generation failed: {0}")]
    TtsError(String),

    #[error("Invalid DSL syntax: {0}")]
    #[allow(dead_code)]
    DslError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::VoiceNotFound(v) => (
                StatusCode::NOT_FOUND,
                "VOICE_NOT_FOUND",
                format!("Voice '{}' not found", v),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::TtsError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TTS_ERROR",
                msg.clone(),
            ),
            AppError::IoError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "IO_ERROR",
                e.to_string(),
            ),
            AppError::DslError(msg) => (StatusCode::BAD_REQUEST, "DSL_ERROR", msg.clone()),
            AppError::JsonError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "JSON_ERROR",
                e.to_string(),
            ),
        };

        tracing::error!("Request failed: {} - {}", code, message);

        (
            status,
            Json(ErrorResponse {
                error: message,
                code: code.to_string(),
            }),
        )
            .into_response()
    }
}
