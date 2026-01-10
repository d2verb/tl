//! # tl - Streaming Translation CLI
//!
//! `tl` is a command-line tool for translating text using OpenAI-compatible API endpoints.
//! It supports streaming output, caching, and multiple provider configurations.
//!
//! ## Features
//!
//! - **Streaming translations**: See translations as they arrive
//! - **Caching**: Avoid redundant API calls with SQLite-based caching
//! - **Multiple providers**: Configure and switch between different API providers
//! - **Interactive mode**: Chat-style translation sessions with `tl chat`
//!
//! ## Quick Start
//!
//! ```bash
//! # Translate a file
//! tl ./notes.md
//!
//! # Translate from stdin
//! cat report.md | tl
//!
//! # Override target language
//! tl --to ja ./notes.md
//!
//! # Interactive chat mode
//! tl chat
//! ```
//!
//! ## Configuration
//!
//! Settings are stored in `~/.config/tl/config.toml`:
//!
//! ```toml
//! [tl]
//! provider = "ollama"
//! model = "gemma3:12b"
//! to = "ja"
//!
//! [providers.ollama]
//! endpoint = "http://localhost:11434"
//! models = ["gemma3:12b", "llama3.2"]
//! ```

/// Translation cache management using `SQLite`.
pub mod cache;

/// Interactive chat mode for translation sessions.
pub mod chat;

/// Command-line interface definitions and handlers.
pub mod cli;

/// Configuration file management and provider settings.
pub mod config;

/// Input reading from files and stdin.
pub mod input;

/// Translation client for OpenAI-compatible APIs.
pub mod translation;

/// Terminal UI components (spinner, colors).
pub mod ui;
