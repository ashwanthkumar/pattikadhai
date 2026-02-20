-- Revert KittenTTS voices back to Kokoro voice names
UPDATE app_settings SET value = 'af_nova' WHERE key = 'tts_voice' AND value = 'Luna';
UPDATE app_settings SET value = 'bf_emma' WHERE key = 'tts_voice' AND value = 'Rosie';
UPDATE app_settings SET value = 'af_heart' WHERE key = 'tts_voice' AND value = 'Kiki';
UPDATE app_settings SET value = 'af_bella' WHERE key = 'tts_voice' AND value = 'Bella';
UPDATE app_settings SET value = 'am_adam' WHERE key = 'tts_voice' AND value = 'Jasper';
UPDATE app_settings SET value = 'am_michael' WHERE key = 'tts_voice' AND value = 'Hugo';
UPDATE app_settings SET value = 'bm_george' WHERE key = 'tts_voice' AND value = 'Bruno';
UPDATE app_settings SET value = 'af_nova' WHERE key = 'tts_voice' AND value = 'Leo';
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('tts_speed', '1.0');
