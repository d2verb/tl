//! Interactive chat mode for translation sessions.
//!
//! Provides a REPL-style interface with slash commands for configuration.

/// Slash command parsing and autocomplete.
pub mod command;
mod session;
mod ui;

pub use session::{ChatSession, SessionConfig};
