use std::path::PathBuf;

use super::process::run_and_stream;

pub struct TtsService {
    scripts_dir: PathBuf,
    models_dir: PathBuf,
}

impl TtsService {
    pub fn new(scripts_dir: PathBuf, models_dir: PathBuf) -> Self {
        Self { scripts_dir, models_dir }
    }

    /// Generate speech audio from text using Qwen3-TTS via mlx-audio
    pub async fn generate(
        &self,
        text: &str,
        output_path: &str,
        voice: Option<&str>,
        seed: Option<u64>,
        temperature: Option<f64>,
    ) -> Result<String, String> {
        let voice = voice.unwrap_or("Vivian");
        let seed_str = seed.unwrap_or(42).to_string();
        let temp_str = temperature.unwrap_or(0.3).to_string();

        let tts_dir = self.scripts_dir.join("tts");
        let hf_home = self.models_dir.join("huggingface");
        let (success, output) = run_and_stream(
            tokio::process::Command::new("uv")
                .args([
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
                    "--seed",
                    &seed_str,
                    "--temperature",
                    &temp_str,
                ])
                .env("HF_HOME", hf_home.to_str().unwrap()),
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
