-- Migrate voice settings from Kokoro to KittenTTS
UPDATE app_settings SET value = 'Luna' WHERE key = 'tts_voice' AND value = 'af_nova';
UPDATE app_settings SET value = 'Rosie' WHERE key = 'tts_voice' AND value = 'bf_emma';
UPDATE app_settings SET value = 'Kiki' WHERE key = 'tts_voice' AND value = 'af_heart';
UPDATE app_settings SET value = 'Bella' WHERE key = 'tts_voice' AND value = 'af_bella';
UPDATE app_settings SET value = 'Jasper' WHERE key = 'tts_voice' AND value = 'am_adam';
UPDATE app_settings SET value = 'Hugo' WHERE key = 'tts_voice' AND value = 'am_michael';
UPDATE app_settings SET value = 'Bruno' WHERE key = 'tts_voice' AND value = 'bm_george';
DELETE FROM app_settings WHERE key = 'tts_speed';
