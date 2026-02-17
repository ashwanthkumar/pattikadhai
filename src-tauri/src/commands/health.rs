use crate::services::health::{self, DependencyStatus};
use crate::services::process::run_and_stream;

#[tauri::command]
pub async fn check_dependency(name: String, app: tauri::AppHandle) -> Result<DependencyStatus, String> {
    match name.as_str() {
        "ollama" => Ok(health::check_ollama().await),
        "gemma3" => Ok(health::check_gemma3().await),
        "uv" => Ok(health::check_uv()),
        "ffmpeg" => Ok(health::check_ffmpeg()),
        "python_deps" => {
            let scripts_dir = super::resolve_scripts_dir(&app);
            log::info!("Checking python deps at: {}", scripts_dir.display());
            Ok(health::check_python_deps(scripts_dir.to_str().unwrap()).await)
        }
        "tts_model" => {
            let scripts_dir = super::resolve_scripts_dir(&app);
            let models_dir = super::resolve_models_dir();
            Ok(health::check_tts_model(
                scripts_dir.to_str().unwrap(),
                models_dir.to_str().unwrap(),
            ).await)
        }
        "music_model" => {
            let scripts_dir = super::resolve_scripts_dir(&app);
            let models_dir = super::resolve_models_dir();
            Ok(health::check_music_model(
                scripts_dir.to_str().unwrap(),
                models_dir.to_str().unwrap(),
            ).await)
        }
        _ => Err(format!("Unknown dependency: {}", name)),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub output: String,
}

#[tauri::command]
pub async fn install_dependency(name: String, app: tauri::AppHandle) -> Result<InstallResult, String> {
    log::info!("install_dependency called for: {}", name);

    let scripts_dir = super::resolve_scripts_dir(&app);

    let (program, args): (&str, Vec<String>) = match name.as_str() {
        "ollama" => ("brew", vec!["install".into(), "ollama".into()]),
        "gemma3" => ("ollama", vec!["pull".into(), "gemma3".into()]),
        "uv" => ("brew", vec!["install".into(), "uv".into()]),
        "ffmpeg" => ("brew", vec!["install".into(), "ffmpeg".into()]),
        "tts_model" => {
            let models_dir = super::resolve_models_dir();
            let hf_home = models_dir.join("huggingface");
            std::fs::create_dir_all(&hf_home)
                .map_err(|e| format!("Failed to create HF_HOME dir: {}", e))?;

            let tts_dir = scripts_dir.join("tts");
            log::info!("Downloading TTS model to: {}", hf_home.display());

            let (success, output) = run_and_stream(
                tokio::process::Command::new("uv")
                    .args([
                        "run", "--project", tts_dir.to_str().unwrap(),
                        "python",
                        tts_dir.join("download_model.py").to_str().unwrap(),
                    ])
                    .env("HF_HOME", hf_home.to_str().unwrap()),
                "tts_model",
            ).await?;

            return Ok(InstallResult {
                success,
                output: if output.is_empty() { "Completed".to_string() } else { output },
            });
        }
        "music_model" => {
            let models_dir = super::resolve_models_dir();
            let checkpoints_dir = models_dir.join("acestep").join("checkpoints");
            std::fs::create_dir_all(&checkpoints_dir)
                .map_err(|e| format!("Failed to create checkpoints dir: {}", e))?;

            let music_dir = scripts_dir.join("music");
            log::info!("Downloading music model to: {}", checkpoints_dir.display());

            let (success, output) = run_and_stream(
                tokio::process::Command::new("uv")
                    .args([
                        "run", "--project", music_dir.to_str().unwrap(),
                        "python",
                        music_dir.join("download_model.py").to_str().unwrap(),
                        "--checkpoints-dir", checkpoints_dir.to_str().unwrap(),
                    ]),
                "music_model",
            ).await?;

            return Ok(InstallResult {
                success,
                output: if output.is_empty() { "Completed".to_string() } else { output },
            });
        }
        "python_deps" => {
            // Sync both TTS and music venvs separately
            let tts_dir = scripts_dir.join("tts");
            let music_dir = scripts_dir.join("music");

            log::info!("Syncing TTS deps at: {}", tts_dir.display());
            let (tts_ok, tts_log) = run_and_stream(
                tokio::process::Command::new("uv")
                    .args(["sync", "--project", tts_dir.to_str().unwrap()]),
                "python_deps:tts",
            ).await?;

            log::info!("Syncing music deps at: {}", music_dir.display());
            let (music_ok, music_log) = run_and_stream(
                tokio::process::Command::new("uv")
                    .args(["sync", "--project", music_dir.to_str().unwrap()]),
                "python_deps:music",
            ).await?;

            let combined = format!(
                "=== TTS deps ===\n{}\n=== Music deps ===\n{}",
                tts_log, music_log
            ).trim().to_string();
            let success = tts_ok && music_ok;

            log::info!("Install python_deps finished: success={}", success);

            return Ok(InstallResult {
                success,
                output: if combined.is_empty() {
                    "Completed with no output".to_string()
                } else {
                    combined
                },
            });
        }
        _ => return Err(format!("Unknown dependency: {}", name)),
    };

    log::info!("Running: {} {:?}", program, args);

    let (success, output) = run_and_stream(
        tokio::process::Command::new(program).args(&args),
        &name,
    ).await?;

    log::info!("Install '{}' finished: success={}", name, success);

    Ok(InstallResult {
        success,
        output: if output.is_empty() {
            "Completed with no output".to_string()
        } else {
            output
        },
    })
}

