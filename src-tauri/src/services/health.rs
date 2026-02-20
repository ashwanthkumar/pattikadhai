use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_command: String,
}

pub async fn check_ollama() -> DependencyStatus {
    let client = reqwest::Client::new();
    let result = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await;

    match result {
        Ok(response) if response.status().is_success() => DependencyStatus {
            name: "Ollama".to_string(),
            installed: true,
            version: Some("Running".to_string()),
            install_command: "brew install ollama && ollama serve".to_string(),
        },
        _ => DependencyStatus {
            name: "Ollama".to_string(),
            installed: false,
            version: None,
            install_command: "brew install ollama && ollama serve".to_string(),
        },
    }
}

pub async fn check_gemma3() -> DependencyStatus {
    let client = reqwest::Client::new();
    let result = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await;

    match result {
        Ok(response) if response.status().is_success() => {
            #[derive(Deserialize)]
            struct TagsResponse {
                models: Vec<ModelInfo>,
            }
            #[derive(Deserialize)]
            struct ModelInfo {
                name: String,
            }

            if let Ok(tags) = response.json::<TagsResponse>().await {
                let has_gemma = tags.models.iter().any(|m| m.name.starts_with("gemma3:4b"));
                DependencyStatus {
                    name: "Gemma 3".to_string(),
                    installed: has_gemma,
                    version: if has_gemma {
                        Some("Available".to_string())
                    } else {
                        None
                    },
                    install_command: "ollama pull gemma3".to_string(),
                }
            } else {
                DependencyStatus {
                    name: "Gemma 3".to_string(),
                    installed: false,
                    version: None,
                    install_command: "ollama pull gemma3".to_string(),
                }
            }
        }
        _ => DependencyStatus {
            name: "Gemma 3".to_string(),
            installed: false,
            version: None,
            install_command: "ollama pull gemma3".to_string(),
        },
    }
}

pub fn check_ffmpeg() -> DependencyStatus {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(output) if output.status.success() => {
            let full = String::from_utf8_lossy(&output.stdout);
            let version = full.lines().next().unwrap_or("").to_string();
            DependencyStatus {
                name: "ffmpeg".to_string(),
                installed: true,
                version: Some(version),
                install_command: "brew install ffmpeg".to_string(),
            }
        }
        _ => DependencyStatus {
            name: "ffmpeg".to_string(),
            installed: false,
            version: None,
            install_command: "brew install ffmpeg".to_string(),
        },
    }
}

pub fn check_espeak_ng() -> DependencyStatus {
    match Command::new("espeak-ng").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DependencyStatus {
                name: "espeak-ng".to_string(),
                installed: true,
                version: Some(version),
                install_command: "brew install espeak-ng".to_string(),
            }
        }
        _ => DependencyStatus {
            name: "espeak-ng".to_string(),
            installed: false,
            version: None,
            install_command: "brew install espeak-ng".to_string(),
        },
    }
}

pub fn check_tts_model(models_dir: &str) -> DependencyStatus {
    let kokoro_dir = std::path::Path::new(models_dir).join("kokoro");
    let model_exists = kokoro_dir.join("model_quantized.onnx").exists();
    let voices_dir = kokoro_dir.join("voices");
    // Check that the voices directory exists and has at least one .bin file
    let voices_exist = voices_dir.is_dir()
        && std::fs::read_dir(&voices_dir)
            .map(|entries| entries.filter_map(|e| e.ok()).any(|e| {
                e.path().extension().and_then(|ext| ext.to_str()) == Some("bin")
            }))
            .unwrap_or(false);
    let ok = model_exists && voices_exist;

    DependencyStatus {
        name: "TTS Model".to_string(),
        installed: ok,
        version: if ok { Some("Downloaded".to_string()) } else { None },
        install_command: "Download via setup wizard".to_string(),
    }
}
