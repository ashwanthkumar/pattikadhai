use crate::db::queries;
use crate::services::health::{self, DependencyStatus};
use crate::services::process::run_and_stream;
use tauri::Manager;

#[tauri::command]
pub async fn check_dependency(name: String, _app: tauri::AppHandle) -> Result<DependencyStatus, String> {
    match name.as_str() {
        "ollama" => Ok(health::check_ollama().await),
        "gemma3:4b" => Ok(health::check_gemma3().await),
        "ffmpeg" => Ok(health::check_ffmpeg()),
        "espeak_ng" => Ok(health::check_espeak_ng()),
        "tts_model" => {
            let models_dir = super::resolve_models_dir();
            Ok(health::check_tts_model(models_dir.to_str().unwrap()))
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
pub async fn install_dependency(name: String, _app: tauri::AppHandle) -> Result<InstallResult, String> {
    log::info!("install_dependency called for: {}", name);

    let (program, args): (&str, Vec<String>) = match name.as_str() {
        "ollama" => ("brew", vec!["install".into(), "ollama".into()]),
        "gemma3:4b" => ("ollama", vec!["pull".into(), "gemma3:4b".into()]),
        "ffmpeg" => ("brew", vec!["install".into(), "ffmpeg".into()]),
        "espeak_ng" => ("brew", vec!["install".into(), "espeak-ng".into()]),
        "tts_model" => {
            let models_dir = super::resolve_models_dir();
            let kokoro_dir = models_dir.join("kokoro");
            let voices_dir = kokoro_dir.join("voices");
            std::fs::create_dir_all(&voices_dir)
                .map_err(|e| format!("Failed to create voices dir: {}", e))?;

            let base_url = "https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX/resolve/main";

            // Download model if not already present
            let model_path = kokoro_dir.join("model_quantized.onnx");
            if !model_path.exists() {
                log::info!("Downloading Kokoro model to {}", model_path.display());
                download_file(
                    &format!("{}/onnx/model_quantized.onnx", base_url),
                    &model_path,
                ).await?;
            }

            // Download voice files (English voices used by the app)
            let voice_names = [
                "af_nova", "bf_emma", "af_heart", "af_bella", "af_jessica",
                "af_sarah", "af_sky", "am_adam", "am_michael", "bm_george",
                "bf_lily", "am_echo",
            ];

            for voice_name in &voice_names {
                let voice_path = voices_dir.join(format!("{}.bin", voice_name));
                if !voice_path.exists() {
                    log::info!("Downloading voice: {}", voice_name);
                    download_file(
                        &format!("{}/voices/{}.bin", base_url, voice_name),
                        &voice_path,
                    ).await?;
                }
            }

            return Ok(InstallResult {
                success: true,
                output: "Kokoro TTS model downloaded successfully".to_string(),
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

/// Download a file from a URL to a local path using reqwest.
async fn download_file(url: &str, path: &std::path::Path) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status {}: {}", response.status(), url));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download body: {}", e))?;

    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|e| format!("Failed to create file {}: {}", path.display(), e))?;

    file.write_all(&bytes)
        .await
        .map_err(|e| format!("Failed to write file {}: {}", path.display(), e))?;

    log::info!("Downloaded {} ({} bytes)", path.display(), bytes.len());
    Ok(())
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
