use std::path::Path;

use crate::KokoroError;

/// Sample rate for Kokoro-82M output audio.
pub const SAMPLE_RATE: u32 = 24000;

/// Save f32 audio samples as a WAV file (24kHz mono).
pub fn save_wav(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), KokoroError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(path, spec).map_err(|e| {
        KokoroError::Audio(format!("Failed to create WAV file {}: {}", path.display(), e))
    })?;

    for &sample in samples {
        writer.write_sample(sample).map_err(|e| {
            KokoroError::Audio(format!("Failed to write WAV sample: {}", e))
        })?;
    }

    writer.finalize().map_err(|e| {
        KokoroError::Audio(format!("Failed to finalize WAV file: {}", e))
    })?;

    Ok(())
}

/// Normalize audio samples so the peak amplitude reaches the target level.
/// This boosts quiet audio without clipping.
pub fn normalize(samples: &mut [f32], target_peak: f32) {
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak > 0.0 && peak < target_peak {
        let gain = target_peak / peak;
        for s in samples.iter_mut() {
            *s *= gain;
        }
    }
}

/// Concatenate multiple audio chunks into a single sample vector.
pub fn concat_samples(chunks: &[Vec<f32>]) -> Vec<f32> {
    let total_len: usize = chunks.iter().map(|c| c.len()).sum();
    let mut result = Vec::with_capacity(total_len);
    for chunk in chunks {
        result.extend_from_slice(chunk);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_samples() {
        let chunks = vec![vec![1.0, 2.0], vec![3.0, 4.0, 5.0]];
        let result = concat_samples(&chunks);
        assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_concat_empty() {
        let chunks: Vec<Vec<f32>> = vec![];
        let result = concat_samples(&chunks);
        assert!(result.is_empty());
    }

    #[test]
    fn test_normalize_boosts_quiet_audio() {
        let mut samples = vec![0.1, -0.2, 0.15, -0.05];
        normalize(&mut samples, 0.95);
        // Peak was 0.2, should now be 0.95
        let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!((peak - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_normalize_no_clipping_loud_audio() {
        let mut samples = vec![0.5, -0.98, 0.3];
        let original = samples.clone();
        normalize(&mut samples, 0.95);
        // Peak 0.98 > 0.95, should not change
        assert_eq!(samples, original);
    }

    #[test]
    fn test_normalize_silence() {
        let mut samples = vec![0.0, 0.0, 0.0];
        normalize(&mut samples, 0.95);
        assert_eq!(samples, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_save_wav() {
        let dir = std::env::temp_dir();
        let path = dir.join("kokoro_test_output.wav");

        // Generate a simple sine wave
        let samples: Vec<f32> = (0..24000)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 24000.0).sin() * 0.5)
            .collect();

        save_wav(&path, &samples, SAMPLE_RATE).unwrap();
        assert!(path.exists());

        // Read it back and verify
        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().sample_rate, 24000);
        assert_eq!(reader.spec().channels, 1);
        assert_eq!(reader.len(), 24000);

        let _ = std::fs::remove_file(&path);
    }
}
