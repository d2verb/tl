# Interactive Configuration Design

## Overview

This document describes the design for interactive configuration commands to improve the user experience of setting up `tl`.

## Problem Statement

Currently, users must manually edit `~/.config/tl/config.toml` to configure the tool. This creates friction for:

- First-time users who don't know the config format
- Users who want to quickly add or modify providers
- Users who prefer interactive workflows over text editing

## Goals

1. Provide interactive commands for configuration
2. Separate default settings (`[tl]` section) from provider management (`[providers.*]` sections)
3. Keep the CLI simple and intuitive
4. Avoid command name conflicts

## Non-Goals

- Replace manual config file editing (power users should still be able to edit directly)
- Provide a full TUI configuration interface (that's a separate feature)

## Design

### Command Structure

```
tl configure                  # Edit [tl] section (defaults)
tl providers                  # List all providers
tl providers add              # Add a new provider
tl providers edit <name>      # Edit an existing provider
tl providers remove <name>    # Remove a provider
```

### Breaking Change

The current `tl providers <name>` command (show details for a specific provider) will be **removed** to avoid conflicts with subcommands like `add`, `edit`, `remove`.

The `tl providers` command (list all) provides sufficient information, and detailed provider info can be seen during `tl providers edit <name>`.

### Command Details

#### `tl configure`

Interactively edit the `[tl]` section defaults.

**Precondition:** At least one provider must be configured. If no providers exist, display an error and prompt the user to run `tl providers add` first.

**Flow:**

```
$ tl configure
Current defaults:
  provider  ollama
  model     gemma3:12b
  to        ja

? Default provider: (ollama) › sakura
? Default model: (gemma3:12b) › gpt-4o
? Default target language: (ja) › en

✓ Configuration saved to ~/.config/tl/config.toml
```

**Validation:**

- Provider must exist in `[providers.*]`
- Target language must be a valid ISO 639-1 code
- Model is free-form (warning if not in provider's model list)

#### `tl providers`

List all configured providers with their key information.

**Flow:**

```
$ tl providers
Configured providers
  sakura (default)
    endpoint  https://api.ai.sakura.ad.jp
    models    gpt-oss-70b, gpt-oss-120b
  ollama
    endpoint  http://localhost:11434
    models    gemma3:12b, llama3.2
```

#### `tl providers add`

Interactively add a new provider.

**Flow:**

```
$ tl providers add
? Provider name: › sakura
? Endpoint URL: › https://api.ai.sakura.ad.jp
? API key method:
  › Environment variable (recommended)
    Store in config file
    None (no auth required)
? Environment variable name: › SAKURA_API_KEY
? Models (comma-separated, optional): › gpt-oss-70b, gpt-oss-120b

✓ Provider 'sakura' added to ~/.config/tl/config.toml
```

**Validation:**

- Provider name must not already exist
- Provider name must not conflict with subcommands (`add`, `edit`, `remove`)
- Endpoint must be a valid URL

#### `tl providers edit <name>`

Interactively edit an existing provider. All fields are shown with current values as defaults.

**Flow:**

```
$ tl providers edit sakura
Editing provider 'sakura':

? Endpoint URL: (https://api.ai.sakura.ad.jp) ›
? API key method: (Environment variable) ›
? Environment variable name: (SAKURA_API_KEY) ›
? Models (comma-separated): (gpt-oss-70b, gpt-oss-120b) › gpt-oss-70b, gpt-oss-120b, gpt-oss-200b

✓ Provider 'sakura' updated
```

**Design Decision:** Models are edited as a comma-separated list rather than individual add/remove operations. This keeps the interface simple. Users who need fine-grained control can edit the config file directly.

#### `tl providers remove <name>`

Remove a provider with confirmation.

**Flow:**

```
$ tl providers remove sakura
? Are you sure you want to remove provider 'sakura'? (y/N) › y

✓ Provider 'sakura' removed
```

**Edge Cases:**

- Cannot remove the default provider without first changing the default
- If removing the last provider, warn the user

## Implementation Notes

### Dependencies

Use the existing `inquire` crate for interactive prompts.

### File Structure

```
src/cli/commands/
├── configure.rs    # tl configure
├── providers.rs    # tl providers, tl providers add/edit/remove
└── ...
```

### Config File Handling

- Use existing `ConfigManager` for reading/writing
- Preserve comments and formatting where possible (may require switching to `toml_edit` crate)

## Future Considerations

- `tl setup` wizard for first-time users (combines provider add + configure)
- Model validation against provider's API (query available models)
- Config file backup before modifications

## Alternatives Considered

### Nested Subcommands for Models

```
tl providers <name> models add <model>
tl providers <name> models remove <model>
```

**Rejected:** Adds complexity for a rarely-used operation. Editing the full models list is sufficient.

### Separate `tl provider` (singular) Command

```
tl provider add/edit/remove
tl providers  # list only
```

**Rejected:** Having both `provider` and `providers` is confusing.

### Keep `tl providers <name>` for Details

**Rejected:** Conflicts with subcommands. A provider named "add" would be ambiguous.
