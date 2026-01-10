# Mini Design Doc: tl - AI Translation CLI Tool

* **Author:** d2verb
* **Status:** Implemented
* **Date:** 2026-01-08

## 1. Abstract

`tl` is a CLI tool that translates text from files or standard input using an OpenAI‑compatible API. Streaming output and caching provide efficient translation without compromising the user experience.

## 2. Goals & Non-Goals

### Goals

* Read text from a file or standard input and translate it into the specified language
* Translation via an OpenAI‑compatible API (including local LLM servers)
* Low‑latency UX through streaming output
* Faster re‑translation via caching of translation results

### Non-Goals

* Implementing a custom translation API (depends on external APIs)
* Bulk translation of multiple files (v1 supports only a single file/stdin)
* UI for managing translation history
* Offline translation

## 3. Context & Problem Statement

There are many situations where you want to quickly translate a text file, but existing CLI tools have the following issues:

* They are dedicated to commercial APIs and do not support local LLMs
* They do not support streaming output, causing long blocks when translating long texts
* They lack a cache, incurring cost when the same sentence is translated repeatedly

This tool supports OpenAI‑compatible APIs, enabling various back‑ends (including local LLMs), and provides a comfortable translation experience through streaming output and caching.

## 4. Proposed Design

### 4.1 System Overview

```
┌───────────────────────────────────────────────────────��─────────┐
│                           tl CLI                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Input     │  │   Config    │  │      Translation        │  │
│  │   Handler   │  │   Manager   │  │        Engine           │  │
│  │             │  │             │  │                         │  │
│  │ - File      │  │ - TOML      │  │ - OpenAI Compatible API │  │
│  │ - Stdin     │  │ - CLI Args  │  │ - Streaming Response    │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                     │                │
│         └────────────────┼─────────────────────┘                │
│                          │                                      │
│                   ┌──────▼──────┐                               │
│                   │    Cache    │                               │
│                   │   Manager   │                               │
│                   │             │                               │
│                   │ - SHA256    │                               │
│                   │ - SQLite    │                               │
│                   └─────────────┘                               │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 CLI Interface

```bash
# Basic usage
tl <file>                    # Translate a file (to the default language)
cat <file> | tl              # Translate from stdin
tl -t <lang> <file>          # Specify target language
tl --to <lang> <file>        # Same as above (long option)

# Other options
tl --help                    # Show help
tl --version                 # Show version
tl languages                 # List supported language codes
tl --no-cache <file>         # Do not use cache
tl --provider <name> <file>  # Specify provider
tl --model <name> <file>     # Specify model
```

### 4.3 Configuration

**File path:** `~/.config/tl/config.toml`

```toml
[tl]
to = "ja"                              # Target language (required)
endpoint = "http://localhost:11434"    # API endpoint (required)
model = "gpt-oss:20b"                  # Model name (required)
```

**Configuration precedence:**

1. CLI options (highest priority)
2. Configuration file
3. Error (if required items are missing)

**Error when required configuration is missing:**

```
Error: Missing required configuration: 'endpoint'

Please provide it via:
  - CLI option: tl --endpoint <url>
  - Config file: ~/.config/tl/config.toml
```

### 4.4 Caching Strategy

**Language codes:**

Language codes are forced to ISO 639‑1 format (`ja`, `en`, `zh`, `ko`, `fr`, `de`, `es`, etc.).

* No need for normalization logic, keeping it simple
* Guarantees consistency of cache keys
* Invalid codes are rejected instantly as validation errors

```
Error: Invalid language code: 'Japanese'

Valid language codes (ISO 639-1): ja, en, zh, ko, fr, de, es, ...
Run 'tl languages' to see all supported codes.
```

**Cache key generation:**

A simple string concatenation could cause collisions, so we serialize to JSON first and then hash.

```rust
let cache_input = serde_json::json!({
    "source_text": source_text,
    "target_language": target_language,
    "model": model,
    "endpoint": endpoint,
    "prompt_hash": prompt_hash
});
let cache_key = SHA256(cache_input.to_string());
```

* JSON makes field boundaries explicit and prevents collisions
* `endpoint`: distinguishes translations from the same model on different servers (production vs. local, etc.)
* `prompt_hash`: SHA256 of the system‑prompt template; changing the prompt automatically invalidates the cache

**Cache storage:** `~/.cache/tl/translations.db` (SQLite)

**Schema:**

```sql
CREATE TABLE translations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cache_key TEXT UNIQUE NOT NULL,
    source_text TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    target_language TEXT NOT NULL,
    model TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    prompt_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_cache_key ON translations(cache_key);
```

**Ctrl+C handling during streaming:**

* Buffer in memory while streaming
* Write to DB only upon completion
* If interrupted with Ctrl+C, do not store in cache (prevents incomplete translations)

### 4.5 Input & Output Processing

**Input handling:**

All input is read fully before translation starts (no streaming input).

```
┌─────────────────┐          ┌─────────────────┐
│  File           │──read───▶│                 │
└─────────────────┘          │  Complete Text  │───▶ Translation
┌─────────────────┐          │  (in memory)    │
│  Stdin (EOF wait) │──read───▶│                 │
└─────────────────┘          └─────────────────┘
```

* File input: read the entire file
* Stdin: read until EOF (supports `cat file | tl`, `curl … | tl`, etc.)
* Translation starts only after input is gathered, fixing the cache‑key calculation

**Maximum input size:**

To prevent OOM, an input‑size limit is enforced.

* Max input size: **1 MB** (default)
* Exceeding the limit results in an error

```
Error: Input size (2.5 MB) exceeds maximum allowed size (1 MB).

Consider splitting the file into smaller parts.
```

* 1 MB is sufficient for most document translations
* Future option `--max-size` could allow changing the limit

**Output processing (streaming):**

The API response is handled as a stream, displaying translated text in real time.

```
┌──────────┐     ┌──────────────┐     ┌──────────────┐
│  Input   │────▶│   API Call   │────▶│   Stdout     │
│  Text    │     │  (Streaming) │     │  (Real-time) │
└──────────┘     └──────┬───────┘     └──────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │   Buffer     │
                 │  (In-Memory) │
                 └──────┬───────┘
                        │ (on complete)
                        ▼
                 ┌──────────────┐
                 │    Cache     │
                 │   (SQLite)   │
                 └──────────────┘
```

* Users are not forced to wait for the whole translation
* While outputting, data is buffered in memory and written to the cache only after completion

**UI display:**

* At translation start: spinner shown (stderr)
* While streaming: real‑time text output (stdout)
* At completion: spinner stops

```
⠋ Translating...
[Translated text streams here]
```

### 4.6 Module Structure

```
src/
├── main.rs              # Entry point, CLI parser
├── lib.rs               # Library root
├── cli/
│   ├── mod.rs
│   ├── args.rs          # CLI argument definitions (clap)
│   └── commands/
│       ├── mod.rs
│       └── translate.rs # Translate command
├── config/
│   ├── mod.rs
│   └── manager.rs       # Read/write config file
├── translation/
│   ├── mod.rs
│   ├── client.rs        # OpenAI‑compatible API client
│   └── prompt.rs        # Translation prompt generation
├── cache/
│   ├── mod.rs
│   └── sqlite.rs        # SQLite cache implementation
├── input/
│   ├── mod.rs
│   └── reader.rs        # File/stdin reading
└── ui/
    ├── mod.rs
    └── spinner.rs       # Spinner display
```

### 4.7 Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }       # CLI argument parser
tokio = { version = "1", features = ["full"] }        # Async runtime
reqwest = { version = "0.12", features = ["stream", "json"] } # HTTP client
serde = { version = "1", features = ["derive"] }      # Serialization
serde_json = "1"                                      # JSON handling
toml = "0.8"                                          # TOML config files
rusqlite = { version = "0.32", features = ["bundled"] } # SQLite cache
sha2 = "0.10"                                         # Hash calculation
hex = "0.4"                                           # Hex encoding
indicatif = "0.17"                                    # Spinner / progress display
inquire = "0.9"                                       # Interactive prompts
dirs = "5"                                            # Platform‑specific directories
anyhow = "1"                                          # Error handling
futures-util = "0.3"                                  # Stream processing
async-stream = "0.3"                                  # Async stream generation

[dev-dependencies]
tempfile = "3"                                        # Temporary files for tests
```

### 4.8 API Request Format

Request to an OpenAI‑compatible API:

```json
POST {endpoint}/v1/chat/completions
Content-Type: application/json

{
  "model": "{model}",
  "messages": [
    {
      "role": "system",
      "content": "You are a translator. Translate the following text to {target_language}. Output only the translated text without any explanations."
    },
    {
      "role": "user",
      "content": "{source_text}"
    }
  ],
  "stream": true
}
```

* `{model}`: model name from configuration or CLI (`--model`)
* `{target_language}`: target language from configuration or CLI (`--to`)
* `{source_text}`: text to be translated

## 5. Implementation Plan

1. **Phase 1: Foundations**
    * Set up project structure
    * Implement CLI argument parser (clap)
    * Implement config file read/write

2. **Phase 2: Translation core**
    * Implement input handler (file/stdin)
    * Implement OpenAI‑compatible API client
    * Implement streaming response handling
    * Implement spinner UI

3. **Phase 3: Caching**
    * Implement SQLite cache
    * Handle cache hits/misses
    * Support `--no-cache` option

4. **Phase 4: Quality improvements**
    * Enhance error handling
    * Add unit and integration tests
    * Polish documentation

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| Differences in API response formats | High | Conform to the OpenAI standard format; test with major LLM servers (Ollama, vLLM, etc.) |
| Memory usage on large files | Medium | Chunked translation is considered for future extensions; v1 only shows a warning |
| Data inconsistency when streaming is aborted | Low | Cache write occurs only on successful completion |
| Concurrent SQLite access | Low | Assume single‑process usage; enable WAL mode to reduce contention |

## 7. Testing & Verification

### Unit Tests

* Config file parsing/serialization
* Cache key generation logic
* CLI argument parsing
* Input source detection (file vs. stdin)

### Integration Tests

* Communication with translation API (mock server)
* Behavior on cache hit
* Error cases (missing file, API connection failure, etc.)

### Success Metrics

* Test coverage ≥ 85 %
* Initial latency before streaming starts < 500 ms (excluding network)
* Response time on cache hit < 50 ms

## 8. Alternatives Considered

### Cache storage

| Option | Pros | Cons | Decision |
| :--- | :--- | :--- | :--- |
| SQLite | Robust, queryable, single‑file | Extra dependency | **Chosen** |
| JSON/TOML file | Simple | Slow with large data, concurrency issues | Rejected |
| sled (embedded DB) | Rust native | Unclear maintenance status | Rejected |

### Async runtime

| Option | Pros | Cons | Decision |
| :--- | :--- | :--- | :--- |
| tokio | Mature, rich ecosystem | Larger binary size | **Chosen** |
| async‑std | Lightweight | Smaller ecosystem | Rejected |
| Synchronous processing | Simpler | Streaming implementation becomes complex | Rejected |

---

## Appendix: Future Enhancements (Out of Scope for v1)

* Bulk translation of multiple files
* Automatic detection of source language
* Chunked processing for very large files
* Cache expiration settings
* Feedback mechanism for translation quality
