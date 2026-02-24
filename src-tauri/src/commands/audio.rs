use crate::db::models::AudioJob;
use crate::db::queries;
use crate::services::pipeline::{AudioPipeline, PipelineProgress};
use rusqlite::Connection;
use tauri::{Emitter, Manager};
use log;

#[derive(serde::Serialize)]
pub struct AudioJobInfo {
    pub job_id: String,
    pub status: String,
}

#[tauri::command]
pub async fn start_audio_generation(
    job_id: String,
    part_id: String,
    text: String,
    app: tauri::AppHandle,
) -> Result<AudioJobInfo, String> {
    let app_clone = app.clone();
    let job_id_clone = job_id.clone();
    let part_id_clone = part_id.clone();

    // Get paths
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let audio_dir = app_data_dir.join("audio");
    let db_path = app_data_dir.join("pattikadhai.db");

    let models_dir = super::resolve_models_dir();
    log::info!("Audio pipeline models dir: {}", models_dir.display());

    // Spawn background task
    tokio::spawn(async move {
        let pipeline = AudioPipeline::new(audio_dir, models_dir);

        // Read voice settings and update job status
        let voice_settings = if let Ok(conn) = Connection::open(&db_path) {
            let _ = queries::update_audio_job_status(&conn, &job_id_clone, "voice_generating", None);
            let _ = queries::update_story_part_audio(&conn, &part_id_clone, "audio_processing", None);
            queries::get_voice_settings(&conn).ok()
        } else {
            None
        };

        match pipeline
            .process(&job_id_clone, &part_id_clone, &text, &app_clone, voice_settings.as_ref())
            .await
        {
            Ok(result) => {
                if let Ok(conn) = Connection::open(&db_path) {
                    let _ = queries::update_audio_job_status(
                        &conn,
                        &job_id_clone,
                        "complete",
                        None,
                    );
                    let _ = queries::update_audio_job_paths(
                        &conn,
                        &job_id_clone,
                        None,
                        Some(&result.audio_path),
                    );
                    let _ = queries::update_story_part_audio(
                        &conn,
                        &part_id_clone,
                        "audio_ready",
                        Some(&result.audio_path),
                    );
                }
            }
            Err(err) => {
                let _ = app_clone.emit(
                    "audio-progress",
                    PipelineProgress {
                        job_id: job_id_clone.clone(),
                        stage: "failed".to_string(),
                        progress: 0.0,
                        error: Some(err.clone()),
                    },
                );
                if let Ok(conn) = Connection::open(&db_path) {
                    let _ = queries::update_audio_job_status(
                        &conn,
                        &job_id_clone,
                        "failed",
                        Some(&err),
                    );
                    let _ = queries::update_story_part_audio(
                        &conn,
                        &part_id_clone,
                        "audio_failed",
                        None,
                    );
                }
            }
        }
    });

    Ok(AudioJobInfo {
        job_id,
        status: "voice_generating".to_string(),
    })
}

#[tauri::command]
pub async fn get_audio_job_status(
    job_id: String,
    app: tauri::AppHandle,
) -> Result<AudioJob, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let db_path = app_data_dir.join("pattikadhai.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open DB: {}", e))?;

    let result = conn.query_row(
        "SELECT id, story_part_id, voice_path, final_path, status, error_message, created_at, updated_at FROM audio_jobs WHERE id = ?1",
        [&job_id],
        |row| {
            Ok(AudioJob {
                id: row.get(0)?,
                story_part_id: row.get(1)?,
                voice_path: row.get(2)?,
                final_path: row.get(3)?,
                status: row.get(4)?,
                error_message: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    ).map_err(|e| format!("Job not found: {}", e))?;

    Ok(result)
}
