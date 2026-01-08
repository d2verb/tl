This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Generate documentation
cargo doc --open

# Formatter
cargo fmt

# Lint
cargo clippy

# Lint (fix it)
cargo clippy --fix --allow-dirty

# Test Coverage
cargo llvm-cov
```

## Development Rules

- **Execute tests, fmt, and clippy** at the end of each task.
- **Draft a design doc** before implementing major features.
- **Always include tests** (Unit or Integration) when fixing bugs or adding features.
- **Target 85%+ test coverage** (best effort).
- Update all related documents at the end of each task to keep documents up to date with changes.
