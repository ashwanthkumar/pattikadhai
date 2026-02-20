use std::collections::HashMap;
use std::sync::LazyLock;

/// Kokoro-82M vocabulary: IPA characters → sparse token IDs (0–177).
/// Extracted from the model's tokenizer.json.
static VOCAB: LazyLock<HashMap<char, i64>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(90);
    m.insert('$', 0);
    m.insert(';', 1);
    m.insert(':', 2);
    m.insert(',', 3);
    m.insert('.', 4);
    m.insert('!', 5);
    m.insert('?', 6);
    m.insert('\u{2014}', 9);   // —
    m.insert('\u{2026}', 10);  // …
    m.insert('"', 11);
    m.insert('(', 12);
    m.insert(')', 13);
    m.insert('\u{201c}', 14);  // "
    m.insert('\u{201d}', 15);  // "
    m.insert(' ', 16);
    m.insert('\u{0303}', 17);  // combining tilde
    m.insert('\u{02a3}', 18);  // dz
    m.insert('\u{02a5}', 19);  // dʑ
    m.insert('\u{02a6}', 20);  // ts
    m.insert('\u{02a8}', 21);  // tɕ
    m.insert('\u{1d5d}', 22);  // ᵝ
    m.insert('\u{ab67}', 23);  // ꭧ
    m.insert('A', 24);
    m.insert('I', 25);
    m.insert('O', 31);
    m.insert('Q', 33);
    m.insert('S', 35);
    m.insert('T', 36);
    m.insert('W', 39);
    m.insert('Y', 41);
    m.insert('\u{1d4a}', 42);  // ᵊ
    m.insert('a', 43);
    m.insert('b', 44);
    m.insert('c', 45);
    m.insert('d', 46);
    m.insert('e', 47);
    m.insert('f', 48);
    m.insert('h', 50);
    m.insert('i', 51);
    m.insert('j', 52);
    m.insert('k', 53);
    m.insert('l', 54);
    m.insert('m', 55);
    m.insert('n', 56);
    m.insert('o', 57);
    m.insert('p', 58);
    m.insert('q', 59);
    m.insert('r', 60);
    m.insert('s', 61);
    m.insert('t', 62);
    m.insert('u', 63);
    m.insert('v', 64);
    m.insert('w', 65);
    m.insert('x', 66);
    m.insert('y', 67);
    m.insert('z', 68);
    m.insert('\u{0251}', 69);  // ɑ
    m.insert('\u{0250}', 70);  // ɐ
    m.insert('\u{0252}', 71);  // ɒ
    m.insert('\u{00e6}', 72);  // æ
    m.insert('\u{03b2}', 75);  // β
    m.insert('\u{0254}', 76);  // ɔ
    m.insert('\u{0255}', 77);  // ɕ
    m.insert('\u{00e7}', 78);  // ç
    m.insert('\u{0256}', 80);  // ɖ
    m.insert('\u{00f0}', 81);  // ð
    m.insert('\u{02a4}', 82);  // dʒ
    m.insert('\u{0259}', 83);  // ə
    m.insert('\u{025a}', 85);  // ɚ
    m.insert('\u{025b}', 86);  // ɛ
    m.insert('\u{025c}', 87);  // ɜ
    m.insert('\u{025f}', 90);  // ɟ
    m.insert('\u{0261}', 92);  // ɡ
    m.insert('\u{0265}', 99);  // ɥ
    m.insert('\u{0268}', 101); // ɨ
    m.insert('\u{026a}', 102); // ɪ
    m.insert('\u{029d}', 103); // ʝ
    m.insert('\u{026f}', 110); // ɯ
    m.insert('\u{0270}', 111); // ɰ
    m.insert('\u{014b}', 112); // ŋ
    m.insert('\u{0273}', 113); // ɳ
    m.insert('\u{0272}', 114); // ɲ
    m.insert('\u{0274}', 115); // ɴ
    m.insert('\u{00f8}', 116); // ø
    m.insert('\u{0278}', 118); // ɸ
    m.insert('\u{03b8}', 119); // θ
    m.insert('\u{0153}', 120); // œ
    m.insert('\u{0279}', 123); // ɹ
    m.insert('\u{027e}', 125); // ɾ
    m.insert('\u{027b}', 126); // ɻ
    m.insert('\u{0281}', 128); // ʁ
    m.insert('\u{027d}', 129); // ɽ
    m.insert('\u{0282}', 130); // ʂ
    m.insert('\u{0283}', 131); // ʃ
    m.insert('\u{0288}', 132); // ʈ
    m.insert('\u{02a7}', 133); // tʃ
    m.insert('\u{028a}', 135); // ʊ
    m.insert('\u{028b}', 136); // ʋ
    m.insert('\u{028c}', 138); // ʌ
    m.insert('\u{0263}', 139); // ɣ
    m.insert('\u{0264}', 140); // ɤ
    m.insert('\u{03c7}', 142); // χ
    m.insert('\u{028e}', 143); // ʎ
    m.insert('\u{0292}', 147); // ʒ
    m.insert('\u{0294}', 148); // ʔ
    m.insert('\u{02c8}', 156); // ˈ
    m.insert('\u{02cc}', 157); // ˌ
    m.insert('\u{02d0}', 158); // ː
    m.insert('\u{02b0}', 162); // ʰ
    m.insert('\u{02b2}', 164); // ʲ
    m.insert('\u{2193}', 169); // ↓
    m.insert('\u{2192}', 171); // →
    m.insert('\u{2197}', 172); // ↗
    m.insert('\u{2198}', 173); // ↘
    m.insert('\u{1d7b}', 177); // ᵻ
    m
});

/// Maximum number of phoneme tokens before padding.
pub const MAX_PHONEME_LEN: usize = 510;

/// Convert IPA phoneme string to token IDs.
/// Characters not in the vocabulary are silently skipped.
pub fn tokenize(phonemes: &str) -> Vec<i64> {
    phonemes
        .chars()
        .filter_map(|c| VOCAB.get(&c).copied())
        .take(MAX_PHONEME_LEN)
        .collect()
}

/// Pad token IDs with 0 at start and end: [0, ...tokens, 0]
pub fn pad_tokens(tokens: &[i64]) -> Vec<i64> {
    let mut padded = Vec::with_capacity(tokens.len() + 2);
    padded.push(0);
    padded.extend_from_slice(tokens);
    padded.push(0);
    padded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocab_size() {
        assert_eq!(VOCAB.len(), 115);
    }

    #[test]
    fn test_basic_tokenize() {
        let tokens = tokenize("hello");
        assert_eq!(tokens, vec![50, 47, 54, 54, 57]); // h=50, e=47, l=54, l=54, o=57
    }

    #[test]
    fn test_unknown_chars_skipped() {
        let tokens = tokenize("h€llo"); // € is not in vocab
        assert_eq!(tokens, vec![50, 54, 54, 57]);
    }

    #[test]
    fn test_pad_tokens() {
        let tokens = vec![50, 47, 54, 54, 57];
        let padded = pad_tokens(&tokens);
        assert_eq!(padded, vec![0, 50, 47, 54, 54, 57, 0]);
    }

    #[test]
    fn test_max_length() {
        let long_input: String = std::iter::repeat('a').take(600).collect();
        let tokens = tokenize(&long_input);
        assert_eq!(tokens.len(), MAX_PHONEME_LEN);
    }

    #[test]
    fn test_ipa_chars() {
        // Test IPA characters like ə (schwa)
        let tokens = tokenize("ə");
        assert_eq!(tokens, vec![83]);
    }

    #[test]
    fn test_punctuation() {
        let tokens = tokenize("hello, world!");
        // h=50, e=47, l=54, l=54, o=57, ,=3, ' '=16, w=65, o=57, r=60, l=54, d=46, !=5
        assert_eq!(tokens, vec![50, 47, 54, 54, 57, 3, 16, 65, 57, 60, 54, 46, 5]);
    }
}
