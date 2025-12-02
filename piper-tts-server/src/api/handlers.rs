use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rodio::Source;
use std::io::Cursor;
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

pub async fn speak_aloud(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SpeakRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
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

    // Play audio in a background task
    tokio::task::spawn_blocking(move || {
        if let Err(e) = play_audio(wav) {
            tracing::error!("Failed to play audio: {}", e);
        }
    });

    Ok(Json(serde_json::json!({
        "status": "playing",
        "text": request.text
    })))
}

pub async fn list_voices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<VoicesResponse>, AppError> {
    let voices = state.tts.list_voices()?;
    Ok(Json(VoicesResponse { voices }))
}

fn play_audio(wav_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let cursor = Cursor::new(wav_data.clone());
    let source = rodio::Decoder::new(cursor)?;
    stream_handle.play_raw(source.convert_samples())?;

    // Sleep to allow audio to finish playing
    std::thread::sleep(std::time::Duration::from_secs(
        (wav_data.len() / 44100 / 2) as u64 + 1
    ));

    Ok(())
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
