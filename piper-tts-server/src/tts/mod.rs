pub mod piper;
pub mod voice;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::dsl;
use crate::error::AppError;

pub use piper::PiperEngine;
pub use voice::{Voice, VoiceInfo};

pub struct TtsService {
    voices_dir: PathBuf,
    engines: RwLock<HashMap<String, Arc<PiperEngine>>>,
}

impl TtsService {
    pub fn new(voices_dir: PathBuf) -> Self {
        Self {
            voices_dir,
            engines: RwLock::new(HashMap::new()),
        }
    }

    pub fn speak(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, AppError> {
        // 1. Get or load engine
        let engine = self.get_engine(voice_id)?;

        // 2. Process DSL
        let processed = dsl::process(text);

        // 3. Phonemize
        let voice = Voice::load(&self.voices_dir, voice_id)?;
        let espeak_voice = voice
            .config
            .espeak
            .as_ref()
            .map(|e| e.voice.as_str())
            .unwrap_or("en");
        let phonemes = piper::phonemize(&processed, espeak_voice)?;

        // 4. Convert to IDs
        let ids = piper::phonemes_to_ids(&phonemes, &voice.config.phoneme_id_map);

        // 5. Synthesize
        let samples = engine.synthesize(&ids)?;

        // 6. Encode WAV
        let wav = piper::samples_to_wav(&samples, voice.config.audio.sample_rate)?;

        Ok(wav)
    }

    fn get_engine(&self, voice_id: &str) -> Result<Arc<PiperEngine>, AppError> {
        // Check cache
        {
            let engines = self.engines.read().unwrap();
            if let Some(engine) = engines.get(voice_id) {
                return Ok(Arc::clone(engine));
            }
        }

        // Load new engine
        let voice = Voice::load(&self.voices_dir, voice_id)?;
        let engine = Arc::new(PiperEngine::new(&voice)?);

        // Cache it
        {
            let mut engines = self.engines.write().unwrap();
            engines.insert(voice_id.to_string(), Arc::clone(&engine));
        }

        Ok(engine)
    }

    pub fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError> {
        let mut voices = Vec::new();

        if !self.voices_dir.exists() {
            return Ok(voices);
        }

        for entry in std::fs::read_dir(&self.voices_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "onnx").unwrap_or(false) {
                let id = path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                if let Ok(voice) = Voice::load(&self.voices_dir, &id) {
                    let language = voice
                        .config
                        .espeak
                        .as_ref()
                        .map(|e| e.voice.clone())
                        .unwrap_or_else(|| "en".to_string());

                    // Parse voice name from ID (e.g., en_GB-alba-medium -> Alba)
                    let name = parse_voice_name(&id);

                    voices.push(VoiceInfo {
                        id,
                        name,
                        language,
                    });
                }
            }
        }

        Ok(voices)
    }
}

fn parse_voice_name(id: &str) -> String {
    // Pattern: language-name-quality (e.g., en_GB-alba-medium)
    let parts: Vec<&str> = id.split('-').collect();
    if parts.len() >= 2 {
        // Capitalize the voice name
        let name = parts[1];
        let mut chars = name.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => id.to_string(),
        }
    } else {
        id.to_string()
    }
}
