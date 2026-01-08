/// Slash command types
#[derive(Debug, Clone)]
pub enum SlashCommand {
    Config,
    Set { key: String, value: String },
    Clear,
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

    match parts.as_slice() {
        ["config"] => Input::Command(SlashCommand::Config),
        ["set", key, value] => Input::Command(SlashCommand::Set {
            key: (*key).to_string(),
            value: (*value).to_string(),
        }),
        ["clear"] => Input::Command(SlashCommand::Clear),
        ["help"] => Input::Command(SlashCommand::Help),
        ["quit" | "exit" | "q"] => Input::Command(SlashCommand::Quit),
        _ => Input::Command(SlashCommand::Unknown(parts.join(" "))),
    }
}

#[cfg(test)]
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
    fn test_parse_set_command() {
        match parse_input("/set to en") {
            Input::Command(SlashCommand::Set { key, value }) => {
                assert_eq!(key, "to");
                assert_eq!(value, "en");
            }
            _ => panic!("Expected Input::Command(SlashCommand::Set)"),
        }
    }

    #[test]
    fn test_parse_clear_command() {
        assert!(matches!(
            parse_input("/clear"),
            Input::Command(SlashCommand::Clear)
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
}
