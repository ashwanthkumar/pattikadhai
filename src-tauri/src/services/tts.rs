use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use kokoro_tts::Kokoro;

/// Thread-safe singleton for the Kokoro TTS engine.
/// Wraps in Result so initialization failures can be retried.
static KOKORO: OnceLock<Result<Mutex<Kokoro>, String>> = OnceLock::new();

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

    /// Generate speech audio from text using Kokoro-82M.
    pub async fn generate(
        &self,
        text: &str,
        output_path: &str,
        voice: Option<&str>,
        speed: Option<f32>,
    ) -> Result<String, String> {
        let voice = voice.unwrap_or("af_nova");
        let speed = speed.unwrap_or(0.5);
        let text = text.to_string();
        let voice = voice.to_string();
        let output_path = output_path.to_string();

        let kokoro_mutex = self.get_kokoro()?;

        // ONNX inference is synchronous — run on a blocking thread
        tokio::task::spawn_blocking(move || {
            let input_chars = text.len();
            let start = std::time::Instant::now();

            let mut kokoro = kokoro_mutex
                .lock()
                .map_err(|e| format!("Failed to lock Kokoro: {}", e))?;

            let audio = kokoro
                .create(&text, &voice, speed, "en-us")
                .map_err(|e| format!("TTS generation failed: {}", e))?;

            let elapsed = start.elapsed();
            log::info!(
                "TTS generated: {} input chars in {:.2}s",
                input_chars,
                elapsed.as_secs_f64()
            );

            let path = std::path::Path::new(&output_path);
            audio
                .save_wav(path)
                .map_err(|e| format!("Failed to save WAV: {}", e))?;

            Ok(output_path)
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
