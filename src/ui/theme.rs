//! Consistent styling utilities for CLI output.
//!
//! Provides color and formatting helpers using owo-colors.
//! Respects the `--no-color` flag and `NO_COLOR` environment variable.

use crate::output;
use owo_colors::OwoColorize;
use std::fmt::Display;

/// Styles for different semantic elements.
///
/// All style functions respect the global `no_color` setting.
/// When colors are disabled, text is returned without formatting.
pub struct Style;

impl Style {
    /// Style for section headers (e.g., "Configuration", "Available commands")
    pub fn header<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.bold())
        }
    }

    /// Style for labels/keys (e.g., "provider", "model")
    pub fn label<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.dimmed())
        }
    }

    /// Style for primary values (e.g., provider names, model names)
    pub fn value<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.cyan())
        }
    }

    /// Style for secondary/supplementary info (e.g., endpoints, descriptions)
    pub fn secondary<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.dimmed())
        }
    }

    /// Style for success messages
    pub fn success<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.green())
        }
    }

    /// Style for error messages
    pub fn error<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.red().bold())
        }
    }

    /// Style for warning messages
    pub fn warning<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.yellow())
        }
    }

    /// Style for commands (e.g., "/config", "/help")
    pub fn command<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.green())
        }
    }

    /// Style for language codes
    pub fn code<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.yellow())
        }
    }

    /// Style for hints/help text
    pub fn hint<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.dimmed().italic())
        }
    }

    /// Style for the default marker
    pub fn default_marker() -> String {
        if output::is_no_color() {
            "(default)".to_string()
        } else {
            format!("{}", "(default)".dimmed())
        }
    }

    /// Style for version info
    pub fn version<T: Display>(text: T) -> String {
        if output::is_no_color() {
            text.to_string()
        } else {
            format!("{}", text.dimmed())
        }
    }
}
