pub mod audio;
pub mod model;
pub mod phonemize;
pub mod tokenize;
pub mod voices;

use std::path::Path;

use model::KokoroModel;
use voices::VoiceStore;

/// Errors from the Kokoro TTS pipeline.
#[derive(Debug, thiserror::Error)]
pub enum KokoroError {
    #[error("espeak-ng not found. Install with: brew install espeak-ng")]
    EspeakNotFound,

    #[error("Phonemization failed: {0}")]
    Phonemize(String),

    #[error("Voice error: {0}")]
    Voice(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Audio error: {0}")]
    Audio(String),
}

/// Generated audio data.
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioData {
    /// Save audio to a WAV file.
    pub fn save_wav(&self, path: &Path) -> Result<(), KokoroError> {
        audio::save_wav(path, &self.samples, self.sample_rate)
    }
}

/// The main Kokoro TTS engine.
pub struct Kokoro {
    model: KokoroModel,
    voices: VoiceStore,
}

impl Kokoro {
    /// Load model and voices.
    ///
    /// - `model_path`: path to `model_quantized.onnx`
    /// - `voices_path`: path to voices directory (containing `*.bin` files)
    ///   or a single NPZ archive file
    pub fn new(model_path: &Path, voices_path: &Path) -> Result<Self, KokoroError> {
        let model = KokoroModel::new(model_path)?;
        let voices = if voices_path.is_dir() {
            VoiceStore::load_dir(voices_path)?
        } else {
            VoiceStore::load_npz(voices_path)?
        };
        Ok(Self { model, voices })
    }

    /// Generate speech audio from text.
    ///
    /// - `text`: input text to synthesize
    /// - `voice`: voice preset name (e.g., "af_nova")
    /// - `speed`: speech rate (1.0 = normal)
    /// - `lang`: espeak-ng language code (e.g., "en-us")
    pub fn create(
        &mut self,
        text: &str,
        voice: &str,
        speed: f32,
        lang: &str,
    ) -> Result<AudioData, KokoroError> {
        // Step 1: Text → IPA phonemes
        let phonemes = phonemize::phonemize(text, lang)?;
        log::info!("Phonemized {} chars → {} phoneme chars", text.len(), phonemes.len());

        // Step 2: Phonemes → token IDs
        let tokens = tokenize::tokenize(&phonemes);

        if tokens.is_empty() {
            return Err(KokoroError::Phonemize(
                "No valid tokens produced from text".to_string(),
            ));
        }

        // Step 3: Handle long text by chunking at sentence boundaries
        if tokens.len() >= tokenize::MAX_PHONEME_LEN {
            return self.create_chunked(text, voice, speed, lang);
        }

        // Step 4: Get voice embedding for this token length
        let voice_data = self.voices.get(voice)?;
        let style = voice_data.embedding(tokens.len())?;

        // Step 5: Pad tokens with [0, ...tokens, 0]
        let padded = tokenize::pad_tokens(&tokens);

        // Step 6: Run ONNX inference
        let mut samples = self.model.infer(&padded, style, speed)?;
        log::info!("Generated {} audio samples", samples.len());

        // Normalize volume so speech is not too quiet
        audio::normalize(&mut samples, 0.95);

        Ok(AudioData {
            samples,
            sample_rate: audio::SAMPLE_RATE,
        })
    }

    /// Generate audio for text that exceeds the 510-token limit.
    /// Splits at sentence boundaries and concatenates audio chunks.
    fn create_chunked(
        &mut self,
        text: &str,
        voice: &str,
        speed: f32,
        lang: &str,
    ) -> Result<AudioData, KokoroError> {
        let sentences = split_sentences(text);
        let mut chunks: Vec<Vec<f32>> = Vec::new();
        let mut batch = String::new();

        for sentence in &sentences {
            // Check if adding this sentence would exceed the limit
            let test = if batch.is_empty() {
                sentence.to_string()
            } else {
                format!("{} {}", batch, sentence)
            };

            let test_phonemes = phonemize::phonemize(&test, lang)?;
            let test_tokens = tokenize::tokenize(&test_phonemes);

            if test_tokens.len() >= tokenize::MAX_PHONEME_LEN && !batch.is_empty() {
                // Process current batch
                let audio = self.create_single(&batch, voice, speed, lang)?;
                chunks.push(audio);
                batch = sentence.to_string();
            } else {
                batch = test;
            }
        }

        // Process remaining batch
        if !batch.is_empty() {
            let audio = self.create_single(&batch, voice, speed, lang)?;
            chunks.push(audio);
        }

        let mut samples = audio::concat_samples(&chunks);
        audio::normalize(&mut samples, 0.95);
        Ok(AudioData {
            samples,
            sample_rate: audio::SAMPLE_RATE,
        })
    }

    /// Generate audio for a single chunk (must be within 510-token limit).
    fn create_single(
        &mut self,
        text: &str,
        voice: &str,
        speed: f32,
        lang: &str,
    ) -> Result<Vec<f32>, KokoroError> {
        let phonemes = phonemize::phonemize(text, lang)?;
        let tokens = tokenize::tokenize(&phonemes);

        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        let voice_data = self.voices.get(voice)?;
        let style = voice_data.embedding(tokens.len())?;
        let padded = tokenize::pad_tokens(&tokens);

        self.model.infer(&padded, style, speed)
    }

    /// List available voice names.
    pub fn voices(&self) -> Vec<String> {
        self.voices.names()
    }
}

/// Split text at sentence boundaries (. ! ? followed by space or end of string).
fn split_sentences(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for i in 0..len {
        current.push(chars[i]);

        let is_sentence_end =
            (chars[i] == '.' || chars[i] == '!' || chars[i] == '?')
                && (i + 1 >= len || chars[i + 1] == ' ');

        if is_sentence_end {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                result.push(trimmed);
            }
            current.clear();
            // Skip the space after punctuation
            if i + 1 < len && chars[i + 1] == ' ' {
                // The space will be skipped by not adding it
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences() {
        let sentences = split_sentences("Hello world. How are you? I'm fine!");
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "How are you?");
        assert_eq!(sentences[2], "I'm fine!");
    }

    #[test]
    fn test_split_sentences_no_punctuation() {
        let sentences = split_sentences("Hello world");
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0], "Hello world");
    }

    #[test]
    fn test_split_sentences_empty() {
        let sentences = split_sentences("");
        assert!(sentences.is_empty());
    }

    #[test]
    fn test_integration() {
        // Full integration test - requires model files and espeak-ng
        let model_path = Path::new("/tmp/kokoro-inspect/model_quantized.onnx");
        let voices_path = Path::new("/tmp/kokoro-inspect/voices-v1.0.bin");

        if !model_path.exists() || !voices_path.exists() {
            return; // Skip if files not available
        }

        let mut kokoro = Kokoro::new(model_path, voices_path).unwrap();

        // Check voices loaded
        let voices = kokoro.voices();
        assert!(voices.len() >= 50);
        assert!(voices.contains(&"af_nova".to_string()));

        // Generate audio
        match kokoro.create("Hello, world!", "af_nova", 1.0, "en-us") {
            Ok(audio) => {
                assert_eq!(audio.sample_rate, 24000);
                assert!(!audio.samples.is_empty());

                // Save to temp file
                let path = std::env::temp_dir().join("kokoro_integration_test.wav");
                audio.save_wav(&path).unwrap();
                assert!(path.exists());
                let _ = std::fs::remove_file(&path);
            }
            Err(KokoroError::EspeakNotFound) => {
                // OK - skip if espeak-ng not installed
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
