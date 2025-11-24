# Phase 3: Piper ONNX Integration

## Overview
Integrate Piper TTS using the ONNX runtime for native Rust inference.

## Background

Piper models consist of:
- `.onnx` file — the neural network model
- `.onnx.json` file — configuration (sample rate, phoneme mapping, etc.)

The inference pipeline:
1. Text → Phonemes (using espeak-ng phonemizer or built-in)
2. Phonemes → IDs (lookup table from config)
3. IDs → ONNX model → Audio samples
4. Audio samples → WAV file

## Tasks

### 3.1 Voice configuration (src/tts/voice.rs)

```rust
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct VoiceConfig {
    pub audio: AudioConfig,
    pub espeak: Option<EspeakConfig>,
    pub phoneme_id_map: HashMap<String, Vec<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    // Usually 22050
}

#[derive(Debug, Deserialize)]
pub struct EspeakConfig {
    pub voice: String,
    // e.g., "en-gb"
}

#[derive(Debug)]
pub struct Voice {
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
        
        let config: VoiceConfig = serde_json::from_reader(
            File::open(&config_path)?
        )?;
        
        Ok(Self {
            id: voice_id.to_string(),
            config,
            model_path,
        })
    }
}
```

### 3.2 Phonemizer

Option A: Shell out to espeak-ng (simpler)
```rust
use std::process::Command;

pub fn phonemize(text: &str, voice: &str) -> Result<String, AppError> {
    let output = Command::new("espeak-ng")
        .args(["--ipa", "-q", "-v", voice, text])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

Option B: Use espeak-ng-sys crate (no subprocess)
```toml
espeak-ng-sys = "0.4"
```

**Recommendation:** Start with subprocess, optimise later if needed.

### 3.3 Phoneme to ID mapping

```rust
pub fn phonemes_to_ids(
    phonemes: &str, 
    id_map: &HashMap<String, Vec<i64>>
) -> Vec<i64> {
    let mut ids = Vec::new();
    
    // Add start token
    ids.push(0);
    
    for phoneme in phonemes.chars() {
        if let Some(mapped) = id_map.get(&phoneme.to_string()) {
            ids.extend(mapped);
        }
    }
    
    // Add end token
    ids.push(0);
    
    ids
}
```

### 3.4 ONNX inference (src/tts/piper.rs)

```rust
use ort::{Session, SessionBuilder, Value};
use std::sync::Arc;

pub struct PiperEngine {
    session: Session,
    config: VoiceConfig,
}

impl PiperEngine {
    pub fn new(voice: &Voice) -> Result<Self, AppError> {
        let session = SessionBuilder::new()?
            .with_model_from_file(&voice.model_path)?;
        
        Ok(Self {
            session,
            config: voice.config.clone(),
        })
    }
    
    pub fn synthesize(&self, phoneme_ids: Vec<i64>) -> Result<Vec<f32>, AppError> {
        // Prepare input tensors
        let input_ids = Value::from_array((
            vec![1, phoneme_ids.len()],
            phoneme_ids.clone(),
        ))?;
        
        let input_lengths = Value::from_array((
            vec![1],
            vec![phoneme_ids.len() as i64],
        ))?;
        
        let scales = Value::from_array((
            vec![3],
            vec![0.667f32, 1.0f32, 0.8f32],  // noise, length, noise_w
        ))?;
        
        // Run inference
        let outputs = self.session.run(vec![
            ("input", input_ids),
            ("input_lengths", input_lengths),
            ("scales", scales),
        ])?;
        
        // Extract audio samples
        let audio = outputs["output"]
            .try_extract::<f32>()?
            .view()
            .to_owned()
            .into_raw_vec();
        
        Ok(audio)
    }
}
```

### 3.5 WAV encoding

```rust
use hound::{WavSpec, WavWriter};

pub fn samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, AppError> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut buffer = Vec::new();
    {
        let mut writer = WavWriter::new(
            std::io::Cursor::new(&mut buffer),
            spec,
        )?;
        
        for sample in samples {
            // Convert f32 [-1.0, 1.0] to i16
            let scaled = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(scaled)?;
        }
        
        writer.finalize()?;
    }
    
    Ok(buffer)
}
```

### 3.6 High-level TTS interface

```rust
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
        let phonemes = phonemize(&processed, &voice.config.espeak.voice)?;
        
        // 4. Convert to IDs
        let ids = phonemes_to_ids(&phonemes, &voice.config.phoneme_id_map);
        
        // 5. Synthesize
        let samples = engine.synthesize(ids)?;
        
        // 6. Encode WAV
        let wav = samples_to_wav(&samples, voice.config.audio.sample_rate)?;
        
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
        
        for entry in std::fs::read_dir(&self.voices_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "onnx").unwrap_or(false) {
                let id = path.file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                
                if let Ok(voice) = Voice::load(&self.voices_dir, &id) {
                    voices.push(VoiceInfo {
                        id,
                        language: voice.config.espeak.voice.clone(),
                    });
                }
            }
        }
        
        Ok(voices)
    }
}
```

## Dependencies

System dependencies (must be installed):
```bash
# Ubuntu/Debian
sudo apt install espeak-ng libespeak-ng-dev

# Or build espeak-ng from source for latest features
```

## Acceptance Criteria

- [ ] Voice configs load correctly
- [ ] Phonemization produces IPA output
- [ ] ONNX model loads and runs
- [ ] Audio samples generated
- [ ] Valid WAV output produced
- [ ] Multiple voices can be loaded
- [ ] Engine caching works

## Testing

Download a test voice:
```bash
mkdir -p voices
cd voices
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx.json
```

Test synthesis:
```rust
#[test]
fn test_basic_synthesis() {
    let service = TtsService::new(PathBuf::from("./voices"));
    let wav = service.speak("Hello world", "en_GB-alba-medium").unwrap();
    
    assert!(!wav.is_empty());
    assert_eq!(&wav[0..4], b"RIFF");  // WAV header
}
```

## Notes

- espeak-ng subprocess is fine for now — optimise if latency matters
- Consider CUDA/GPU acceleration later via `ort` features
- Watch memory usage with multiple loaded models
- Piper model format may vary — test with multiple voices
