//! Language code validation and supported languages.

use anyhow::Result;

use crate::ui::Style;

/// Supported language codes (ISO 639-1) and their names.
pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("af", "Afrikaans"),
    ("am", "Amharic"),
    ("ar", "Arabic"),
    ("az", "Azerbaijani"),
    ("be", "Belarusian"),
    ("bg", "Bulgarian"),
    ("bn", "Bengali"),
    ("bs", "Bosnian"),
    ("ca", "Catalan"),
    ("cs", "Czech"),
    ("cy", "Welsh"),
    ("da", "Danish"),
    ("de", "German"),
    ("el", "Greek"),
    ("en", "English"),
    ("es", "Spanish"),
    ("et", "Estonian"),
    ("eu", "Basque"),
    ("fa", "Persian"),
    ("fi", "Finnish"),
    ("fil", "Filipino"),
    ("fr", "French"),
    ("ga", "Irish"),
    ("gl", "Galician"),
    ("gu", "Gujarati"),
    ("he", "Hebrew"),
    ("hi", "Hindi"),
    ("hr", "Croatian"),
    ("hu", "Hungarian"),
    ("hy", "Armenian"),
    ("id", "Indonesian"),
    ("is", "Icelandic"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ka", "Georgian"),
    ("kk", "Kazakh"),
    ("km", "Khmer"),
    ("kn", "Kannada"),
    ("ko", "Korean"),
    ("la", "Latin"),
    ("lo", "Lao"),
    ("lt", "Lithuanian"),
    ("lv", "Latvian"),
    ("mk", "Macedonian"),
    ("ml", "Malayalam"),
    ("mn", "Mongolian"),
    ("mr", "Marathi"),
    ("ms", "Malay"),
    ("mt", "Maltese"),
    ("my", "Myanmar (Burmese)"),
    ("ne", "Nepali"),
    ("nl", "Dutch"),
    ("no", "Norwegian"),
    ("pa", "Punjabi"),
    ("pl", "Polish"),
    ("ps", "Pashto"),
    ("pt", "Portuguese"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("si", "Sinhala"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("sq", "Albanian"),
    ("sr", "Serbian"),
    ("sv", "Swedish"),
    ("sw", "Swahili"),
    ("ta", "Tamil"),
    ("te", "Telugu"),
    ("th", "Thai"),
    ("tl", "Tagalog"),
    ("tr", "Turkish"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("uz", "Uzbek"),
    ("vi", "Vietnamese"),
    ("zh", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
];

/// Prints all supported language codes to stdout.
pub fn print_languages() {
    println!("{}", Style::header("Supported language codes (ISO 639-1)"));
    for (code, name) in SUPPORTED_LANGUAGES {
        println!("  {:5} {}", Style::code(code), Style::secondary(name));
    }
}

/// Validates that the given language code is supported.
///
/// # Errors
///
/// Returns an error if the language code is not in the supported list.
pub fn validate_language(lang: &str) -> Result<()> {
    if SUPPORTED_LANGUAGES.iter().any(|(code, _)| *code == lang) {
        Ok(())
    } else {
        anyhow::bail!(
            "Invalid language code: '{lang}'\n\n\
             Valid language codes (ISO 639-1): ja, en, zh, ko, fr, de, es, ...\n\
             Run 'tl languages' to see all supported codes."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_language_valid() {
        assert!(validate_language("ja").is_ok());
        assert!(validate_language("en").is_ok());
        assert!(validate_language("zh-TW").is_ok());
    }

    #[test]
    fn test_validate_language_invalid() {
        assert!(validate_language("invalid").is_err());
        assert!(validate_language("").is_err());
        assert!(validate_language("JP").is_err()); // Case sensitive
    }
}
