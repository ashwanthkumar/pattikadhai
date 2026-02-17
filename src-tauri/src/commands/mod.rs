pub mod audio;
pub mod health;
pub mod stories;

use tauri::Manager;

/// Resolve the scripts/ directory, preferring the workspace root in dev mode.
pub fn resolve_scripts_dir(app: &tauri::AppHandle) -> std::path::PathBuf {
    // In dev mode, resource_dir points into the build output which won't have scripts.
    // Prefer the workspace scripts/ directory relative to the Cargo manifest dir.
    let dev_scripts = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("scripts"));

    if let Some(ref dir) = dev_scripts {
        if dir.exists() {
            return dir.clone();
        }
    }

    // Fallback for bundled app
    let resource = app.path().resource_dir().map(|d| d.join("scripts"));

    if let Ok(ref dir) = resource {
        if dir.exists() {
            return dir.clone();
        }
    }

    // Last resort
    std::env::current_dir()
        .unwrap_or_default()
        .join("scripts")
}

/// Resolve the shared models directory at `~/.pattikadhai/models/`.
pub fn resolve_models_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".pattikadhai")
        .join("models")
}
