# tl – streaming, cached translation CLI

[![CI](https://github.com/d2verb/tl/actions/workflows/ci.yml/badge.svg)](https://github.com/d2verb/tl/actions/workflows/ci.yml)

`tl` is a small CLI that streams translations coming from standard input or files through any OpenAI-compatible endpoint (local or remote). Configure multiple providers with their own endpoints, API keys, and models, then switch between them as needed.

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

## Quick Usage

```sh
tl ./notes.md                       # translate a file
cat report.md | tl                   # translate stdin
tl --to ja ./notes.md                # override target language
tl --provider openrouter ./notes.md  # use a specific provider
tl --model gpt-4o ./notes.md         # use a specific model
tl --no-cache ./notes.md             # bypass cache
tl -w ./notes.md                     # overwrite file with translation
```

`tl` caches each translation (keyed on the input, language, model, endpoint, and prompt) so rerunning the same source is fast and cheap. Streaming responses keep your terminal responsive, and a spinner on stderr signals when work is in progress.

## Configuration

Settings live in `~/.config/tl/config.toml`.

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

### Provider Configuration

Each provider has:
- `endpoint` (required) – the OpenAI-compatible API endpoint
- `api_key_env` (optional) – environment variable name containing the API key
- `api_key` (optional) – API key stored directly in config (not recommended)
- `models` (optional) – list of available models for this provider

CLI options always supersede the config file.

### Managing Providers

```sh
tl providers                        # list all providers
tl providers add                    # add a new provider interactively
tl providers edit <name>            # edit an existing provider
tl providers remove <name>          # remove a provider
```

### Configuring Defaults

Use `tl configure` to set default provider, model, and target language interactively:

```sh
tl configure
```

## Chat Mode

For interactive translation sessions, use `tl chat`:

```sh
tl chat                              # start chat mode with config defaults
tl chat --to ja                      # override target language
tl chat --provider openrouter        # use a specific provider
tl chat --model gpt-4o               # use a specific model
```

Type text and press Enter to translate. Translations stream in real-time.

### Slash Commands

- `/config` – show current configuration
- `/help` – list available commands
- `/quit` – exit chat mode

## Troubleshooting

- Use `tl languages` to see the supported ISO 639-1 codes before passing `--to`.
- Streaming is cancel-safe: pressing `Ctrl+C` while streaming aborts without polluting the cache.
- No cache hits? Run with `--no-cache` to force a fresh request.
- API key issues? Set the environment variable specified in `api_key_env` for your provider.
