use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize)]
pub struct VoiceConfig {
    pub audio: AudioConfig,
    pub espeak: Option<EspeakConfig>,
    #[serde(default)]
    pub phoneme_id_map: HashMap<String, Vec<i64>>,
    #[serde(default)]
    pub inference: Option<InferenceConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EspeakConfig {
    pub voice: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InferenceConfig {
    #[serde(default = "default_noise_scale")]
    pub noise_scale: f32,
    #[serde(default = "default_length_scale")]
    pub length_scale: f32,
    #[serde(default = "default_noise_w")]
    pub noise_w: f32,
}

fn default_noise_scale() -> f32 {
    0.667
}

fn default_length_scale() -> f32 {
    1.0
}

fn default_noise_w() -> f32 {
    0.8
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            noise_scale: default_noise_scale(),
            length_scale: default_length_scale(),
            noise_w: default_noise_w(),
        }
    }
}

#[derive(Debug)]
pub struct Voice {
    #[allow(dead_code)]
    pub id: String,
    pub config: VoiceConfig,
    pub model_path: PathBuf,
}

impl Voice {
    pub fn load(voices_dir: &Path, voice_id: &str) -> Result<Self, AppError> {
        let model_path = voices_dir.join(format!("{}.onnx", voice_id));
        let config_path = voices_dir.join(format!("{}.onnx.json", voice_id));

        if !model_path.exists() {
            return Err(AppError::VoiceNotFound(voice_id.to_string()));
        }

        if !config_path.exists() {
            return Err(AppError::VoiceNotFound(format!(
                "{} (missing config file)",
                voice_id
            )));
        }

        let config: VoiceConfig = serde_json::from_reader(File::open(&config_path)?)?;

        Ok(Self {
            id: voice_id.to_string(),
            config,
            model_path,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
}
