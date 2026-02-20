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
                let has_gemma = tags.models.iter().any(|m| m.name.starts_with("gemma3"));
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

pub fn check_uv() -> DependencyStatus {
    match Command::new("uv").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            DependencyStatus {
                name: "uv".to_string(),
                installed: true,
                version: Some(version),
                install_command: "brew install uv".to_string(),
            }
        }
        _ => DependencyStatus {
            name: "uv".to_string(),
            installed: false,
            version: None,
            install_command: "brew install uv".to_string(),
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

pub async fn check_tts_model(scripts_dir: &str, models_dir: &str) -> DependencyStatus {
    let tts_dir = format!("{}/tts", scripts_dir);
    let hf_home = format!("{}/huggingface", models_dir);

    let mut cmd = tokio::process::Command::new("uv");
    cmd.args([
            "run", "--frozen", "--project", &tts_dir,
            "python",
            &format!("{}/check_model.py", tts_dir),
        ])
        .env("HF_HOME", &hf_home);
    if let Some(lib) = super::tts::espeak_library_path() {
        cmd.env("PHONEMIZER_ESPEAK_LIBRARY", lib);
    }

    let ok = cmd
        .output()
        .await
        .map(|o| {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout).trim() == "installed"
        })
        .unwrap_or(false);

    DependencyStatus {
        name: "TTS Model".to_string(),
        installed: ok,
        version: if ok { Some("Downloaded".to_string()) } else { None },
        install_command: "Download via setup wizard".to_string(),
    }
}

pub async fn check_python_deps(scripts_dir: &str) -> DependencyStatus {
    let tts_dir = format!("{}/tts", scripts_dir);

    let mut cmd = tokio::process::Command::new("uv");
    cmd.args([
            "run", "--frozen", "--project", &tts_dir,
            "python", "-c", "import kittentts; print('ok')",
        ]);
    if let Some(lib) = super::tts::espeak_library_path() {
        cmd.env("PHONEMIZER_ESPEAK_LIBRARY", lib);
    }

    let tts_ok = cmd
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    DependencyStatus {
        name: "Python ML deps".to_string(),
        installed: tts_ok,
        version: if tts_ok { Some("Installed".to_string()) } else { None },
        install_command: format!("uv sync --project {}", tts_dir),
    }
}
