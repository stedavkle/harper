use serde::{Deserialize, Serialize};

/// The language Harper is checking.
///
/// Currently supports English and German. Defaults to English.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    German,
}

impl Language {
    /// Parse a language from a string abbreviation or name.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "english" | "en" | "en-us" | "en-gb" | "en-au" | "en-ca" | "en-in" => {
                Some(Language::English)
            }
            "german" | "de" | "de-de" | "de-at" | "de-ch" | "deutsch" => Some(Language::German),
            _ => None,
        }
    }
}
