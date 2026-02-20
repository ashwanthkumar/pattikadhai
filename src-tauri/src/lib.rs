mod commands;
mod db;
mod services;

use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};

fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "Create initial schema",
            sql: include_str!("../migrations/001_initial.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "Seed genres",
            sql: include_str!("../migrations/002_seed_genres.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "Seed sample stories",
            sql: include_str!("../migrations/003_seed_stories.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "App settings table",
            sql: include_str!("../migrations/004_app_settings.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "Migrate voice settings to Kokoro",
            sql: include_str!("../migrations/005_kokoro_voice_settings.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "Migrate voice settings to KittenTTS",
            sql: include_str!("../migrations/006_kittentts_voice_settings.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:pattikadhai.db", migrations())
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let db_path = app.path().app_data_dir()?.join("pattikadhai.db");
            if db_path.exists() {
                match rusqlite::Connection::open(&db_path) {
                    Ok(conn) => {
                        match db::queries::apply_rusqlite_migrations(&conn) {
                            Ok(()) => log::info!("Applied rusqlite migrations on startup"),
                            Err(e) => log::warn!("Failed to apply rusqlite migrations: {e}"),
                        }
                        match db::queries::reset_stale_audio_jobs(&conn) {
                            Ok(()) => log::info!("Reset stale audio jobs on startup"),
                            Err(e) => log::warn!("Failed to reset stale audio jobs: {e}"),
                        }
                    }
                    Err(e) => log::warn!("Failed to open DB on startup: {e}"),
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::check_dependency,
            commands::health::install_dependency,
            commands::health::apply_migrations,
            commands::stories::generate_story_text,
            commands::stories::continue_story,
            commands::stories::get_story_detail,
            commands::audio::start_audio_generation,
            commands::audio::get_audio_job_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
