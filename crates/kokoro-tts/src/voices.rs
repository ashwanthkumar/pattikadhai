use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::KokoroError;

const EXPECTED_FLOATS: usize = 510 * 256; // 130560
const EXPECTED_BYTES: usize = EXPECTED_FLOATS * 4; // 522240

/// Voice embedding data: shape (510, 256) stored as flat f32 array.
/// To get embedding for token length N: `data[N * 256 .. (N+1) * 256]`
pub struct Voice {
    /// Flat f32 data, length = 510 * 256 = 130560
    data: Vec<f32>,
}

impl Voice {
    /// Get the style embedding for a given token count (before padding).
    /// Returns a slice of 256 f32 values.
    pub fn embedding(&self, token_len: usize) -> Result<&[f32], KokoroError> {
        if token_len >= 510 {
            return Err(KokoroError::Voice(format!(
                "Token length {} exceeds max 509",
                token_len
            )));
        }
        let start = token_len * 256;
        let end = start + 256;
        Ok(&self.data[start..end])
    }
}

/// Collection of all loaded voices.
pub struct VoiceStore {
    voices: HashMap<String, Voice>,
}

impl VoiceStore {
    /// Load voices from a directory of raw f32 `.bin` files.
    /// Each file is named `<voice_name>.bin` and contains 130560 little-endian f32 values
    /// (510 * 256, representing the voice embedding for each possible token length).
    pub fn load_dir(dir: &Path) -> Result<Self, KokoroError> {
        let entries = std::fs::read_dir(dir).map_err(|e| {
            KokoroError::Voice(format!("Failed to read voices directory {}: {}", dir.display(), e))
        })?;

        let mut voices = HashMap::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                KokoroError::Voice(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext != Some("bin") {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            if name.is_empty() {
                continue;
            }

            let raw = std::fs::read(&path).map_err(|e| {
                KokoroError::Voice(format!("Failed to read voice file {}: {}", path.display(), e))
            })?;

            if raw.len() != EXPECTED_BYTES {
                log::warn!(
                    "Skipping voice '{}': unexpected size {} (expected {})",
                    name, raw.len(), EXPECTED_BYTES
                );
                continue;
            }

            let data = parse_raw_f32(&raw);
            voices.insert(name, Voice { data });
        }

        if voices.is_empty() {
            return Err(KokoroError::Voice(format!(
                "No valid voice files found in {}",
                dir.display()
            )));
        }

        log::info!("Loaded {} voices from {}", voices.len(), dir.display());
        Ok(Self { voices })
    }

    /// Load voices from a .bin NPZ (NumPy ZIP) archive (legacy format).
    pub fn load_npz(path: &Path) -> Result<Self, KokoroError> {
        let file = std::fs::File::open(path).map_err(|e| {
            KokoroError::Voice(format!("Failed to open voices file {}: {}", path.display(), e))
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            KokoroError::Voice(format!("Failed to read NPZ archive: {}", e))
        })?;

        let mut voices = HashMap::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                KokoroError::Voice(format!("Failed to read NPZ entry {}: {}", i, e))
            })?;

            let name = entry
                .name()
                .strip_suffix(".npy")
                .unwrap_or(entry.name())
                .to_string();

            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| {
                KokoroError::Voice(format!("Failed to read NPZ entry data: {}", e))
            })?;

            let data = parse_npy_f32(&buf)?;

            if data.len() != EXPECTED_FLOATS {
                return Err(KokoroError::Voice(format!(
                    "Voice '{}' has unexpected size: {} (expected {})",
                    name,
                    data.len(),
                    EXPECTED_FLOATS
                )));
            }

            voices.insert(name, Voice { data });
        }

        log::info!("Loaded {} voices from NPZ", voices.len());
        Ok(Self { voices })
    }

    /// Get a voice by name.
    pub fn get(&self, name: &str) -> Result<&Voice, KokoroError> {
        self.voices
            .get(name)
            .ok_or_else(|| KokoroError::Voice(format!("Unknown voice: '{}'", name)))
    }

    /// List all available voice names.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.voices.keys().cloned().collect();
        names.sort();
        names
    }
}

/// Parse raw little-endian f32 bytes into a Vec<f32>.
fn parse_raw_f32(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// Parse a NumPy .npy file (v1.0/v2.0) containing float32 data.
fn parse_npy_f32(data: &[u8]) -> Result<Vec<f32>, KokoroError> {
    if data.len() < 10 || &data[0..6] != b"\x93NUMPY" {
        return Err(KokoroError::Voice("Invalid .npy magic bytes".to_string()));
    }

    let major = data[6];

    let (header_start, header_len) = if major == 1 {
        (10usize, u16::from_le_bytes([data[8], data[9]]) as usize)
    } else if major == 2 {
        (12usize, u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize)
    } else {
        return Err(KokoroError::Voice(format!("Unsupported .npy version: {}", major)));
    };

    let raw = &data[header_start + header_len..];

    if raw.len() % 4 != 0 {
        return Err(KokoroError::Voice("NPY data length not aligned to f32".to_string()));
    }

    Ok(parse_raw_f32(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npy_invalid_magic() {
        let data = b"not a numpy file";
        assert!(parse_npy_f32(data).is_err());
    }

    #[test]
    fn test_voice_embedding_bounds() {
        let data: Vec<f32> = (0..510 * 256).map(|i| i as f32).collect();
        let voice = Voice { data };

        let emb = voice.embedding(0).unwrap();
        assert_eq!(emb.len(), 256);
        assert_eq!(emb[0], 0.0);
        assert_eq!(emb[255], 255.0);

        let emb = voice.embedding(1).unwrap();
        assert_eq!(emb[0], 256.0);

        assert!(voice.embedding(510).is_err());
    }

    #[test]
    fn test_load_voices_npz() {
        let path = Path::new("/tmp/kokoro-inspect/voices-v1.0.bin");
        if !path.exists() {
            return;
        }

        let store = VoiceStore::load_npz(path).unwrap();
        let names = store.names();
        assert_eq!(names.len(), 54);
        assert!(names.contains(&"af_nova".to_string()));

        let voice = store.get("af_nova").unwrap();
        let emb = voice.embedding(10).unwrap();
        assert_eq!(emb.len(), 256);
    }

    #[test]
    fn test_parse_raw_f32() {
        let bytes: Vec<u8> = vec![0, 0, 128, 63, 0, 0, 0, 64]; // 1.0f32, 2.0f32
        let floats = parse_raw_f32(&bytes);
        assert_eq!(floats, vec![1.0, 2.0]);
    }
}
