use rusqlite::{params, Connection, Result};

pub struct VoiceSettings {
    pub voice: String,
    pub seed: u64,
    pub temperature: f64,
}

/// Read TTS voice settings from app_settings table
pub fn get_voice_settings(conn: &Connection) -> Result<VoiceSettings> {
    let mut voice = "Vivian".to_string();
    let mut seed: u64 = 42;
    let mut temperature: f64 = 0.3;

    let mut stmt = conn.prepare(
        "SELECT key, value FROM app_settings WHERE key IN ('tts_voice', 'tts_seed', 'tts_temperature')",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "tts_voice" => voice = value,
            "tts_seed" => seed = value.parse().unwrap_or(42),
            "tts_temperature" => temperature = value.parse().unwrap_or(0.3),
            _ => {}
        }
    }

    Ok(VoiceSettings {
        voice,
        seed,
        temperature,
    })
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
    music_path: Option<&str>,
    final_path: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE audio_jobs SET voice_path = ?1, music_path = ?2, final_path = ?3, updated_at = datetime('now') WHERE id = ?4",
        params![voice_path, music_path, final_path, job_id],
    )?;
    Ok(())
}

/// Reset any in-progress audio jobs to failed on app restart.
/// This handles the case where the app was closed or crashed mid-generation.
pub fn reset_stale_audio_jobs(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE audio_jobs SET status = 'failed', error_message = 'Interrupted by app restart', updated_at = datetime('now') WHERE status IN ('pending', 'voice_generating', 'music_generating', 'mixing')",
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
                music_path TEXT,
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
                music_path TEXT,
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
            Some("/music.wav"),
            Some("/final.mp3"),
        )
        .unwrap();

        let (voice, music, final_p): (Option<String>, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT voice_path, music_path, final_path FROM audio_jobs WHERE id = 'job1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(voice.unwrap(), "/voice.wav");
        assert_eq!(music.unwrap(), "/music.wav");
        assert_eq!(final_p.unwrap(), "/final.mp3");
    }

    #[test]
    fn test_reset_stale_audio_jobs() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audio_jobs (
                id TEXT PRIMARY KEY,
                story_part_id TEXT NOT NULL,
                voice_path TEXT,
                music_path TEXT,
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
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('j3', 'p3', 'music_generating');
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('j4', 'p4', 'mixing');
            INSERT INTO audio_jobs (id, story_part_id, status) VALUES ('j5', 'p5', 'completed');
            INSERT INTO story_parts (id, status) VALUES ('p1', 'audio_processing');
            INSERT INTO story_parts (id, status) VALUES ('p2', 'audio_processing');
            INSERT INTO story_parts (id, status) VALUES ('p3', 'audio_ready');
            INSERT INTO story_parts (id, status) VALUES ('p4', 'text_ready');",
        )
        .unwrap();

        reset_stale_audio_jobs(&conn).unwrap();

        // All in-progress jobs should be failed
        for job_id in &["j1", "j2", "j3", "j4"] {
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

        // Completed job should be untouched
        let status: String = conn
            .query_row("SELECT status FROM audio_jobs WHERE id = 'j5'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(status, "completed");

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

    #[test]
    fn test_get_voice_settings_defaults() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../migrations/004_app_settings.sql"))
            .unwrap();

        let settings = get_voice_settings(&conn).unwrap();
        assert_eq!(settings.voice, "Vivian");
        assert_eq!(settings.seed, 42);
        assert!((settings.temperature - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_voice_settings_custom() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(include_str!("../../migrations/004_app_settings.sql"))
            .unwrap();
        conn.execute(
            "UPDATE app_settings SET value = '99' WHERE key = 'tts_seed'",
            [],
        )
        .unwrap();
        conn.execute(
            "UPDATE app_settings SET value = '0.7' WHERE key = 'tts_temperature'",
            [],
        )
        .unwrap();

        let settings = get_voice_settings(&conn).unwrap();
        assert_eq!(settings.seed, 99);
        assert!((settings.temperature - 0.7).abs() < f64::EPSILON);
    }
}
