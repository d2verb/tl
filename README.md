# tl – streaming, cached translation CLI

`tl` is a small CLI that streams translations coming from standard input or files through any OpenAI-compatible endpoint (local or remote). Configure a default target language, endpoint, and model once, then override per-command as needed.

## Install

```sh
cargo install --path .
```

## Quick Usage

```sh
tl ./notes.md                       # translate a file
cat report.md | tl                   # translate stdin
tl --to ja ./notes.md                # override target language
tl --endpoint http://localhost:11434 ./notes.md  # temporary endpoint
tl --no-cache ./notes.md             # bypass cache
```

`tl` caches each translation (keyed on the input, language, model, endpoint, and prompt) so rerunning the same source is fast and cheap. Streaming responses keep your terminal responsive, and a spinner on stderr signals when work is in progress.

## Configuration

Settings live in `~/.config/tl/config.toml`. Run `tl configure` to set your defaults interactively (it pre-fills existing values) and `tl configure --show` to inspect them.

```toml
[tl]
to = "ja"
endpoint = "http://localhost:11434"
model = "gpt-oss:20b"
```

CLI options always supersede the config file. If a required value is missing, `tl` prints a clear error and points you toward `tl configure` or the corresponding flag.

## Chat Mode

For interactive translation sessions, use `tl chat`:

```sh
tl chat                              # start chat mode with config defaults
tl chat --to ja                      # override target language
```

In chat mode, type text to translate and press Enter. Use slash commands to manage the session:

- `/config` – show current configuration
- `/set <key> <value>` – change a setting (e.g., `/set to en`)
- `/help` – list available commands
- `/quit` – exit chat mode

## Troubleshooting

- Use `tl languages` to see the supported ISO 639-1 codes before passing `--to`.
- Streaming is cancel-safe: pressing `Ctrl+C` while streaming aborts without polluting the cache.
- No cache hits? Run with `--no-cache` or `tl --endpoint …` to force a fresh request.
