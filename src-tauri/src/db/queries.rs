use rusqlite::{params, Connection, Result};

/// Apply rusqlite-side migrations that may not be covered by tauri-plugin-sql.
/// Each migration is idempotent so safe to re-run.
pub fn apply_rusqlite_migrations(conn: &Connection) -> Result<()> {
    // Migration 5: Kokoro voice settings
    conn.execute_batch(include_str!("../../migrations/005_kokoro_voice_settings.sql"))?;
    // Migration 6: KittenTTS voice settings
    conn.execute_batch(include_str!("../../migrations/006_kittentts_voice_settings.sql"))?;
    Ok(())
}

pub struct VoiceSettings {
    pub voice: String,
}

/// Read TTS voice settings from app_settings table
pub fn get_voice_settings(conn: &Connection) -> Result<VoiceSettings> {
    let mut voice = "Luna".to_string();

    let mut stmt = conn.prepare(
        "SELECT value FROM app_settings WHERE key = 'tts_voice'",
    )?;
    let rows = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    })?;

    for row in rows {
        voice = row?;
    }

    Ok(VoiceSettings { voice })
}

/// Update audio job status (used from background Tokio tasks)
pub fn update_audio_job_status(
    conn: &Connection,
    job_id: &str,
    status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE audio_jobs SET status = ?1, error_message = ?2, updated_at = datetime('now') WHERE id = ?3",
        params![status, error_message, job_id],
    )?;
    Ok(())
}

/// Update audio job paths
pub fn update_audio_job_paths(
    conn: &Connection,
    job_id: &str,
    voice_path: Option<&str>,
    final_path: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE audio_jobs SET voice_path = ?1, final_path = ?2, updated_at = datetime('now') WHERE id = ?3",
        params![voice_path, final_path, job_id],
    )?;
    Ok(())
}

/// Reset any in-progress audio jobs to failed on app restart.
/// This handles the case where the app was closed or crashed mid-generation.
pub fn reset_stale_audio_jobs(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE audio_jobs SET status = 'failed', error_message = 'Interrupted by app restart', updated_at = datetime('now') WHERE status IN ('pending', 'voice_generating')",
        [],
    )?;
    conn.execute(
        "UPDATE story_parts SET status = 'audio_failed', updated_at = datetime('now') WHERE status = 'audio_processing'",
        [],
    )?;
    Ok(())
}

/// Update story part status and audio path
pub fn update_story_part_audio(
    conn: &Connection,
    part_id: &str,
    status: &str,
    audio_path: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE story_parts SET status = ?1, audio_path = ?2, updated_at = datetime('now') WHERE id = ?3",
        params![status, audio_path, part_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_audio_job_status() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audio_jobs (
                id TEXT PRIMARY KEY,
                story_part_id TEXT NOT NULL,
                voice_path TEXT,
                final_path TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('job1', 'part1', 'pending');",
        )
        .unwrap();

        update_audio_job_status(&conn, "job1", "voice_generating", None).unwrap();

        let status: String = conn
            .query_row("SELECT status FROM audio_jobs WHERE id = 'job1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "voice_generating");
    }

    #[test]
    fn test_update_audio_job_paths() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audio_jobs (
                id TEXT PRIMARY KEY,
                story_part_id TEXT NOT NULL,
                voice_path TEXT,
                final_path TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('job1', 'part1', 'pending');",
        )
        .unwrap();

        update_audio_job_paths(
            &conn,
            "job1",
            Some("/voice.wav"),
            Some("/final.wav"),
        )
        .unwrap();

        let (voice, final_p): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT voice_path, final_path FROM audio_jobs WHERE id = 'job1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(voice.unwrap(), "/voice.wav");
        assert_eq!(final_p.unwrap(), "/final.wav");
    }

    #[test]
    fn test_reset_stale_audio_jobs() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audio_jobs (
                id TEXT PRIMARY KEY,
                story_part_id TEXT NOT NULL,
                voice_path TEXT,
                final_path TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE story_parts (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL DEFAULT 'text_ready',
                audio_path TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('j1', 'p1', 'pending');
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('j2', 'p2', 'voice_generating');
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('j3', 'p3', 'complete');
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('j4', 'p4', 'failed');
            INSERT INTO story_parts (id, status) VALUES ('p1', 'audio_processing');
            INSERT INTO story_parts (id, status) VALUES ('p2', 'audio_processing');
            INSERT INTO story_parts (id, status) VALUES ('p3', 'audio_ready');
            INSERT INTO story_parts (id, status) VALUES ('p4', 'text_ready');",
        )
        .unwrap();

        reset_stale_audio_jobs(&conn).unwrap();

        // In-progress jobs (pending, voice_generating) should be failed
        for job_id in &["j1", "j2"] {
            let (status, error): (String, Option<String>) = conn
                .query_row(
                    "SELECT status, error_message FROM audio_jobs WHERE id = ?1",
                    params![job_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(status, "failed", "job {} should be failed", job_id);
            assert_eq!(error.unwrap(), "Interrupted by app restart");
        }

        // Complete job should be untouched
        let status: String = conn
            .query_row("SELECT status FROM audio_jobs WHERE id = 'j3'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "complete");

        // Already-failed job should be untouched (error_message stays null)
        let status: String = conn
            .query_row("SELECT status FROM audio_jobs WHERE id = 'j4'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "failed");

        // audio_processing story parts should be audio_failed
        for part_id in &["p1", "p2"] {
            let status: String = conn
                .query_row(
                    "SELECT status FROM story_parts WHERE id = ?1",
                    params![part_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(status, "audio_failed", "part {} should be audio_failed", part_id);
        }

        // Other story parts should be untouched
        let status: String = conn
            .query_row("SELECT status FROM story_parts WHERE id = 'p3'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "audio_ready");

        let status: String = conn
            .query_row("SELECT status FROM story_parts WHERE id = 'p4'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "text_ready");
    }

    /// Apply migrations 004 + 005 + 006 to set up app_settings in test DB
    fn setup_app_settings(conn: &Connection) {
        conn.execute_batch(include_str!("../../migrations/004_app_settings.sql")).unwrap();
        conn.execute_batch(include_str!("../../migrations/005_kokoro_voice_settings.sql")).unwrap();
        conn.execute_batch(include_str!("../../migrations/006_kittentts_voice_settings.sql")).unwrap();
    }

    #[test]
    fn test_get_voice_settings_defaults() {
        let conn = Connection::open_in_memory().unwrap();
        setup_app_settings(&conn);

        let settings = get_voice_settings(&conn).unwrap();
        assert_eq!(settings.voice, "Luna");
    }

    #[test]
    fn test_get_voice_settings_custom() {
        let conn = Connection::open_in_memory().unwrap();
        setup_app_settings(&conn);
        conn.execute(
            "UPDATE app_settings SET value = 'Kiki' WHERE key = 'tts_voice'",
            [],
        )
        .unwrap();

        let settings = get_voice_settings(&conn).unwrap();
        assert_eq!(settings.voice, "Kiki");
    }
}
