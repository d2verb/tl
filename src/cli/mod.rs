//! Command-line interface definitions and handlers.

/// CLI argument parsing with clap.
pub mod args;

/// Subcommand implementations.
pub mod commands;

pub use args::{Args, Command, ProvidersCommand, StylesCommand};
