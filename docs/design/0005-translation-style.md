# Translation Style Feature

## Overview

This document describes the design for a translation style feature that allows users to control the tone and style of translations (e.g., casual, formal, literal).

## Problem Statement

Currently, `tl` does not provide control over translation style. The LLM decides the tone based on context, which may not always match user expectations. Users may want:

- Casual translations for personal messages
- Formal translations for business documents
- Literal translations for technical accuracy
- Custom styles for specific use cases (e.g., regional dialects)

## Goals

1. Provide preset styles for common use cases
2. Allow users to define custom styles
3. Support style selection via CLI, configuration, and chat mode
4. Integrate with existing caching mechanism

## Non-Goals

- Language-specific style variants (styles are universal)
- Style auto-detection based on input content
- Style mixing (combining multiple styles)

## Design

### Preset Styles

Four preset styles are embedded in the code (not user-modifiable):

| Key | Description | Prompt Hint |
|-----|-------------|-------------|
| `casual` | Casual, conversational tone | "Translate in a casual, friendly, conversational tone" |
| `formal` | Formal, business-appropriate | "Translate in a formal, polite, business-appropriate tone" |
| `literal` | Literal, close to source | "Translate as literally as possible while remaining grammatical" |
| `natural` | Natural, idiomatic | "Translate naturally, prioritizing idiomatic expressions" |

### Custom Styles

Users can define custom styles in the configuration file. Each style has a `description` (for display in lists) and a `prompt` (instruction appended to the system prompt):

```toml
[styles.kansai]
description = "Translate into Kansai dialect"
prompt = "Translate into Kansai dialect. Use Kansai-ben expressions like やねん, ほんま, めっちゃ."

[styles.tech_blog]
description = "Friendly technical blog style"
prompt = "Translate in a friendly technical blog style. Keep technical terms accurate but explain them conversationally."

[styles.academic]
description = "Academic, scholarly tone"
prompt = "Translate in an academic, scholarly tone. Use formal language and precise terminology."
```

**Constraints:**
- Custom style keys must not conflict with preset keys
- Custom style keys must start with a letter and contain only alphanumeric characters and underscores

### Command Structure

```
tl styles                  # List all styles (presets + custom)
tl styles add              # Add a new custom style
tl styles edit <name>      # Edit a custom style
tl styles remove <name>    # Remove a custom style
```

### CLI Option

```bash
tl --style casual ./file.md
tl --style my_custom ./file.md
```

### Configuration

```toml
[tl]
provider = "ollama"
model = "gemma3:12b"
to = "ja"
style = "casual"  # Optional default style

[styles.my_custom]
description = "My custom style"
prompt = "Translate with my custom style instructions here."
```

### Chat Mode Integration

New `/set` command for runtime configuration changes:

```
/set style casual
/set style my_custom
/set to ja
/set model gpt-4o
```

**Design Decision:** Using `/set <key> <value>` pattern instead of separate commands (e.g., `/style casual`) provides:
- Consistent interface for all runtime settings
- Extensibility for future settings
- Clearer semantics (setting vs. action)

### Style Resolution

Priority order (highest to lowest):
1. CLI option (`--style`)
2. Chat `/set style` (in chat mode)
3. Config file default (`[tl] style`)
4. No style (current behavior)

### System Prompt Integration

When a style is specified, append the style instruction to the system prompt:

```rust
fn build_system_prompt(target_language: &str, style: Option<&str>) -> String {
    let base = format!("Translate the following text to {target_language}.");
    match style {
        Some(s) => format!("{base} {s}"),
        None => base,
    }
}
```

### Cache Key Changes

The cache key must include the style to avoid returning cached translations with different styles:

```rust
let cache_input = serde_json::json!({
    "source_text": self.source_text,
    "target_language": self.target_language,
    "model": self.model,
    "endpoint": self.endpoint,
    "prompt_hash": prompt_hash,
    "style": self.style,  // New field
});
```

### Error Handling

- Unknown style key: Error with message listing available styles
- Attempting to add/edit/remove a preset: Error indicating presets are immutable
- Duplicate custom style key: Error during `tl styles add`

### Command Details

#### `tl styles`

List all available styles with their descriptions.

```
$ tl styles
Preset styles
  casual    Casual, conversational tone
  formal    Formal, business-appropriate
  literal   Literal, close to source
  natural   Natural, idiomatic

Custom styles
  kansai    Translate into Kansai dialect
  tech_blog Translate in a friendly technical blog style
```

If no custom styles are defined, omit the "Custom styles" section.

#### `tl styles add`

Interactively add a new custom style.

```
$ tl styles add
? Style name: › kansai
? Description (shown in style list): › Translate into Kansai dialect
? Prompt (instruction for LLM): › Translate into Kansai dialect. Use Kansai-ben expressions like やねん, ほんま, めっちゃ.

Style 'kansai' added
```

**Validation:**
- Name must not conflict with presets
- Name must start with a letter and contain only alphanumeric characters and underscores
- Name must not already exist in custom styles

#### `tl styles edit <name>`

Edit an existing custom style. Both description and prompt can be modified.

```
$ tl styles edit kansai
Editing style 'kansai':

? Description (shown in style list): (Translate into Kansai dialect) › Translate into Kansai dialect with humor
? Prompt (instruction for LLM): (Translate into Kansai dialect...) › Translate into Kansai dialect with humor. Use expressions like やねん, ほんま, めっちゃ and add playful tone.

Style 'kansai' updated
```

**Validation:**
- Cannot edit preset styles (error message)

#### `tl styles remove <name>`

Remove a custom style with confirmation.

```
$ tl styles remove kansai
? Remove style 'kansai'? (y/N) › y

Style 'kansai' removed
```

**Edge Cases:**
- Cannot remove preset styles
- If the removed style is the default, warn but allow removal

### Chat `/set` Command Design

The `/set` command follows a `key value` pattern:

```
/set <key> <value>
```

Supported keys:
| Key | Value | Example |
|-----|-------|---------|
| `style` | Style name or empty to clear | `/set style casual`, `/set style` |
| `to` | Language code | `/set to ja` |
| `model` | Model name | `/set model gpt-4o` |

**Parser Changes:**

```rust
pub enum SlashCommand {
    Quit,
    Help,
    Config,
    Set { key: String, value: Option<String> },
    Unknown(String),
}
```

## Implementation Notes

### File Structure

```
src/
├── style/
│   └── mod.rs          # Presets, resolution, validation, errors
├── cli/commands/
│   ├── styles.rs       # tl styles commands
│   └── ...
└── chat/
    ├── command.rs      # /set command parsing
    └── session.rs      # SessionConfig with cached custom_styles
```

### Data Structures

```rust
// Preset styles (hardcoded, in src/style/mod.rs)
pub struct PresetStyle {
    pub key: &'static str,
    pub description: &'static str,
    pub prompt: &'static str,
}

pub const PRESETS: &[PresetStyle] = &[
    PresetStyle {
        key: "casual",
        description: "Casual, conversational tone",
        prompt: "Use a casual, friendly, conversational tone.",
    },
    // ...
];

// Custom style (user-defined, stored in config file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStyle {
    pub description: String,  // For display in style list
    pub prompt: String,       // Instruction for LLM
}

// Style resolution result
pub enum ResolvedStyle {
    Preset(&'static PresetStyle),
    Custom { key: String, prompt: String },
}
```

### Config Changes

```rust
// In ConfigFile (src/config/manager.rs)
pub struct ConfigFile {
    pub tl: Option<TlConfig>,
    pub providers: HashMap<String, Provider>,
    pub styles: HashMap<String, CustomStyle>,  // key -> {description, prompt}
}

pub struct TlConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub to: Option<String>,
    pub style: Option<String>,  // Default style key
}

// ResolvedConfig includes both style_name (for display) and style_prompt (for LLM)
pub struct ResolvedConfig {
    // ... other fields ...
    pub style_name: Option<String>,    // The style key (e.g., "casual")
    pub style_prompt: Option<String>,  // The prompt to append to system prompt
}

// SessionConfig caches custom_styles to avoid file I/O on /set style
pub struct SessionConfig {
    // ... other fields ...
    pub style_name: Option<String>,
    pub style_prompt: Option<String>,
    pub custom_styles: HashMap<String, CustomStyle>,
}
```

## Testing & Verification

### Unit Tests

- Preset lookup by key
- Custom style resolution
- Style conflict detection (preset vs custom)
- Cache key generation with/without style
- `/set` command parsing

### Integration Tests

- `tl styles add/edit/remove` workflow
- `--style` option with translation
- Style persistence in config file
- Cache invalidation when style changes

### Manual Testing

- Verify different styles produce noticeably different translations
- Verify error messages are clear for invalid style names

## Alternatives Considered

### Style as System Prompt Override

Allow users to completely override the system prompt instead of appending style instructions.

**Rejected:** Too powerful and easy to break. Style instructions are safer.

### Per-Language Style Definitions

Allow different style definitions per target language (e.g., Japanese-specific honorifics).

**Rejected:** Adds complexity. Users can create custom styles like `formal_ja` if needed.

### Inline Style Syntax

Support inline style specification: `tl "Hello" --style="friendly and warm"`

**Rejected:** Conflicts with the key-based approach. Use `tl styles add` for ad-hoc styles.

## Future Considerations

- Style suggestions based on input content type
- Style preview (show example translation before applying)
- Import/export custom styles
- Community style presets
