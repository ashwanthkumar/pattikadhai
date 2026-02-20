use std::path::PathBuf;
use std::sync::OnceLock;

use super::process::run_and_stream;

/// Find the espeak-ng shared library path for the phonemizer backend.
/// Caches the result for the lifetime of the process.
pub fn espeak_library_path() -> Option<&'static str> {
    static ESPEAK_LIB: OnceLock<Option<String>> = OnceLock::new();
    ESPEAK_LIB
        .get_or_init(|| {
            // Check common Homebrew locations
            let candidates = [
                "/opt/homebrew/lib/libespeak-ng.dylib",
                "/usr/local/lib/libespeak-ng.dylib",
            ];
            for path in &candidates {
                if std::path::Path::new(path).exists() {
                    return Some(path.to_string());
                }
            }
            // Fallback: ask brew
            if let Ok(output) = std::process::Command::new("brew")
                .args(["--prefix", "espeak-ng"])
                .output()
            {
                if output.status.success() {
                    let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let lib_path = format!("{}/lib/libespeak-ng.dylib", prefix);
                    if std::path::Path::new(&lib_path).exists() {
                        return Some(lib_path);
                    }
                }
            }
            None
        })
        .as_deref()
}

pub struct TtsService {
    scripts_dir: PathBuf,
    models_dir: PathBuf,
}

impl TtsService {
    pub fn new(scripts_dir: PathBuf, models_dir: PathBuf) -> Self {
        Self { scripts_dir, models_dir }
    }

    /// Generate speech audio from text using KittenTTS
    pub async fn generate(
        &self,
        text: &str,
        output_path: &str,
        voice: Option<&str>,
    ) -> Result<String, String> {
        let voice = voice.unwrap_or("Luna");

        let tts_dir = self.scripts_dir.join("tts");
        let hf_home = self.models_dir.join("huggingface");
        let mut cmd = tokio::process::Command::new("uv");
        cmd.args([
                "run",
                "--project",
                tts_dir.to_str().unwrap(),
                "python",
                tts_dir.join("tts_generate.py").to_str().unwrap(),
                "--text",
                text,
                "--output",
                output_path,
                "--voice",
                voice,
            ])
            .env("HF_HOME", hf_home.to_str().unwrap());
        if let Some(lib) = espeak_library_path() {
            cmd.env("PHONEMIZER_ESPEAK_LIBRARY", lib);
        }
        let (success, output) = run_and_stream(
            &mut cmd,
            "tts",
        )
        .await?;

        if !success {
            return Err(format!("TTS generation failed: {}", output));
        }

        Ok(output_path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tts_service_script_path() {
        let service = TtsService::new(PathBuf::from("/app/scripts"), PathBuf::from("/home/.pattikadhai/models"));
        assert_eq!(
            service.scripts_dir.join("tts").join("tts_generate.py").to_str().unwrap(),
            "/app/scripts/tts/tts_generate.py"
        );
    }
}
