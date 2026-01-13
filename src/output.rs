//! Global output configuration and utilities.
//!
//! This module provides centralized control over CLI output behavior,
//! including quiet mode, color support, and stderr/stdout routing.
//!
//! ## Design Principles
//!
//! - Translation output goes to stdout (for piping)
//! - Status messages, progress, and logs go to stderr
//! - Errors always go to stderr
//! - Quiet mode suppresses non-essential output
//! - Colors can be disabled via flag or NO_COLOR environment variable

use std::io::{self, Write};
use std::sync::OnceLock;

/// Global output configuration.
static OUTPUT_CONFIG: OnceLock<OutputConfig> = OnceLock::new();

/// Output configuration settings.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Suppress non-essential output.
    pub quiet: bool,
    /// Disable colored output.
    pub no_color: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            quiet: false,
            // Check NO_COLOR environment variable (https://no-color.org/)
            no_color: std::env::var("NO_COLOR").is_ok(),
        }
    }
}

/// Initialize the global output configuration.
///
/// This should be called once at startup with the CLI flags.
/// If called multiple times, subsequent calls are ignored.
pub fn init(config: OutputConfig) {
    let _ = OUTPUT_CONFIG.set(config);
}

/// Get the current output configuration.
pub fn config() -> &'static OutputConfig {
    OUTPUT_CONFIG.get_or_init(OutputConfig::default)
}

/// Check if quiet mode is enabled.
pub fn is_quiet() -> bool {
    config().quiet
}

/// Check if colors are disabled.
pub fn is_no_color() -> bool {
    config().no_color
}

/// Print a status message to stderr (respects quiet mode).
///
/// Use this for progress indicators, informational messages, etc.
#[macro_export]
macro_rules! status {
    ($($arg:tt)*) => {
        if !$crate::output::is_quiet() {
            eprintln!($($arg)*);
        }
    };
}

/// Print a status message to stderr without newline (respects quiet mode).
#[macro_export]
macro_rules! status_no_newline {
    ($($arg:tt)*) => {
        if !$crate::output::is_quiet() {
            eprint!($($arg)*);
            let _ = std::io::stderr().flush();
        }
    };
}

/// Print an info message to stderr (respects quiet mode).
///
/// Use this for non-essential informational output.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if !$crate::output::is_quiet() {
            eprintln!($($arg)*);
        }
    };
}

/// Print a warning message to stderr (always shown, even in quiet mode).
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

/// Flush stderr.
pub fn flush_stderr() {
    let _ = io::stderr().flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_config_default() {
        // Note: This test may be affected by NO_COLOR env var in test environment
        let config = OutputConfig::default();
        assert!(!config.quiet);
    }

    #[test]
    fn test_is_quiet_default() {
        // Without initialization, should use default (not quiet)
        // Note: This relies on OnceLock not being set yet or being set to default
        let config = OutputConfig::default();
        assert!(!config.quiet);
    }
}
