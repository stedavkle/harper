use decompound::{DecompositionOptions, decompound};
use harper_core::spell::Dictionary;

/// Check if a word is a valid German compound by attempting to decompose it
/// into dictionary words.
pub fn is_valid_compound(word: &str, dict: &impl Dictionary) -> bool {
    if word.len() < 4 {
        return false;
    }

    let options = DecompositionOptions::TRY_TITLECASE_SUFFIX;

    let result = decompound(word, &|part: &str| {
        let chars: Vec<char> = part.chars().collect();
        dict.contains_word(&chars)
    }, options);

    match result {
        Ok(parts) => parts.len() >= 2,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compound_splitting() {
        let dict = crate::curated_dictionary();

        // "Dampfschiff" = "Dampf" + "Schiff"
        assert!(
            is_valid_compound("Dampfschiff", &*dict),
            "Dampfschiff should be recognized as a compound"
        );
    }

    #[test]
    fn rejects_non_compound() {
        let dict = crate::curated_dictionary();
        assert!(!is_valid_compound("Xyzplqm", &*dict));
    }
}
