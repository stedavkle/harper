mod compound;
mod german_spell_check;

use std::sync::{Arc, LazyLock};

use harper_core::linting::LintGroup;
use harper_core::spell::{Dictionary, FstDictionary, MutableDictionary};

pub use german_spell_check::GermanSpellCheck;

const COMPRESSED_DICT: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/compressed-dictionary-de.zst"));
const ANNOTATIONS: &str = include_str!("../annotations_de.json");

fn decompress_dictionary() -> String {
    let decompressed = zstd::stream::decode_all(COMPRESSED_DICT)
        .expect("Failed to decompress German dictionary");
    String::from_utf8(decompressed).expect("German dictionary is valid UTF-8")
}

static DICT: LazyLock<Arc<FstDictionary>> = LazyLock::new(|| {
    let word_list = decompress_dictionary();
    let mutable = MutableDictionary::from_rune_files(&word_list, ANNOTATIONS)
        .expect("Failed to load German dictionary");
    Arc::new(mutable.into())
});

/// Get the curated German dictionary.
pub fn curated_dictionary() -> Arc<FstDictionary> {
    (*DICT).clone()
}

/// Check if a word is a valid German compound by attempting to decompose it.
pub fn is_valid_compound(word: &str, dict: &impl Dictionary) -> bool {
    compound::is_valid_compound(word, dict)
}

/// Create a German lint group with compound-aware spell-checking.
///
/// Includes spell-checking (with compound word support), repeated words,
/// spacing, and basic formatting. German-specific grammar rules
/// (das/dass, seit/seid, noun capitalization) will be added in a future phase.
pub fn lint_group_german(dictionary: Arc<impl Dictionary + Clone + 'static>) -> LintGroup {
    // Start with the language-agnostic group (which includes SpellCheck and formatting rules)
    let mut out = LintGroup::new_language_agnostic(dictionary.clone());

    // Replace the standard SpellCheck with our compound-aware German version
    out.replace("SpellCheck", GermanSpellCheck::new(dictionary));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_loads() {
        let dict = curated_dictionary();
        assert!(dict.contains_word(&['H', 'a', 'u', 's']));
    }

    #[test]
    fn contains_common_words() {
        let dict = curated_dictionary();

        let test_words: &[&str] = &[
            "Haus", "gehen", "ist", "haben", "der", "die", "das", "und", "nicht",
        ];

        for word_str in test_words {
            let word: Vec<char> = word_str.chars().collect();
            assert!(
                dict.contains_word(&word),
                "Dictionary should contain '{word_str}'"
            );
        }
    }

    #[test]
    fn contains_inflected_forms() {
        let dict = curated_dictionary();

        let test_words: &[&str] = &[
            "Häuser",   // plural of Haus
            "ging",     // past tense of gehen
            "gegangen", // past participle of gehen
        ];

        for word_str in test_words {
            let word: Vec<char> = word_str.chars().collect();
            assert!(
                dict.contains_word(&word),
                "Dictionary should contain inflected form '{word_str}'"
            );
        }
    }

    #[test]
    fn rejects_nonsense() {
        let dict = curated_dictionary();
        assert!(!dict.contains_word(&['X', 'y', 'z', 'p', 'l', 'q', 'm']));
    }
}
