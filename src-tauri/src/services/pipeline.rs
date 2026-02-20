use std::path::PathBuf;

use crate::db::queries::VoiceSettings;
use crate::services::mixer::AudioMixer;
use crate::services::tts::TtsService;
use tauri::Emitter;

const CHUNK_SIZE: usize = 800;

/// Merge paragraphs into chunks of approximately `CHUNK_SIZE` characters.
/// Oversized paragraphs are split at sentence boundaries.
fn chunk_text(text: &str) -> Vec<String> {
    let paragraphs: Vec<&str> = text.split("\n\n").filter(|p| !p.trim().is_empty()).collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for para in paragraphs {
        let para = para.trim();
        if para.len() > CHUNK_SIZE {
            // Flush current buffer first
            if !current.is_empty() {
                chunks.push(current.clone());
                current.clear();
            }
            // Split oversized paragraph at sentence boundaries
            let mut sentence_buf = String::new();
            for sentence in split_sentences(para) {
                if !sentence_buf.is_empty() && sentence_buf.len() + sentence.len() + 1 > CHUNK_SIZE {
                    chunks.push(sentence_buf.clone());
                    sentence_buf.clear();
                }
                if !sentence_buf.is_empty() {
                    sentence_buf.push(' ');
                }
                sentence_buf.push_str(sentence);
            }
            if !sentence_buf.is_empty() {
                chunks.push(sentence_buf);
            }
        } else if !current.is_empty() && current.len() + para.len() + 2 > CHUNK_SIZE {
            chunks.push(current.clone());
            current.clear();
            current.push_str(para);
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_string());
    }
    chunks
}

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

impl AudioPipeline {
    pub fn new(audio_dir: PathBuf, models_dir: PathBuf) -> Self {
        Self {
            tts: TtsService::new(models_dir),
            audio_dir,
        }
    }

    /// Run the voice-only audio pipeline for a story part
    pub async fn process(
        &self,
        job_id: &str,
        part_id: &str,
        text: &str,
        app_handle: &tauri::AppHandle,
        voice_settings: Option<&VoiceSettings>,
    ) -> Result<String, String> {
        let final_path = self
            .audio_dir
            .join(format!("{}_final.wav", part_id))
            .to_string_lossy()
            .to_string();

        // Ensure audio directory exists
        tokio::fs::create_dir_all(&self.audio_dir)
            .await
            .map_err(|e| format!("Failed to create audio dir: {}", e))?;

        // Emit voice generation start
        let _ = app_handle.emit(
            "audio-progress",
            PipelineProgress {
                job_id: job_id.to_string(),
                stage: "voice_generating".to_string(),
                progress: 0.1,
                error: None,
            },
        );

        // Extract voice settings
        let voice_name = voice_settings.map(|vs| vs.voice.as_str());
        let speed = voice_settings.and_then(|vs| vs.speed);

        // Merge paragraphs into ~800-char chunks
        let chunks = chunk_text(text);

        if chunks.len() > 1 {
            let mut wav_paths = Vec::new();
            for (i, chunk) in chunks.iter().enumerate() {
                let chunk_path = self
                    .audio_dir
                    .join(format!("{}_chunk_{}.wav", part_id, i))
                    .to_string_lossy()
                    .to_string();
                self.tts.generate(chunk, &chunk_path, voice_name, speed).await?;
                wav_paths.push(chunk_path);
            }

            // Concat all chunks into final_path
            let refs: Vec<&str> = wav_paths.iter().map(|s| s.as_str()).collect();
            AudioMixer::concat_wav(&refs, &final_path).await?;

            // Clean up chunk files
            for path in &wav_paths {
                let _ = tokio::fs::remove_file(path).await;
            }
        } else {
            // Single chunk: TTS writes directly to final_path
            self.tts.generate(text, &final_path, voice_name, speed).await?;
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

        Ok(final_path)
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
    fn chunk_text_single_short_paragraph() {
        let chunks = chunk_text("Hello world.");
        assert_eq!(chunks, vec!["Hello world."]);
    }

    #[test]
    fn chunk_text_merges_small_paragraphs() {
        let text = "Para one.\n\nPara two.\n\nPara three.";
        let chunks = chunk_text(text);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("Para one."));
        assert!(chunks[0].contains("Para three."));
    }

    #[test]
    fn chunk_text_splits_at_size_boundary() {
        // Create paragraphs that together exceed CHUNK_SIZE
        let para = "A".repeat(1200);
        let text = format!("{}\n\n{}", para, para);
        let chunks = chunk_text(&text);
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn chunk_text_splits_oversized_paragraph() {
        // Single paragraph > CHUNK_SIZE with sentence boundaries
        let sentence = "This is a sentence. ";
        let big_para = sentence.repeat(150); // ~3000 chars
        let chunks = chunk_text(&big_para);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.len() <= CHUNK_SIZE + 100); // some tolerance
        }
    }

    #[test]
    fn chunk_text_empty_input() {
        let chunks = chunk_text("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn chunk_text_whitespace_only() {
        let chunks = chunk_text("   \n\n   ");
        assert!(chunks.is_empty());
    }
}
