use harper_core::linting::{Lint, LintKind, Linter, SpellCheck};
use harper_core::spell::Dictionary;
use harper_core::{Dialect, Document};

use crate::compound;

/// A spell-checker that understands German compound words.
///
/// Wraps the standard `SpellCheck` linter and filters out false positives
/// for valid compound words (e.g., "Dampfschiff" = "Dampf" + "Schiff").
pub struct GermanSpellCheck<T: Dictionary> {
    inner: SpellCheck<T>,
    dict: T,
}

impl<T: Dictionary + Clone> GermanSpellCheck<T> {
    pub fn new(dictionary: T) -> Self {
        Self {
            inner: SpellCheck::new(dictionary.clone(), Dialect::American),
            dict: dictionary,
        }
    }
}

impl<T: Dictionary + Clone> Linter for GermanSpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let lints = self.inner.lint(document);

        lints
            .into_iter()
            .filter(|lint| {
                if lint.lint_kind != LintKind::Spelling {
                    return true;
                }

                let word_str = document.get_span_content_str(&lint.span);
                !compound::is_valid_compound(&word_str, &self.dict)
            })
            .collect()
    }

    fn description(&self) -> &str {
        "Looks and provides corrections for misspelled words, with German compound word support."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::Document;
    use harper_core::linting::Linter;

    #[test]
    fn does_not_flag_compound_words() {
        let dict = crate::curated_dictionary();
        let mut linter = GermanSpellCheck::new(dict.clone());

        let doc = Document::new_plain_english_curated("Dampfschiff");
        let lints = linter.lint(&doc);

        // Dampfschiff = Dampf + Schiff, should not be flagged
        let spelling_lints: Vec<_> = lints
            .iter()
            .filter(|l| l.lint_kind == LintKind::Spelling)
            .collect();
        assert!(
            spelling_lints.is_empty(),
            "Dampfschiff should not be flagged as misspelled (it's a compound word)"
        );
    }

    #[test]
    fn still_flags_nonsense() {
        let dict = crate::curated_dictionary();
        let mut linter = GermanSpellCheck::new(dict.clone());

        let doc = Document::new_plain_english_curated("Xyzplqmwort");
        let lints = linter.lint(&doc);

        let spelling_lints: Vec<_> = lints
            .iter()
            .filter(|l| l.lint_kind == LintKind::Spelling)
            .collect();
        assert!(
            !spelling_lints.is_empty(),
            "Nonsense words should still be flagged"
        );
    }
}
