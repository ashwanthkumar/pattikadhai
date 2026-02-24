use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use kokoro_tts::Kokoro;

/// Thread-safe singleton for the Kokoro TTS engine.
/// Wraps in Result so initialization failures can be retried.
static KOKORO: OnceLock<Result<Mutex<Kokoro>, String>> = OnceLock::new();

pub struct TtsRawResult {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_secs: f64,
}

pub struct TtsService {
    models_dir: PathBuf,
}

impl TtsService {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    /// Initialize or retrieve the Kokoro engine singleton.
    fn get_kokoro(&self) -> Result<&'static Mutex<Kokoro>, String> {
        let result = KOKORO.get_or_init(|| {
            let kokoro_dir = self.models_dir.join("kokoro");
            let model_path = kokoro_dir.join("model_quantized.onnx");
            let voices_dir = kokoro_dir.join("voices");

            if !model_path.exists() {
                return Err(format!(
                    "Kokoro model not found at {}. Run setup wizard to download.",
                    model_path.display()
                ));
            }
            if !voices_dir.exists() {
                return Err(format!(
                    "Kokoro voices not found at {}. Run setup wizard to download.",
                    voices_dir.display()
                ));
            }

            log::info!("Loading Kokoro TTS model from {}", kokoro_dir.display());
            match Kokoro::new(&model_path, &voices_dir) {
                Ok(kokoro) => {
                    log::info!("Kokoro TTS model loaded successfully");
                    Ok(Mutex::new(kokoro))
                }
                Err(e) => Err(format!("Failed to load Kokoro TTS: {}", e)),
            }
        });
        result.as_ref().map_err(|e| e.clone())
    }

    /// Generate raw audio samples without writing to disk.
    pub async fn generate_raw(
        &self,
        text: &str,
        voice: Option<&str>,
        speed: Option<f32>,
    ) -> Result<TtsRawResult, String> {
        let voice = voice.unwrap_or("af_nova").to_string();
        let speed = speed.unwrap_or(0.5);
        let text = text.to_string();

        let kokoro_mutex = self.get_kokoro()?;

        tokio::task::spawn_blocking(move || {
            let start = std::time::Instant::now();

            let mut kokoro = kokoro_mutex
                .lock()
                .map_err(|e| format!("Failed to lock Kokoro: {}", e))?;

            let audio = kokoro
                .create(&text, &voice, speed, "en-us")
                .map_err(|e| format!("TTS generation failed: {}", e))?;

            let elapsed = start.elapsed();
            let duration_secs = audio.samples.len() as f64 / audio.sample_rate as f64;
            log::info!(
                "TTS raw generated: {} chars, {:.2}s audio in {:.2}s",
                text.len(),
                duration_secs,
                elapsed.as_secs_f64()
            );

            Ok(TtsRawResult {
                samples: audio.samples,
                sample_rate: audio.sample_rate,
                duration_secs,
            })
        })
        .await
        .map_err(|e| format!("TTS task panicked: {}", e))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tts_service_creation() {
        let service = TtsService::new(PathBuf::from("/home/.pattikadhai/models"));
        assert_eq!(
            service.models_dir.join("kokoro").join("model_quantized.onnx").to_str().unwrap(),
            "/home/.pattikadhai/models/kokoro/model_quantized.onnx"
        );
    }
}
