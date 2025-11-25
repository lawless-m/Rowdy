use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use super::{HealthResponse, SpeakRequest, VoicesResponse};
use crate::api::routes::AppState;
use crate::error::AppError;

pub async fn speak(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SpeakRequest>,
) -> Result<Response, AppError> {
    // Validate input
    if request.text.is_empty() {
        return Err(AppError::BadRequest("Text cannot be empty".into()));
    }

    if request.text.len() > 10000 {
        return Err(AppError::BadRequest(
            "Text too long (max 10000 chars)".into(),
        ));
    }

    if request.voice.is_empty() {
        return Err(AppError::BadRequest("Voice cannot be empty".into()));
    }

    // Generate audio
    let wav = state.tts.speak(&request.text, &request.voice)?;

    // Return audio response
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "audio/wav")], wav).into_response())
}

pub async fn list_voices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<VoicesResponse>, AppError> {
    let voices = state.tts.list_voices()?;
    Ok(Json(VoicesResponse { voices }))
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
