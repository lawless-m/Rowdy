pub mod handlers;
pub mod routes;

use serde::{Deserialize, Serialize};

use crate::tts::VoiceInfo;

#[derive(Debug, Deserialize)]
pub struct SpeakRequest {
    pub text: String,
    pub voice: String,
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
