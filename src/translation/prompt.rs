pub const SYSTEM_PROMPT_TEMPLATE: &str = "You are a translator. Translate the following text to {target_language}. \
     Output only the translated text without any explanations. \
     Preserve the original formatting including blank lines and whitespace.";

#[allow(clippy::literal_string_with_formatting_args)]
pub fn build_system_prompt(target_language: &str) -> String {
    // {target_language} is a placeholder for string replacement, not a format argument
    SYSTEM_PROMPT_TEMPLATE.replace("{target_language}", target_language)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt() {
        let prompt = build_system_prompt("Japanese");
        assert!(prompt.contains("Japanese"));
        assert!(prompt.contains("Translate the following text"));
    }

    #[test]
    fn test_system_prompt_template_has_placeholder() {
        assert!(SYSTEM_PROMPT_TEMPLATE.contains("{target_language}"));
    }
}
