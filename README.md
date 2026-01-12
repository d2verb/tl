# tl – streaming, cached translation CLI

[![CI](https://github.com/d2verb/tl/actions/workflows/ci.yml/badge.svg)](https://github.com/d2verb/tl/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tl-cli.svg)](https://crates.io/crates/tl-cli)
[![docs.rs](https://docs.rs/tl-cli/badge.svg)](https://docs.rs/tl-cli)

`tl` is a small CLI that streams translations through any OpenAI-compatible endpoint (local or remote). Configure multiple providers with their own endpoints, API keys, and models, then switch between them as needed.

## Install

### Using cargo (recommended)

```sh
cargo install tl-cli
```

### Using installer scripts

**macOS/Linux:**
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/d2verb/tl/releases/latest/download/tl-cli-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/d2verb/tl/releases/latest/download/tl-cli-installer.ps1 | iex
```

### From source

```sh
git clone https://github.com/d2verb/tl.git
cd tl
cargo install --path .
```

## Getting Started

### 1. Add a provider

```sh
tl providers add
```

Follow the prompts to configure your first provider. Example for local Ollama:

```
Provider name: ollama
Endpoint URL: http://localhost:11434
API key method: None (no auth required)
Models: gemma3:12b, llama3.2
```

### 2. Set defaults

```sh
tl configure
```

Select your default provider, model, and target language.

### 3. Translate

```sh
echo "Hello, world!" | tl
```

You should see the translation stream in real-time.

## Usage

```sh
tl ./notes.md                       # translate a file
cat report.md | tl                   # translate stdin
tl --to ja ./notes.md                # override target language
tl --provider openrouter ./notes.md  # use a specific provider
tl --model gpt-4o ./notes.md         # use a specific model
tl --style casual ./notes.md         # use a translation style
tl --no-cache ./notes.md             # bypass cache
tl -w ./notes.md                     # overwrite file with translation
```

Translations are cached (keyed on input, language, model, endpoint, and prompt) so rerunning the same source is fast and cheap.

## Managing Providers

```sh
tl providers                        # list all providers
tl providers add                    # add a new provider interactively
tl providers edit <name>            # edit an existing provider
tl providers remove <name>          # remove a provider
```

## Translation Styles

Styles control the tone and manner of translations. Four preset styles are available:

| Style | Description |
|-------|-------------|
| `casual` | Casual, conversational tone |
| `formal` | Formal, business-appropriate |
| `literal` | Literal, close to source |
| `natural` | Natural, idiomatic expressions |

```sh
tl styles                           # list all styles (presets + custom)
tl styles show <name>               # show style details (description + prompt)
tl styles add                       # add a custom style interactively
tl styles edit <name>               # edit a custom style
tl styles remove <name>             # remove a custom style
```

Use styles with the `--style` option:

```sh
tl --style formal ./email.md
tl --style casual ./chat.txt
```

## Chat Mode

For interactive translation sessions:

```sh
tl chat                              # start with config defaults
tl chat --to ja                      # override target language
tl chat --provider openrouter        # use a specific provider
```

Type text and press Enter to translate. Available commands:

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/config` | Show current configuration |
| `/set style <name>` | Set translation style (or clear with `/set style`) |
| `/set to <lang>` | Change target language |
| `/set model <name>` | Change model |
| `/quit` | Exit chat mode |

## Configuration Reference

Settings are stored in `~/.config/tl/config.toml`:

```toml
[tl]
provider = "ollama"
model = "gemma3:12b"
to = "ja"
style = "casual"                     # optional default style

[providers.ollama]
endpoint = "http://localhost:11434"
models = ["gemma3:12b", "llama3.2"]

[providers.openrouter]
endpoint = "https://openrouter.ai/api"
api_key_env = "OPENROUTER_API_KEY"
models = ["anthropic/claude-3.5-sonnet", "openai/gpt-4o"]

[styles.ojisan]
description = "Middle-aged man texting style"
prompt = "Translate with excessive emoji, overly familiar tone, and random punctuation."

[styles.keigo]
description = "Polite Japanese honorifics"
prompt = "Translate using polite Japanese with appropriate keigo (honorific language)."
```

### Provider options

- `endpoint` (required) – OpenAI-compatible API endpoint
- `api_key_env` (optional) – environment variable name for API key
- `api_key` (optional) – API key in config (not recommended)
- `models` (optional) – available models for this provider

### Custom style options

- `description` (required) – short description shown in `tl styles` list
- `prompt` (required) – instruction appended to the system prompt for the LLM

CLI options always override config file values.

## Troubleshooting

- Run `tl languages` to see supported ISO 639-1 language codes.
- Pressing `Ctrl+C` while streaming aborts without polluting the cache.
- Use `--no-cache` to force a fresh API request.
- API key issues? Ensure the environment variable specified in `api_key_env` is set.
