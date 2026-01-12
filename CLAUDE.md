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

## Tool Usage Guidelines

- **Documentation Search**: Prioritize using the `context7` MCP tool for all technical documentation and API specification searches. Always fetch the latest information before proceeding with implementation.
- **Code Search & Analysis**: Prioritize using the `serena` MCP tool for deep code understanding, semantic searches, and dependency analysis across the project.
- **Advanced Consultation & Troubleshooting**: Prioritize using the `codex` MCP tool for the following scenarios:
  - Architectural design discussions and system modeling.
  - Comprehensive code reviews for critical features or complex logic.
  - Bug fixes that have failed 3 or more times.
