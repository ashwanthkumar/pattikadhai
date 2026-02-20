use std::process::Command;
use std::sync::OnceLock;

use misaki_rs::{G2P, Language};

use crate::KokoroError;

/// Punctuation characters that Kokoro can use for pauses/prosody but
/// phonemizers strip. We split the input text at these characters,
/// phonemize each segment, then rejoin with the original punctuation
/// so the model receives the tokens.
const PRESERVED_PUNCT: &[char] = &[
    '\u{2014}', // — em-dash
    '\u{2026}', // … ellipsis
    ';',
    ':',
    '"',        // straight double quote
    '\u{201c}', // " left smart quote
    '\u{201d}', // " right smart quote
];

/// A segment of text: either a run of normal text or a punctuation char.
enum Segment {
    Text(String),
    Punct(char),
}

/// Split input text into alternating text / punctuation segments.
fn split_preserving_punct(text: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if PRESERVED_PUNCT.contains(&ch) {
            if !current.is_empty() {
                segments.push(Segment::Text(std::mem::take(&mut current)));
            }
            segments.push(Segment::Punct(ch));
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        segments.push(Segment::Text(current));
    }
    segments
}

/// Thread-safe singleton for the misaki G2P engine.
static MISAKI_G2P: OnceLock<G2P> = OnceLock::new();

fn get_misaki() -> &'static G2P {
    MISAKI_G2P.get_or_init(|| {
        log::info!("Initializing misaki-rs G2P (en-us)");
        G2P::new(Language::EnglishUS)
    })
}

/// Phonemize a text segment using misaki-rs (primary) with espeak-ng fallback.
fn phonemize_segment(text: &str, lang: &str) -> Result<String, KokoroError> {
    let g2p = get_misaki();
    let trimmed = match g2p.g2p(text) {
        Ok((phonemes, _tokens)) => phonemes.trim().to_string(),
        Err(e) => {
            log::warn!("misaki-rs failed for {:?}: {}, falling back to espeak-ng", text, e);
            return espeak_phonemize(text, lang);
        }
    };

    if !trimmed.is_empty() {
        log::debug!("misaki-rs phonemized: {:?} → {:?}", text, trimmed);
        return Ok(trimmed.to_string());
    }

    // Fallback to espeak-ng if misaki produced nothing
    log::debug!("misaki-rs returned empty for {:?}, falling back to espeak-ng", text);
    espeak_phonemize(text, lang)
}

/// Convert text to phonemes, preserving punctuation characters that
/// Kokoro uses for pauses but phonemizers would strip.
pub fn phonemize(text: &str, lang: &str) -> Result<String, KokoroError> {
    // Convert newlines to sentence boundaries so paragraphs get pauses.
    let text = text.replace("\n\n", ". ").replace('\n', ". ");
    let segments = split_preserving_punct(&text);
    let mut result = String::new();

    for segment in segments {
        match segment {
            Segment::Punct(ch) => {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push(ch);
                result.push(' ');
            }
            Segment::Text(t) => {
                let trimmed = t.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let phonemes = phonemize_segment(trimmed, lang)?;
                if !phonemes.is_empty() {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push_str(&phonemes);
                }
            }
        }
    }

    Ok(result.trim().to_string())
}

/// Run espeak-ng on a text segment and return IPA phonemes.
/// Joins multi-line output with ". " to preserve sentence boundaries.
fn espeak_phonemize(text: &str, lang: &str) -> Result<String, KokoroError> {
    let output = Command::new("espeak-ng")
        .args(["-v", lang, "--ipa", "-q", text])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                KokoroError::EspeakNotFound
            } else {
                KokoroError::Phonemize(format!("Failed to run espeak-ng: {}", e))
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(KokoroError::Phonemize(format!(
            "espeak-ng failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // espeak-ng outputs each sentence on a separate line, stripping
    // sentence-ending punctuation (. ! ?).  Re-insert ". " between
    // lines so the Kokoro model receives the period token and
    // produces proper pauses/stops at sentence boundaries.
    let phonemes = stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(". ");

    Ok(phonemes)
}

/// Check if espeak-ng is available on the system.
pub fn check_espeak_ng() -> Result<String, KokoroError> {
    let output = Command::new("espeak-ng")
        .arg("--version")
        .output()
        .map_err(|_| KokoroError::EspeakNotFound)?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        Err(KokoroError::EspeakNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_espeak_ng() {
        match check_espeak_ng() {
            Ok(version) => {
                assert!(!version.is_empty());
            }
            Err(KokoroError::EspeakNotFound) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_phonemize_basic() {
        match phonemize("Hello", "en-us") {
            Ok(phonemes) => {
                assert!(!phonemes.is_empty());
            }
            Err(KokoroError::EspeakNotFound) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_misaki_phonemize_directly() {
        let g2p = get_misaki();
        let (phonemes, _tokens) = g2p.g2p("Hello world").unwrap();
        assert!(!phonemes.trim().is_empty(), "misaki should produce phonemes");
    }

    #[test]
    fn test_split_preserving_punct() {
        let segments = split_preserving_punct("he felt something \u{2014} courage");
        let mut texts = Vec::new();
        let mut puncts = Vec::new();
        for seg in segments {
            match seg {
                Segment::Text(t) => texts.push(t),
                Segment::Punct(c) => puncts.push(c),
            }
        }
        assert_eq!(texts.len(), 2);
        assert_eq!(puncts, vec!['\u{2014}']);
        assert_eq!(texts[0], "he felt something ");
        assert_eq!(texts[1], " courage");
    }

    #[test]
    fn test_split_preserving_quotes() {
        let segments = split_preserving_punct("She said \u{201c}hello\u{201d} softly");
        let mut puncts = Vec::new();
        for seg in segments {
            if let Segment::Punct(c) = seg {
                puncts.push(c);
            }
        }
        assert_eq!(puncts, vec!['\u{201c}', '\u{201d}']);
    }

    #[test]
    fn test_phonemize_em_dash() {
        match phonemize("him \u{2014} courage", "en-us") {
            Ok(phonemes) => {
                assert!(
                    phonemes.contains('\u{2014}'),
                    "Em-dash should be preserved in phonemes: {}",
                    phonemes
                );
            }
            Err(KokoroError::EspeakNotFound) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_phonemize_newlines() {
        match phonemize("First paragraph.\n\nSecond paragraph.", "en-us") {
            Ok(phonemes) => {
                // Newlines should be converted to ". " producing period tokens
                assert!(!phonemes.is_empty());
            }
            Err(KokoroError::EspeakNotFound) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
