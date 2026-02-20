pub mod audio;
pub mod health;
pub mod stories;

/// Resolve the shared models directory at `~/.pattikadhai/models/`.
pub fn resolve_models_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".pattikadhai")
        .join("models")
}
