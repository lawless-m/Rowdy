use std::collections::HashMap;
use std::io::Cursor;
use std::process::Command;

use hound::{SampleFormat, WavSpec, WavWriter};
use tract_onnx::prelude::*;

use crate::error::AppError;
use crate::tts::voice::Voice;

pub struct PiperEngine {
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    noise_scale: f32,
    length_scale: f32,
    noise_w: f32,
}

impl PiperEngine {
    pub fn new(voice: &Voice) -> Result<Self, AppError> {
        // Load the ONNX model using tract
        let model = tract_onnx::onnx()
            .model_for_path(&voice.model_path)
            .map_err(|e| AppError::TtsError(format!("Failed to load model: {}", e)))?
            .into_optimized()
            .map_err(|e| AppError::TtsError(format!("Failed to optimize model: {}", e)))?
            .into_runnable()
            .map_err(|e| AppError::TtsError(format!("Failed to make model runnable: {}", e)))?;

        let inference = voice.config.inference.clone().unwrap_or_default();

        Ok(Self {
            model,
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
        let input_tensor: Tensor = tract_ndarray::Array2::from_shape_vec(
            (1, input_len),
            phoneme_ids.to_vec(),
        )
        .map_err(|e| AppError::TtsError(format!("Failed to create input tensor: {}", e)))?
        .into();

        // input_lengths: [batch] = [1]
        let lengths_tensor: Tensor =
            tract_ndarray::Array1::from_vec(vec![input_len as i64]).into();

        // scales: [3] = [noise_scale, length_scale, noise_w]
        let scales_tensor: Tensor = tract_ndarray::Array1::from_vec(vec![
            self.noise_scale,
            self.length_scale,
            self.noise_w,
        ])
        .into();

        // Run inference
        let outputs = self
            .model
            .run(tvec![
                input_tensor.into(),
                lengths_tensor.into(),
                scales_tensor.into(),
            ])
            .map_err(|e| AppError::TtsError(format!("Inference failed: {}", e)))?;

        // Extract audio samples from output
        // Output shape is typically [1, 1, samples]
        let output = outputs
            .first()
            .ok_or_else(|| AppError::TtsError("Missing output tensor".to_string()))?;

        let output_array = output
            .to_array_view::<f32>()
            .map_err(|e| AppError::TtsError(format!("Failed to extract output: {}", e)))?;

        let audio: Vec<f32> = output_array.iter().copied().collect();

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
            // Convert f32 [-1.0, 1.0] to i16
            let scaled = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
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
