# tl – streaming, cached translation CLI

[![CI](https://github.com/d2verb/tl/actions/workflows/ci.yml/badge.svg)](https://github.com/d2verb/tl/actions/workflows/ci.yml)

`tl` is a small CLI that streams translations through any OpenAI-compatible endpoint (local or remote). Configure multiple providers with their own endpoints, API keys, and models, then switch between them as needed.

## Install

```sh
cargo install tl-cli
```

Or build from source:

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

## Chat Mode

For interactive translation sessions:

```sh
tl chat                              # start with config defaults
tl chat --to ja                      # override target language
tl chat --provider openrouter        # use a specific provider
```

Type text and press Enter to translate. Use `/help` for commands, `/quit` to exit.

## Configuration Reference

Settings are stored in `~/.config/tl/config.toml`:

```toml
[tl]
provider = "ollama"
model = "gemma3:12b"
to = "ja"

[providers.ollama]
endpoint = "http://localhost:11434"
models = ["gemma3:12b", "llama3.2"]

[providers.openrouter]
endpoint = "https://openrouter.ai/api"
api_key_env = "OPENROUTER_API_KEY"
models = ["anthropic/claude-3.5-sonnet", "openai/gpt-4o"]
```

### Provider options

- `endpoint` (required) – OpenAI-compatible API endpoint
- `api_key_env` (optional) – environment variable name for API key
- `api_key` (optional) – API key in config (not recommended)
- `models` (optional) – available models for this provider

CLI options always override config file values.

## Troubleshooting

- Run `tl languages` to see supported ISO 639-1 language codes.
- Pressing `Ctrl+C` while streaming aborts without polluting the cache.
- Use `--no-cache` to force a fresh API request.
- API key issues? Ensure the environment variable specified in `api_key_env` is set.
