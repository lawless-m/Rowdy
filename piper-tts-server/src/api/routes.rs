use axum::{
    http::{header, Method},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use super::handlers;
use crate::tts::TtsService;

pub struct AppState {
    pub tts: TtsService,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let api_routes = Router::new()
        .route("/speak", post(handlers::speak))
        .route("/voices", get(handlers::list_voices))
        .route("/health", get(handlers::health));

    Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static").append_index_html_on_directories(true))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
