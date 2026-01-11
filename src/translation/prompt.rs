pub const SYSTEM_PROMPT_TEMPLATE: &str = "You are a translator. Translate the following text to {target_language}. \
     Output only the translated text without any explanations. \
     Preserve the original formatting including blank lines and whitespace.";

/// Builds the system prompt with optional style instructions.
#[allow(clippy::literal_string_with_formatting_args)]
pub fn build_system_prompt_with_style(target_language: &str, style: Option<&str>) -> String {
    // {target_language} is a placeholder for string replacement, not a format argument
    let base = SYSTEM_PROMPT_TEMPLATE.replace("{target_language}", target_language);
    match style {
        Some(style_prompt) => format!("{base} {style_prompt}"),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt_with_style_no_style() {
        let prompt = build_system_prompt_with_style("Japanese", None);
        assert!(prompt.contains("Japanese"));
        assert!(prompt.contains("Translate the following text"));
    }

    #[test]
    fn test_build_system_prompt_with_style_casual() {
        let prompt = build_system_prompt_with_style("Japanese", Some("Use a casual tone."));
        assert!(prompt.contains("Japanese"));
        assert!(prompt.contains("Use a casual tone."));
    }

    #[test]
    fn test_system_prompt_template_has_placeholder() {
        assert!(SYSTEM_PROMPT_TEMPLATE.contains("{target_language}"));
    }
}
