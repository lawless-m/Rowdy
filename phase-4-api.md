# Phase 4: API Layer

## Overview
Expose TTS functionality via HTTP endpoints using axum.

## Tasks

### 4.1 API types (src/api/mod.rs)

```rust
pub mod routes;
pub mod handlers;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SpeakRequest {
    pub text: String,
    pub voice: String,
}

#[derive(Debug, Serialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
}

#[derive(Debug, Serialize)]
pub struct VoicesResponse {
    pub voices: Vec<VoiceInfo>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}
```

### 4.2 Router setup (src/api/routes.rs)

```rust
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;
use std::sync::Arc;

use crate::tts::TtsService;

pub struct AppState {
    pub tts: TtsService,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        .route("/speak", post(handlers::speak))
        .route("/voices", get(handlers::list_voices))
        .route("/health", get(handlers::health));
    
    Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .with_state(state)
}
```

### 4.3 Handlers (src/api/handlers.rs)

```rust
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use super::{AppState, SpeakRequest, VoicesResponse, HealthResponse, ErrorResponse};

pub async fn speak(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SpeakRequest>,
) -> Result<Response, AppError> {
    // Validate input
    if request.text.is_empty() {
        return Err(AppError::BadRequest("Text cannot be empty".into()));
    }
    
    if request.text.len() > 10000 {
        return Err(AppError::BadRequest("Text too long (max 10000 chars)".into()));
    }
    
    // Generate audio
    let wav = state.tts.speak(&request.text, &request.voice)?;
    
    // Return audio response
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "audio/wav")],
        wav,
    ).into_response())
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
```

### 4.4 Error handling

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::VoiceNotFound(v) => (
                StatusCode::NOT_FOUND,
                "VOICE_NOT_FOUND",
                format!("Voice '{}' not found", v),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg.clone(),
            ),
            AppError::TtsError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TTS_ERROR",
                msg.clone(),
            ),
            AppError::OnnxError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ONNX_ERROR",
                e.to_string(),
            ),
            AppError::IoError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "IO_ERROR",
                e.to_string(),
            ),
            AppError::DslError(msg) => (
                StatusCode::BAD_REQUEST,
                "DSL_ERROR",
                msg.clone(),
            ),
        };
        
        tracing::error!("Request failed: {} - {}", code, message);
        
        (
            status,
            Json(ErrorResponse {
                error: message,
                code: code.to_string(),
            }),
        ).into_response()
    }
}
```

### 4.5 CORS configuration

```rust
use tower_http::cors::{CorsLayer, Any};

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);
    
    // ... rest of router setup
    
    Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .layer(cors)
        .with_state(state)
}
```

### 4.6 Main server (src/main.rs)

```rust
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber;

mod api;
mod dsl;
mod error;
mod tts;

use api::{routes::AppState, routes::create_router};
use tts::TtsService;

#[tokio::main]
async fn main() {
    // Initialise logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
        )
        .init();
    
    // Configuration
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    let voices_dir = std::env::var("VOICES_DIR")
        .unwrap_or_else(|_| "./voices".to_string());
    
    // Create TTS service
    let tts = TtsService::new(voices_dir.into());
    
    // Create app state
    let state = Arc::new(AppState { tts });
    
    // Create router
    let app = create_router(state);
    
    // Start server
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 4.7 Request logging middleware

```rust
use axum::middleware;
use tower_http::trace::TraceLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    // ...
    
    Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
        )
        .with_state(state)
}
```

## Acceptance Criteria

- [ ] `POST /api/speak` returns WAV audio
- [ ] `GET /api/voices` returns available voices
- [ ] `GET /api/health` returns status
- [ ] Empty text returns 400
- [ ] Unknown voice returns 404
- [ ] CORS headers present
- [ ] Request logging works
- [ ] Static files served at `/`

## Testing

```bash
# Health check
curl http://localhost:3000/api/health

# List voices
curl http://localhost:3000/api/voices

# Generate speech
curl -X POST http://localhost:3000/api/speak \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world", "voice": "en_GB-alba-medium"}' \
  --output test.wav

# Play it
aplay test.wav
```

## Notes

- Consider rate limiting for production
- Add request size limits
- Could add `/api/speak/stream` later for streaming audio
- Consider async synthesis for long texts
