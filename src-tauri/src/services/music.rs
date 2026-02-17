use std::path::PathBuf;

use super::process::run_and_stream;

pub struct MusicService {
    scripts_dir: PathBuf,
    models_dir: PathBuf,
}

impl MusicService {
    pub fn new(scripts_dir: PathBuf, models_dir: PathBuf) -> Self {
        Self { scripts_dir, models_dir }
    }

    /// Generate background music using ACE-Step
    pub async fn generate(
        &self,
        genre: &str,
        duration_secs: u32,
        output_path: &str,
    ) -> Result<String, String> {
        let music_dir = self.scripts_dir.join("music");
        let project_root = self.models_dir.join("acestep");
        let (success, output) = run_and_stream(
            tokio::process::Command::new("uv")
                .args([
                    "run",
                    "--project",
                    music_dir.to_str().unwrap(),
                    "python",
                    music_dir
                        .join("generate_music.py")
                        .to_str()
                        .unwrap(),
                    "--genre",
                    genre,
                    "--duration",
                    &duration_secs.to_string(),
                    "--output",
                    output_path,
                    "--project-root",
                    project_root.to_str().unwrap(),
                ]),
            "music",
        )
        .await?;

        if !success {
            return Err(format!("Music generation failed: {}", output));
        }

        Ok(output_path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_service_script_path() {
        let service = MusicService::new(PathBuf::from("/app/scripts"), PathBuf::from("/home/.pattikadhai/models"));
        assert_eq!(
            service
                .scripts_dir
                .join("music")
                .join("generate_music.py")
                .to_str()
                .unwrap(),
            "/app/scripts/music/generate_music.py"
        );
    }
}
