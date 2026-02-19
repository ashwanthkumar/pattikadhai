-- Migrate voice settings from Qwen3-TTS to Kokoro
UPDATE app_settings SET value = 'af_nova' WHERE key = 'tts_voice' AND value = 'Vivian';
DELETE FROM app_settings WHERE key IN ('tts_seed', 'tts_temperature');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('tts_speed', '1.0');
