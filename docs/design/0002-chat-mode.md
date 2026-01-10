# Mini Design Doc: tl chat - Interactive Translation Mode

*   **Author:** d2verb
*   **Status:** Implemented
*   **Date:** 2026-01-08

## 1. Abstract

The `tl chat` command provides an interactive chat-style translation feature. It adopts a modern CLI UI like Claude Code, codex-cli, and gemini-cli, and also supports configuration operations via slash commands.

## 2. Goals & Non-Goals

### Goals
*   Translation in an interactive chat format
*   Modern CLI UI (prompts, color-coding, etc.)
*   Configuration display via slash commands

### Non-Goals
*   Persistence of conversation history (v1 only within session)
*   Multi-turn translation (continuous translation considering context)
*   Plugin system
*   Input history feature (out of scope for v1)

## 3. Context & Problem Statement

The current `tl` command is specialized for one-shot translations, and when translating multiple texts consecutively, the command must be executed each time. By adding an interactive mode, the workflow for consecutive translations can be improved, and checking/changing configurations becomes easier.

## 4. Proposed Design

### 4.1 CLI Interface

```bash
# Start chat mode
tl chat

# Start with options
tl chat -t ja -e http://localhost:11434 -m llama3.2
```

### 4.2 UI Design

```
$ tl chat -t ja

tl v0.1.0 - Interactive Translation Mode
Type text to translate, or use /commands. Press Ctrl+C to exit.

> Hello, world!
Hello, world!

> How are you?
How are you?

> /config
Current configuration:
  provider = ollama
  model    = llama3.2
  to       = ja
  endpoint = http://localhost:11434

> /help
Available commands:
  /config  Show current configuration
  /help    Show this help
  /quit    Exit chat mode

> /quit
Goodbye!
```

### 4.3 Slash Commands

| Command | Description |
|:--------|:------------|
| `/config` | Show current configuration |
| `/help` | Show help |
| `/quit` or `/exit` | Exit chat mode |

### 4.4 Session Configuration

Configuration loading priority at startup:
1. CLI arguments (highest priority)
2. config.toml
3. Default values (none → error)

Configuration is immutable during session. To change settings, restart with different CLI arguments.

### 4.5 UI Components

```
┌─────────────────────────────────────────────────────────┐
│  tl v0.1.0 - Interactive Translation Mode               │
│  Type text to translate, or use /commands.              │
│  Press Ctrl+C to exit                                 
├─────────────────────────────────────────────────────────┤
│                                                         │
│  > [user input]                                         │
│  [translated output]                                    │
│                                                         │
│  > [user input]                                         │
│  ⠋ Translating...                                       │
│  [streaming output...]                                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**UI elements:**
*   Header: version, mode, basic operation instructions
*   Prompt: waiting for input with `> `
*   Output: translation result (streaming)
*   Spinner: display while translating
*   Color coding:
    *   Prompt `>` : green
    *   Command `/...` : blue
    *   Error: red
    *   Translation result: default

### 4.6 Module Structure

```
src/
├── cli/
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── translate.rs
│   │   └── chat.rs          # new: chat mode
│   └── ...
├── chat/                     # new: chat functionality
│   ├── mod.rs
│   ├── session.rs            # session management
│   ├── command.rs            # slash command handling
│   └── ui.rs                 # UI rendering
└── ...
```

### 4.7 Dependencies

Additional crate needed:

```toml
[dependencies]
inquire = "0.9"              # interactive prompt (Unicode support)
```

### 4.8 Data Structures

```rust
/// Configuration for chat session
pub struct SessionConfig {
    pub provider_name: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub to: String,
}

/// Slash command
pub enum SlashCommand {
    Config,
    Help,
    Quit,
    Unknown(String),
}

/// Input type
pub enum Input {
    Text(String),           // text to translate
    Command(SlashCommand),  // slash command
    Empty,                  // empty input
}
```

### 4.9 Command Parsing

```rust
fn parse_input(input: &str) -> Input {
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
```

### 4.10 Main Loop

```rust
use inquire::Text;

async fn run_chat(initial_config: SessionConfig) -> Result<()> {
    print_header();

    let mut config = initial_config;

    loop {
        let input = Text::new("")
            .with_help_message("Type text to translate, /help for commands, Ctrl+C to quit")
            .prompt();

        match input {
            Ok(line) => {
                match parse_input(&line) {
                    Input::Empty => {}
                    Input::Command(cmd) => {
                        if !handle_command(&mut config, cmd) {
                            break;  // /quit
                        }
                    }
                    Input::Text(text) => {
                        translate_and_print(&config, &text).await?;
                    }
                }
            }
            Err(inquire::InquireError::OperationCanceled
                | inquire::InquireError::OperationInterrupted) => break,  // Ctrl+C or Ctrl+D
            Err(e) => return Err(e.into()),
        }
    }

    println!("Goodbye!");
    Ok(())
}
```

## 5. Implementation Plan

1.  **Phase 1: Basic structure**
    *   Add `tl chat` subcommand
    *   Implement `SessionConfig`
    *   Basic REPL loop

2.  **Phase 2: Slash commands**
    *   Implement command parser
    *   Implement `/config`, `/help`, `/quit`

3.  **Phase 3: UI improvements**
    *   Color-coded display
    *   Integration with streaming translation

4.  **Phase 4: Quality improvements**
    *   Error handling
    *   Add tests

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| Streaming output conflicts with prompt | Low | Hide the prompt while translating |
| Handling of long text input | Low | Consider multi-line input as a future extension |

## 7. Testing & Verification

### Unit Tests
*   Parsing of slash commands
*   Initialization of `SessionConfig`
*   Classification of input (text vs command)

### Integration Tests
*   Start and end of chat session
*   Execution of translation

### Manual Testing
*   UI verification on various terminal emulators
*   Verification of Ctrl+C behavior

## 8. Alternatives Considered

### Input method

| Option | Pros | Cons | Decision |
| :--- | :--- | :--- | :--- |
| inquire | Simple API, modern UI, Unicode support | No history feature | **Adopted** |
| rustyline | History, completion, line editing | API a bit complex | Not adopted |
| Standard input only | Simple | No editing, poor UX | Not adopted |
| ratatui (TUI) | Rich UI | Complex, over‑engineering | Not adopted |

### Command prefix

| Option | Pros | Cons | Decision |
| :--- | :--- | :--- | :--- |
| `/command` | Intuitive, similar to other tools | Needs handling when translation text starts with `/` | **Adopted** |
| `:command` | Vim‑style | Not widely used | Not adopted |
| `!command` | Shell‑style | `!` is common in translation text | Not adopted |

---

## Appendix: Future Enhancements (Out of Scope for v1)

*   Support for multi-line input (here‑document style)
*   Persistence of input history
*   Command completion
*   Custom prompt
*   Multi‑turn translation (context retention)
