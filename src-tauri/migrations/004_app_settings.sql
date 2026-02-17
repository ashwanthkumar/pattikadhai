CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('tts_voice', 'Vivian');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('tts_seed', '42');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('tts_temperature', '0.3');
