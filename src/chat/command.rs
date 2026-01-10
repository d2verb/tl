use inquire::autocompletion::{Autocomplete, Replacement};

// Available slash commands: (command, description)
const SLASH_COMMANDS: &[(&str, &str)] = &[
    ("/config", "Show current configuration"),
    ("/help", "Show available commands"),
    ("/quit", "Exit chat mode"),
];

/// Slash command autocompleter
#[derive(Clone, Default)]
pub struct SlashCommandCompleter;

impl Autocomplete for SlashCommandCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        if !input.starts_with('/') {
            return Ok(vec![]);
        }

        let suggestions: Vec<String> = SLASH_COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(input))
            .map(|(cmd, desc)| format!("{cmd}  {desc}"))
            .collect();

        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, inquire::CustomUserError> {
        let replacement =
            highlighted_suggestion.map(|s| s.split_whitespace().next().unwrap_or("").to_string());
        Ok(replacement)
    }
}

/// Slash command types
#[derive(Debug, Clone)]
pub enum SlashCommand {
    Config,
    Help,
    Quit,
    Unknown(String),
}

/// Input types
#[derive(Debug)]
pub enum Input {
    Text(String),
    Command(SlashCommand),
    Empty,
}

pub fn parse_input(input: &str) -> Input {
    let input = input.trim();

    if input.is_empty() {
        return Input::Empty;
    }

    input
        .strip_prefix('/')
        .map_or_else(|| Input::Text(input.to_string()), parse_slash_command)
}

fn parse_slash_command(cmd: &str) -> Input {
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    match parts.first().copied() {
        Some("config") => Input::Command(SlashCommand::Config),
        Some("help") => Input::Command(SlashCommand::Help),
        Some("quit" | "exit" | "q") => Input::Command(SlashCommand::Quit),
        _ => Input::Command(SlashCommand::Unknown(parts.join(" "))),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_input() {
        assert!(matches!(parse_input(""), Input::Empty));
        assert!(matches!(parse_input("   "), Input::Empty));
    }

    #[test]
    fn test_parse_text_input() {
        match parse_input("Hello, world!") {
            Input::Text(text) => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected Input::Text"),
        }
    }

    #[test]
    fn test_parse_config_command() {
        assert!(matches!(
            parse_input("/config"),
            Input::Command(SlashCommand::Config)
        ));
    }

    #[test]
    fn test_parse_help_command() {
        assert!(matches!(
            parse_input("/help"),
            Input::Command(SlashCommand::Help)
        ));
    }

    #[test]
    fn test_parse_quit_commands() {
        assert!(matches!(
            parse_input("/quit"),
            Input::Command(SlashCommand::Quit)
        ));
        assert!(matches!(
            parse_input("/exit"),
            Input::Command(SlashCommand::Quit)
        ));
        assert!(matches!(
            parse_input("/q"),
            Input::Command(SlashCommand::Quit)
        ));
    }

    #[test]
    fn test_parse_unknown_command() {
        match parse_input("/unknown") {
            Input::Command(SlashCommand::Unknown(cmd)) => assert_eq!(cmd, "unknown"),
            _ => panic!("Expected Input::Command(SlashCommand::Unknown)"),
        }
    }

    // SlashCommandCompleter tests

    #[test]
    fn test_completer_no_suggestions_for_regular_text() {
        let mut completer = SlashCommandCompleter;
        let suggestions = completer.get_suggestions("hello").unwrap();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_completer_suggestions_for_slash() {
        let mut completer = SlashCommandCompleter;
        let suggestions = completer.get_suggestions("/").unwrap();
        assert_eq!(suggestions.len(), 3); // /config, /help, /quit
    }

    #[test]
    fn test_completer_suggestions_filter_by_prefix() {
        let mut completer = SlashCommandCompleter;

        let suggestions = completer.get_suggestions("/c").unwrap();
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].starts_with("/config"));

        let suggestions = completer.get_suggestions("/q").unwrap();
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].starts_with("/quit"));
    }

    #[test]
    fn test_completer_completion() {
        let mut completer = SlashCommandCompleter;
        let suggestion = "/config  Show current configuration".to_string();
        let completion = completer.get_completion("/c", Some(suggestion)).unwrap();
        assert_eq!(completion, Some("/config".to_string()));
    }

    #[test]
    fn test_completer_completion_none() {
        let mut completer = SlashCommandCompleter;
        let completion = completer.get_completion("/x", None).unwrap();
        assert!(completion.is_none());
    }
}
