use std::collections::HashMap;
use std::io::Cursor;
use std::process::Command;
use std::sync::Mutex;

use hound::{SampleFormat, WavSpec, WavWriter};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Value;

use crate::error::AppError;
use crate::tts::voice::Voice;

pub struct PiperEngine {
    session: Mutex<Session>,
    noise_scale: f32,
    length_scale: f32,
    noise_w: f32,
}

impl PiperEngine {
    pub fn new(voice: &Voice) -> Result<Self, AppError> {
        // Load the ONNX model using ort (official ONNX Runtime)
        let session = Session::builder()
            .map_err(|e| AppError::TtsError(format!("Failed to create session builder: {}", e)))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| AppError::TtsError(format!("Failed to set optimization level: {}", e)))?
            .with_intra_threads(4)
            .map_err(|e| AppError::TtsError(format!("Failed to set threads: {}", e)))?
            .commit_from_file(&voice.model_path)
            .map_err(|e| AppError::TtsError(format!("Failed to load model: {}", e)))?;

        let inference = voice.config.inference.clone().unwrap_or_default();

        Ok(Self {
            session: Mutex::new(session),
            noise_scale: inference.noise_scale,
            length_scale: inference.length_scale,
            noise_w: inference.noise_w,
        })
    }

    pub fn synthesize(&self, phoneme_ids: &[i64]) -> Result<Vec<f32>, AppError> {
        if phoneme_ids.is_empty() {
            return Ok(Vec::new());
        }

        let input_len = phoneme_ids.len();

        // Prepare input tensors
        // input: [batch, sequence] = [1, phoneme_count]
        let input_value = Value::from_array((vec![1, input_len], phoneme_ids.to_vec()))
            .map_err(|e| AppError::TtsError(format!("Failed to create input tensor: {}", e)))?;

        // input_lengths: [batch] = [1]
        let lengths_value = Value::from_array((vec![1], vec![input_len as i64]))
            .map_err(|e| AppError::TtsError(format!("Failed to create lengths tensor: {}", e)))?;

        // scales: [3] = [noise_scale, length_scale, noise_w]
        let scales_value = Value::from_array((vec![3], vec![
            self.noise_scale,
            self.length_scale,
            self.noise_w,
        ]))
            .map_err(|e| AppError::TtsError(format!("Failed to create scales tensor: {}", e)))?;

        // Run inference
        let mut session = self.session.lock().unwrap();
        let outputs = session
            .run(ort::inputs![input_value, lengths_value, scales_value])
            .map_err(|e| AppError::TtsError(format!("Inference failed: {}", e)))?;

        // Extract audio samples from output
        let output = outputs
            .get("output")
            .or_else(|| outputs.get("audio"))
            .ok_or_else(|| AppError::TtsError("Missing output tensor".to_string()))?;

        let output_view = output
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::TtsError(format!("Failed to extract output tensor: {}", e)))?;

        let audio: Vec<f32> = output_view.1.iter().copied().collect();

        Ok(audio)
    }
}

/// Convert text to phonemes using espeak-ng
pub fn phonemize(text: &str, voice: &str) -> Result<String, AppError> {
    if text.is_empty() {
        return Ok(String::new());
    }

    let output = Command::new("espeak-ng")
        .args(["--ipa", "-q", "-v", voice, text])
        .output()
        .map_err(|e| {
            AppError::TtsError(format!(
                "Failed to run espeak-ng (is it installed?): {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::TtsError(format!("espeak-ng failed: {}", stderr)));
    }

    let phonemes = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(phonemes)
}

/// Convert phonemes to IDs using the voice's phoneme map
pub fn phonemes_to_ids(phonemes: &str, id_map: &HashMap<String, Vec<i64>>) -> Vec<i64> {
    let mut ids = Vec::new();

    // Add BOS (beginning of sequence) - typically 0 or mapped value
    if let Some(bos) = id_map.get("^") {
        ids.extend(bos);
    } else {
        ids.push(0);
    }

    // Process each character/phoneme
    for ch in phonemes.chars() {
        let ch_str = ch.to_string();
        if let Some(mapped) = id_map.get(&ch_str) {
            ids.extend(mapped);
        }
        // Add padding between phonemes if available
        if let Some(pad) = id_map.get("_") {
            ids.extend(pad);
        }
    }

    // Add EOS (end of sequence)
    if let Some(eos) = id_map.get("$") {
        ids.extend(eos);
    } else {
        ids.push(0);
    }

    ids
}

/// Convert audio samples to WAV format
pub fn samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, AppError> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = WavWriter::new(cursor, spec)
            .map_err(|e| AppError::TtsError(format!("Failed to create WAV writer: {}", e)))?;

        for sample in samples {
            // Convert f32 [-1.0, 1.0] to i16 with 2x gain boost
            let scaled = (sample * 2.0 * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer
                .write_sample(scaled)
                .map_err(|e| AppError::TtsError(format!("Failed to write sample: {}", e)))?;
        }

        writer
            .finalize()
            .map_err(|e| AppError::TtsError(format!("Failed to finalize WAV: {}", e)))?;
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phonemes_to_ids_empty() {
        let map = HashMap::new();
        let ids = phonemes_to_ids("", &map);
        // Should have at least BOS and EOS
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_samples_to_wav_empty() {
        let wav = samples_to_wav(&[], 22050).unwrap();
        // Should produce valid WAV header even for empty audio
        assert!(wav.starts_with(b"RIFF"));
    }

    #[test]
    fn test_samples_to_wav_valid() {
        let samples: Vec<f32> = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let wav = samples_to_wav(&samples, 22050).unwrap();
        assert!(wav.starts_with(b"RIFF"));
        assert!(wav.len() > 44); // Header + some data
    }
}
