use crate::db::queries;
use crate::services::health::{self, DependencyStatus};
use crate::services::process::run_and_stream;
use tauri::Manager;

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
        "python_deps" => {
            let tts_dir = scripts_dir.join("tts");

            log::info!("Syncing TTS deps at: {}", tts_dir.display());
            let (success, output) = run_and_stream(
                tokio::process::Command::new("uv")
                    .args(["sync", "--project", tts_dir.to_str().unwrap()]),
                "python_deps:tts",
            ).await?;

            log::info!("Install python_deps finished: success={}", success);

            return Ok(InstallResult {
                success,
                output: if output.is_empty() {
                    "Completed with no output".to_string()
                } else {
                    output
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

#[tauri::command]
pub async fn apply_migrations(app: tauri::AppHandle) -> Result<String, String> {
    let db_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("pattikadhai.db");

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open DB: {}", e))?;

    queries::apply_rusqlite_migrations(&conn)
        .map_err(|e| format!("Migration failed: {}", e))?;

    Ok("Migrations applied successfully".to_string())
}
