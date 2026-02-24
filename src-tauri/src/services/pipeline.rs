use std::path::PathBuf;

use crate::db::queries::VoiceSettings;
use crate::services::tts::TtsService;
use tauri::Emitter;

/// Naive sentence splitter: split on ". ", "! ", "? " keeping the delimiter with the preceding text.
fn split_sentences(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        if (bytes[i] == b'.' || bytes[i] == b'!' || bytes[i] == b'?') && bytes[i + 1] == b' ' {
            result.push(&text[start..=i]);
            start = i + 2; // skip the space
        }
    }
    if start < text.len() {
        result.push(&text[start..]);
    }
    result
}

pub struct AudioPipeline {
    tts: TtsService,
    audio_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PipelineProgress {
    pub job_id: String,
    pub stage: String,
    pub progress: f32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SentenceAudio {
    pub job_id: String,
    pub index: usize,
    pub total: usize,
    pub text: String,
    pub wav_path: String,
    pub duration_secs: f64,
}

pub struct PipelineResult {
    pub audio_path: String,
    pub timing_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimingSegment {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

impl AudioPipeline {
    pub fn new(audio_dir: PathBuf, models_dir: PathBuf) -> Self {
        Self {
            tts: TtsService::new(models_dir),
            audio_dir,
        }
    }

    /// Run the voice-only audio pipeline for a story part with per-sentence streaming.
    pub async fn process(
        &self,
        job_id: &str,
        part_id: &str,
        text: &str,
        app_handle: &tauri::AppHandle,
        voice_settings: Option<&VoiceSettings>,
    ) -> Result<PipelineResult, String> {
        let final_path = self
            .audio_dir
            .join(format!("{}_final.wav", part_id))
            .to_string_lossy()
            .to_string();
        let timing_path = self
            .audio_dir
            .join(format!("{}_timing.json", part_id))
            .to_string_lossy()
            .to_string();

        // Ensure audio directory exists
        tokio::fs::create_dir_all(&self.audio_dir)
            .await
            .map_err(|e| format!("Failed to create audio dir: {}", e))?;

        // Extract voice settings
        let voice_name = voice_settings.map(|vs| vs.voice.as_str());
        let speed = voice_settings.and_then(|vs| vs.speed);

        // Split text into sentences
        let sentences = split_sentences(text);
        let total = sentences.len();

        // Emit start with total sentence count
        let _ = app_handle.emit(
            "audio-progress",
            PipelineProgress {
                job_id: job_id.to_string(),
                stage: "voice_generating".to_string(),
                progress: 0.0,
                error: None,
            },
        );

        let mut all_samples: Vec<f32> = Vec::new();
        let mut timing_segments: Vec<TimingSegment> = Vec::new();
        let mut sentence_wav_paths: Vec<String> = Vec::new();
        let mut cumulative_secs: f64 = 0.0;
        let mut sample_rate: u32 = 24000;

        for (i, sentence_text) in sentences.iter().enumerate() {
            let sentence_text = sentence_text.trim();
            if sentence_text.is_empty() {
                continue;
            }

            // Generate raw audio for this sentence
            let raw = self
                .tts
                .generate_raw(sentence_text, voice_name, speed)
                .await?;
            sample_rate = raw.sample_rate;

            // Save per-sentence WAV
            let sent_wav_path = self
                .audio_dir
                .join(format!("{}_sent_{}.wav", part_id, i))
                .to_string_lossy()
                .to_string();

            let path_for_save = sent_wav_path.clone();
            let samples_for_save = raw.samples.clone();
            let sr = raw.sample_rate;
            tokio::task::spawn_blocking(move || {
                kokoro_tts::audio::save_wav(
                    std::path::Path::new(&path_for_save),
                    &samples_for_save,
                    sr,
                )
                .map_err(|e| format!("Failed to save sentence WAV: {}", e))
            })
            .await
            .map_err(|e| format!("Save WAV task panicked: {}", e))??;

            sentence_wav_paths.push(sent_wav_path.clone());

            // Build timing segment
            let start = cumulative_secs;
            cumulative_secs += raw.duration_secs;
            timing_segments.push(TimingSegment {
                text: sentence_text.to_string(),
                start,
                end: cumulative_secs,
            });

            // Accumulate samples for final WAV
            all_samples.extend_from_slice(&raw.samples);

            // Emit per-sentence event
            let _ = app_handle.emit(
                "audio-sentence",
                SentenceAudio {
                    job_id: job_id.to_string(),
                    index: i,
                    total,
                    text: sentence_text.to_string(),
                    wav_path: sent_wav_path,
                    duration_secs: raw.duration_secs,
                },
            );

            // Emit progress update
            let progress = (i + 1) as f32 / total as f32;
            let _ = app_handle.emit(
                "audio-progress",
                PipelineProgress {
                    job_id: job_id.to_string(),
                    stage: "voice_generating".to_string(),
                    progress,
                    error: None,
                },
            );
        }

        // Write final concatenated WAV from accumulated samples
        {
            let final_path_clone = final_path.clone();
            let samples = all_samples;
            let sr = sample_rate;
            tokio::task::spawn_blocking(move || {
                kokoro_tts::audio::save_wav(
                    std::path::Path::new(&final_path_clone),
                    &samples,
                    sr,
                )
                .map_err(|e| format!("Failed to save final WAV: {}", e))
            })
            .await
            .map_err(|e| format!("Save final WAV task panicked: {}", e))??;
        }

        // Write timing JSON sidecar
        let timing_json = serde_json::to_string_pretty(&timing_segments)
            .map_err(|e| format!("Failed to serialize timing: {}", e))?;
        tokio::fs::write(&timing_path, timing_json)
            .await
            .map_err(|e| format!("Failed to write timing JSON: {}", e))?;

        // Clean up per-sentence WAV files
        for path in &sentence_wav_paths {
            let _ = tokio::fs::remove_file(path).await;
        }

        // Emit completion
        let _ = app_handle.emit(
            "audio-progress",
            PipelineProgress {
                job_id: job_id.to_string(),
                stage: "complete".to_string(),
                progress: 1.0,
                error: None,
            },
        );

        Ok(PipelineResult {
            audio_path: final_path,
            timing_path,
        })
    }
}

/// Build the audio output directory path given a part ID
#[cfg(test)]
pub fn audio_path_for_part(audio_dir: &str, part_id: &str, suffix: &str) -> String {
    format!("{}/{}_{}", audio_dir, part_id, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_path_is_deterministic() {
        let path1 = audio_path_for_part("/audio", "part-123", "final.wav");
        let path2 = audio_path_for_part("/audio", "part-123", "final.wav");
        assert_eq!(path1, path2);
        assert_eq!(path1, "/audio/part-123_final.wav");
    }

    #[test]
    fn split_sentences_basic() {
        let sentences = split_sentences("Hello world. How are you? I am fine!");
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "How are you?");
        assert_eq!(sentences[2], "I am fine!");
    }

    #[test]
    fn split_sentences_no_trailing_space() {
        let sentences = split_sentences("One sentence.");
        assert_eq!(sentences, vec!["One sentence."]);
    }

    #[test]
    fn split_sentences_empty() {
        let sentences = split_sentences("");
        assert!(sentences.is_empty());
    }
}
