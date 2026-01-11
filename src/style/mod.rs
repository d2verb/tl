//! Translation style management.
//!
//! Provides preset and custom translation styles to control the tone
//! and style of translations.

use std::collections::HashMap;

use crate::config::CustomStyle;

/// A preset translation style (hardcoded, not modifiable by users).
#[derive(Debug, Clone)]
pub struct PresetStyle {
    /// The style key (e.g., "casual", "formal").
    pub key: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Prompt text appended to the system prompt.
    pub prompt: &'static str,
}

/// All available preset styles.
pub const PRESETS: &[PresetStyle] = &[
    PresetStyle {
        key: "casual",
        description: "Casual, conversational tone",
        prompt: "Use a casual, friendly, conversational tone.",
    },
    PresetStyle {
        key: "formal",
        description: "Formal, business-appropriate",
        prompt: "Use a formal, polite, business-appropriate tone.",
    },
    PresetStyle {
        key: "literal",
        description: "Literal, close to source",
        prompt: "Translate as literally as possible while remaining grammatical.",
    },
    PresetStyle {
        key: "natural",
        description: "Natural, idiomatic",
        prompt: "Translate naturally, prioritizing idiomatic expressions over literal accuracy.",
    },
];

/// Resolved style information.
#[derive(Debug, Clone)]
pub enum ResolvedStyle {
    /// A preset style.
    Preset(&'static PresetStyle),
    /// A custom user-defined style.
    Custom { key: String, prompt: String },
}

impl ResolvedStyle {
    /// Returns the prompt text for this style.
    pub fn prompt(&self) -> &str {
        match self {
            Self::Preset(preset) => preset.prompt,
            Self::Custom { prompt, .. } => prompt,
        }
    }

    /// Returns the key for this style.
    pub fn key(&self) -> &str {
        match self {
            Self::Preset(preset) => preset.key,
            Self::Custom { key, .. } => key,
        }
    }
}

/// Looks up a preset style by key.
pub fn get_preset(key: &str) -> Option<&'static PresetStyle> {
    PRESETS.iter().find(|p| p.key == key)
}

/// Returns true if the key is a preset style.
pub fn is_preset(key: &str) -> bool {
    get_preset(key).is_some()
}

/// Returns custom style keys sorted alphabetically.
#[allow(clippy::implicit_hasher)]
pub fn sorted_custom_keys(styles: &HashMap<String, CustomStyle>) -> Vec<&String> {
    let mut keys: Vec<_> = styles.keys().collect();
    keys.sort();
    keys
}

/// Resolves a style key to a `ResolvedStyle`.
///
/// First checks presets, then custom styles.
/// Returns an error if the style is not found.
#[allow(clippy::implicit_hasher)]
pub fn resolve_style(
    key: &str,
    custom_styles: &HashMap<String, CustomStyle>,
) -> Result<ResolvedStyle, StyleError> {
    // Check presets first
    if let Some(preset) = get_preset(key) {
        return Ok(ResolvedStyle::Preset(preset));
    }

    // Check custom styles
    if let Some(custom) = custom_styles.get(key) {
        return Ok(ResolvedStyle::Custom {
            key: key.to_string(),
            prompt: custom.prompt.clone(),
        });
    }

    let custom_keys: Vec<String> = sorted_custom_keys(custom_styles)
        .into_iter()
        .cloned()
        .collect();
    Err(StyleError::NotFound {
        key: key.to_string(),
        custom_keys,
    })
}

/// Style-related errors.
#[derive(Debug, Clone)]
pub enum StyleError {
    /// Style not found. Contains the key and list of custom style keys.
    NotFound {
        key: String,
        custom_keys: Vec<String>,
    },
    /// Attempted to modify a preset style.
    PresetImmutable(String),
    /// Style key already exists.
    AlreadyExists(String),
    /// Invalid style key format.
    InvalidKey(String),
}

impl std::fmt::Display for StyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { key, custom_keys } => {
                let preset_keys: Vec<_> = PRESETS.iter().map(|p| p.key).collect();
                let mut all_keys: Vec<&str> = preset_keys;
                all_keys.extend(custom_keys.iter().map(String::as_str));
                write!(
                    f,
                    "Style '{key}' not found\n\nAvailable styles: {}",
                    all_keys.join(", ")
                )
            }
            Self::PresetImmutable(key) => {
                write!(f, "Cannot modify preset style '{key}'")
            }
            Self::AlreadyExists(key) => {
                write!(f, "Style '{key}' already exists")
            }
            Self::InvalidKey(key) => {
                write!(
                    f,
                    "Invalid style key '{key}': must start with a letter and contain only alphanumeric characters and underscores"
                )
            }
        }
    }
}

impl std::error::Error for StyleError {}

/// Validates a custom style key.
///
/// Keys must start with a letter, contain only alphanumeric characters and underscores,
/// and cannot conflict with presets.
pub fn validate_custom_key(key: &str) -> Result<(), StyleError> {
    // Check empty first
    if key.is_empty() {
        return Err(StyleError::InvalidKey(key.to_string()));
    }

    // Must start with a letter
    if !key.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return Err(StyleError::InvalidKey(key.to_string()));
    }

    // All characters must be alphanumeric or underscore
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(StyleError::InvalidKey(key.to_string()));
    }

    // Cannot conflict with presets
    if is_preset(key) {
        return Err(StyleError::PresetImmutable(key.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_count() {
        assert_eq!(PRESETS.len(), 4);
    }

    #[test]
    fn test_get_preset_exists() {
        assert!(get_preset("casual").is_some());
        assert!(get_preset("formal").is_some());
        assert!(get_preset("literal").is_some());
        assert!(get_preset("natural").is_some());
    }

    #[test]
    fn test_get_preset_not_exists() {
        assert!(get_preset("nonexistent").is_none());
    }

    #[test]
    fn test_is_preset() {
        assert!(is_preset("casual"));
        assert!(!is_preset("my_custom"));
    }

    #[test]
    fn test_sorted_custom_keys() {
        let mut styles = HashMap::new();
        styles.insert(
            "zebra".to_string(),
            CustomStyle {
                description: "z desc".to_string(),
                prompt: "z prompt".to_string(),
            },
        );
        styles.insert(
            "alpha".to_string(),
            CustomStyle {
                description: "a desc".to_string(),
                prompt: "a prompt".to_string(),
            },
        );
        styles.insert(
            "beta".to_string(),
            CustomStyle {
                description: "b desc".to_string(),
                prompt: "b prompt".to_string(),
            },
        );

        let keys = sorted_custom_keys(&styles);
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sorted_custom_keys_empty() {
        let styles: HashMap<String, CustomStyle> = HashMap::new();
        let keys = sorted_custom_keys(&styles);
        assert!(keys.is_empty());
    }

    #[test]
    fn test_resolve_style_preset() {
        let custom: HashMap<String, CustomStyle> = HashMap::new();
        let resolved = resolve_style("casual", &custom);
        assert!(resolved.is_ok());
        assert_eq!(
            resolved.as_ref().ok().map(ResolvedStyle::key),
            Some("casual")
        );
    }

    #[test]
    fn test_resolve_style_custom() {
        let mut custom = HashMap::new();
        custom.insert(
            "my_style".to_string(),
            CustomStyle {
                description: "My description".to_string(),
                prompt: "My custom prompt".to_string(),
            },
        );

        let resolved = resolve_style("my_style", &custom);
        assert!(resolved.is_ok());
        assert_eq!(
            resolved.as_ref().ok().map(ResolvedStyle::key),
            Some("my_style")
        );
        assert_eq!(
            resolved.as_ref().ok().map(ResolvedStyle::prompt),
            Some("My custom prompt")
        );
    }

    #[test]
    fn test_resolve_style_not_found() {
        let custom: HashMap<String, CustomStyle> = HashMap::new();
        let resolved = resolve_style("nonexistent", &custom);
        assert!(resolved.is_err());
    }

    #[test]
    fn test_validate_custom_key_valid() {
        assert!(validate_custom_key("my_style").is_ok());
        assert!(validate_custom_key("style123").is_ok());
        assert!(validate_custom_key("MyStyle").is_ok());
    }

    #[test]
    fn test_validate_custom_key_preset() {
        let result = validate_custom_key("casual");
        assert!(matches!(result, Err(StyleError::PresetImmutable(_))));
    }

    #[test]
    fn test_validate_custom_key_invalid() {
        assert!(validate_custom_key("").is_err());
        assert!(validate_custom_key("123start").is_err());
        assert!(validate_custom_key("has-dash").is_err());
        assert!(validate_custom_key("has space").is_err());
    }

    #[test]
    fn test_validate_custom_key_underscore_prefix() {
        // Underscore at start is invalid (must start with letter)
        assert!(matches!(
            validate_custom_key("_style"),
            Err(StyleError::InvalidKey(_))
        ));
    }

    #[test]
    fn test_validate_custom_key_single_letter() {
        // Single letter is valid
        assert!(validate_custom_key("a").is_ok());
    }

    // StyleError display tests

    #[test]
    fn test_style_error_not_found_display_shows_presets() {
        let error = StyleError::NotFound {
            key: "unknown".to_string(),
            custom_keys: vec![],
        };
        let msg = error.to_string();
        assert!(msg.contains("Style 'unknown' not found"));
        assert!(msg.contains("casual"));
        assert!(msg.contains("formal"));
        assert!(msg.contains("literal"));
        assert!(msg.contains("natural"));
    }

    #[test]
    fn test_style_error_not_found_display_includes_custom() {
        let error = StyleError::NotFound {
            key: "unknown".to_string(),
            custom_keys: vec!["my_custom".to_string(), "another".to_string()],
        };
        let msg = error.to_string();
        assert!(msg.contains("Available styles:"));
        assert!(msg.contains("casual"));
        assert!(msg.contains("my_custom"));
        assert!(msg.contains("another"));
    }

    #[test]
    fn test_style_error_preset_immutable_display() {
        let error = StyleError::PresetImmutable("casual".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Cannot modify preset style 'casual'"));
    }

    #[test]
    fn test_style_error_already_exists_display() {
        let error = StyleError::AlreadyExists("my_style".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Style 'my_style' already exists"));
    }

    #[test]
    fn test_style_error_invalid_key_display() {
        let error = StyleError::InvalidKey("123bad".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Invalid style key '123bad'"));
        assert!(msg.contains("must start with a letter"));
    }

    #[test]
    fn test_resolve_style_error_includes_custom_keys() {
        let mut custom = HashMap::new();
        custom.insert(
            "my_style".to_string(),
            CustomStyle {
                description: "desc".to_string(),
                prompt: "prompt".to_string(),
            },
        );

        let result = resolve_style("nonexistent", &custom);
        match result {
            Err(StyleError::NotFound { custom_keys, .. }) => {
                assert!(custom_keys.contains(&"my_style".to_string()));
            }
            _ => panic!("Expected StyleError::NotFound"),
        }
    }
}
