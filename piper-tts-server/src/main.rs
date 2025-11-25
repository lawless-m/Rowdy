use std::net::SocketAddr;
use std::sync::Arc;

use tracing_subscriber::EnvFilter;

mod api;
mod dsl;
mod error;
mod tts;

use api::routes::{create_router, AppState};
use tts::TtsService;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Configuration from environment
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    let voices_dir = std::env::var("VOICES_DIR").unwrap_or_else(|_| "./voices".to_string());

    // Start server
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address");

    tracing::info!("Piper TTS Server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Starting server on http://{}", addr);
    tracing::info!("Voices directory: {}", voices_dir);

    // Create TTS service
    let tts = TtsService::new(voices_dir.into());

    // Create app state
    let state = Arc::new(AppState { tts });

    // Create router
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
