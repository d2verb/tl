# Mini Design Doc: Multi-Provider & Multi-Model Support

* **Author:** d2verb
* **Status:** Implemented
* **Date:** 2026-01-08

## 1. Abstract

Allow configuring multiple providers (local Ollama, OpenRouter, OpenAI, etc.) and registering multiple models for each provider. Support providers that require an API Key.

## 2. Goals & Non-Goals

### Goals

* Configure and switch between multiple providers
* Register multiple models per provider
* Secure management of API Keys (environment variables or config file)
* Provider/model selection from CLI
* Manage `provider`, `model`, `to` as required settings

### Non-Goals

* Encryption of API Keys (out of scope for v1)
* Provider‑specific feature support (rate limiting, usage tracking, etc.)
* Concurrent requests to multiple providers (fallback)

## 3. Context & Problem Statement

The current `tl` supports only a single endpoint and model. In real use cases:

* Want to switch between local Ollama (fast, free) and OpenRouter (high quality)
* Even within the same provider, want to switch models depending on the task (lightweight vs high‑accuracy)
* OpenRouter and OpenAI require an API Key

## 4. Proposed Design

### 4.1 Configuration Structure

```toml
# ~/.config/tl/config.toml

# Global settings (all required, specified via config or CLI)
[tl]
provider = "ollama"
model = "gemma3:12b"
to = "ja"

# Provider definitions
[providers.ollama]
endpoint = "http://localhost:11434"
models = ["gemma3:12b", "llama3.2", "qwen2.5:14b"]

[providers.openrouter]
endpoint = "https://openrouter.ai/api"
api_key_env = "OPENROUTER_API_KEY"      # read from environment variable
# api_key = "sk-..."                    # or specify directly (discouraged)
models = [
    "anthropic/claude-3.5-sonnet",
    "openai/gpt-4o",
    "google/gemini-2.0-flash-exp:free",
]

[providers.openai]
endpoint = "https://api.openai.com"
api_key_env = "OPENAI_API_KEY"
models = ["gpt-4o", "gpt-4o-mini"]
```

### 4.2 CLI Interface

```bash
# Provider and model specification
tl --provider ollama ./file.md
tl --provider openrouter --model anthropic/claude-3.5-sonnet ./file.md
tl -p ollama -m llama3.2 ./file.md

# Same for chat mode
tl chat --provider openrouter
tl chat -p ollama -m gemma3:12b

# Short form (provider:model syntax)
tl --use ollama:gemma3:12b ./file.md
tl --use openrouter:anthropic/claude-3.5-sonnet ./file.md

# List providers
tl providers

# List models for a provider
tl providers ollama
```

### 4.3 Priority Order

Configuration priority (highest first):

```
1. CLI arguments (--provider, --model, --to)
2. [tl] section in config.toml (provider, model, to)

Note: provider, model, and to are all required. If any is missing, an error is raised.
```

### 4.4 API Key Handling

```
Priority:
1. Environment variable (specified by api_key_env)
2. api_key field in config file (discouraged, show warning)

If a provider that requires an API Key has none set:
- Show an error message with instructions on how to set the environment variable
```

### 4.5 Module Structure

```
src/
├── config/
│   ├── mod.rs
│   ├── manager.rs
│   └── provider.rs      # new: Provider, ProviderConfig
└── ...
```

### 4.6 Data Structures

```rust
/// Entire configuration file
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub tl: TlConfig,
    pub providers: HashMap<String, ProviderConfig>,
}

/// [tl] section
#[derive(Debug, Deserialize, Serialize)]
pub struct TlConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub to: Option<String>,
}

/// Provider configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub endpoint: String,
    pub api_key: Option<String>,          // directly specified (discouraged)
    pub api_key_env: Option<String>,      // environment variable name
    pub models: Vec<String>,
}

impl ProviderConfig {
    /// Retrieve API Key (environment variable takes precedence)
    pub fn get_api_key(&self) -> Option<String> {
        if let Some(env_var) = &self.api_key_env {
            if let Ok(key) = std::env::var(env_var) {
                return Some(key);
            }
        }
        self.api_key.clone()
    }

    /// Whether an API Key is required
    pub fn requires_api_key(&self) -> bool {
        self.api_key.is_some() || self.api_key_env.is_some()
    }
}

/// Resolved translation configuration
#[derive(Debug)]
pub struct ResolvedConfig {
    pub provider_name: String,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub target_language: String,
}
```

### 4.7 HTTP Header for API Key

```rust
// Add Authorization header when API Key is present
let mut request = self.client.post(&url).json(&chat_request);

if let Some(api_key) = &config.api_key {
    request = request.header("Authorization", format!("Bearer {api_key}"));
}
```

### 4.8 Error Messages

```
# Missing required configuration
Error: Missing required configuration: 'provider'

Please provide it via:
  - CLI option: tl --provider <name>
  - Config file: ~/.config/tl/config.toml

# Provider not found
Error: Provider 'unknown' not found

Available providers:
  - ollama
  - openrouter

Add providers to ~/.config/tl/config.toml

# API Key not set
Error: Provider 'openrouter' requires an API key

Set the OPENROUTER_API_KEY environment variable:
  export OPENROUTER_API_KEY="your-api-key"

Or set api_key in ~/.config/tl/config.toml

# Model not found (warning only, execution continues)
Warning: Model 'unknown-model' is not in the configured models list for 'ollama'
Configured models: gemma3:12b, llama3.2, qwen2.5:14b
Proceeding anyway...
```

## 5. Implementation Plan

1. **Phase 1: Data structure changes**
    * Implement `Config`, `ProviderConfig`
    * Read/write configuration file

2. **Phase 2: Add CLI options**
    * `--provider`, `--model` options
    * `providers` subcommand
    * Configuration resolution logic (`ResolvedConfig`)

3. **Phase 3: API Key support**
    * Load from environment variable
    * Add to HTTP headers
    * Improve error messages

4. **Phase 4: Tests & Documentation**
    * Unit tests
    * Update README

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| API Key leakage | High | Recommend environment variables; show warning when directly specified |
| Configuration file complexity | Low | Clear error messages and sample config in README |

## 7. Testing & Verification

### Unit Tests

* Configuration file parsing
* Provider resolution logic
* API Key retrieval

### Integration Tests

* Provider switching via CLI arguments
* Requests with API Key

### Manual Testing

* Verify translations with each provider

## 8. Alternatives Considered

### Configuration Structure

| Option | Pros | Cons | Decision |
| :--- | :--- | :--- | :--- |
| TOML table (`[providers.xxx]`) | Clear hierarchical structure | Slightly verbose | **Adopted** |
| Array (`[[provider]]`) | Order is explicit | Access by name is cumbersome | Rejected |
| Separate file (`providers.toml`) | Separation of concerns | More file management overhead | Rejected |

### API Key Management

| Option | Pros | Cons | Decision |
| :--- | :--- | :--- | :--- |
| Environment variable first | Secure, 12‑factor compliant | Settings are split | **Adopted** |
| Config file only | Simple | Security risk | Rejected |
| System keychain | Most secure | Complex implementation, OS‑dependent | Rejected |

---

## Appendix: Future Enhancements (Out of Scope for v1)

* Provider fallback (try next provider on error)
* Usage tracking
* Rate‑limit handling
* Encrypted storage of API Keys
* Provider‑specific custom headers
