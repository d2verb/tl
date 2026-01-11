//! Consistent styling utilities for CLI output.
//!
//! Provides color and formatting helpers using owo-colors.

use owo_colors::OwoColorize;
use std::fmt::Display;

/// Styles for different semantic elements.
pub struct Style;

impl Style {
    /// Style for section headers (e.g., "Configuration", "Available commands")
    pub fn header<T: Display>(text: T) -> String {
        format!("{}", text.bold())
    }

    /// Style for labels/keys (e.g., "provider", "model")
    pub fn label<T: Display>(text: T) -> String {
        format!("{}", text.dimmed())
    }

    /// Style for primary values (e.g., provider names, model names)
    pub fn value<T: Display>(text: T) -> String {
        format!("{}", text.cyan())
    }

    /// Style for secondary/supplementary info (e.g., endpoints, descriptions)
    pub fn secondary<T: Display>(text: T) -> String {
        format!("{}", text.dimmed())
    }

    /// Style for success messages
    pub fn success<T: Display>(text: T) -> String {
        format!("{}", text.green())
    }

    /// Style for error messages
    pub fn error<T: Display>(text: T) -> String {
        format!("{}", text.red().bold())
    }

    /// Style for warning messages
    pub fn warning<T: Display>(text: T) -> String {
        format!("{}", text.yellow())
    }

    /// Style for commands (e.g., "/config", "/help")
    pub fn command<T: Display>(text: T) -> String {
        format!("{}", text.green())
    }

    /// Style for language codes
    pub fn code<T: Display>(text: T) -> String {
        format!("{}", text.yellow())
    }

    /// Style for hints/help text
    pub fn hint<T: Display>(text: T) -> String {
        format!("{}", text.dimmed().italic())
    }

    /// Style for the default marker
    pub fn default_marker() -> String {
        format!("{}", "(default)".dimmed())
    }

    /// Style for version info
    pub fn version<T: Display>(text: T) -> String {
        format!("{}", text.dimmed())
    }
}
