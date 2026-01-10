//! XDG-style path utilities for configuration and cache directories.
//!
//! This module provides consistent path resolution across platforms,
//! preferring XDG Base Directory Specification conventions over
//! OS-specific locations.

use std::path::PathBuf;

/// Returns the configuration directory for tl.
///
/// Resolution order:
/// 1. `$XDG_CONFIG_HOME/tl` if `XDG_CONFIG_HOME` is set
/// 2. `~/.config/tl` otherwise
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map_or_else(
        |_| home_dir().join(".config").join("tl"),
        |xdg| PathBuf::from(xdg).join("tl"),
    )
}

/// Returns the cache directory for tl.
///
/// Resolution order:
/// 1. `$XDG_CACHE_HOME/tl` if `XDG_CACHE_HOME` is set
/// 2. `~/.cache/tl` otherwise
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
pub fn cache_dir() -> PathBuf {
    std::env::var("XDG_CACHE_HOME").map_or_else(
        |_| home_dir().join(".cache").join("tl"),
        |xdg| PathBuf::from(xdg).join("tl"),
    )
}

/// Returns the user's home directory.
///
/// # Panics
///
/// Panics if the home directory cannot be determined.
#[allow(clippy::expect_used)]
fn home_dir() -> PathBuf {
    dirs::home_dir().expect("Failed to determine home directory")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_default() {
        // Clear XDG_CONFIG_HOME to test default behavior
        let original = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };

        let dir = config_dir();
        assert!(dir.ends_with(".config/tl"));

        // Restore
        if let Some(val) = original {
            unsafe { std::env::set_var("XDG_CONFIG_HOME", val) };
        }
    }

    #[test]
    fn test_config_dir_xdg_override() {
        let original = std::env::var("XDG_CONFIG_HOME").ok();
        unsafe { std::env::set_var("XDG_CONFIG_HOME", "/custom/config") };

        let dir = config_dir();
        assert_eq!(dir, PathBuf::from("/custom/config/tl"));

        // Restore
        if let Some(val) = original {
            unsafe { std::env::set_var("XDG_CONFIG_HOME", val) };
        } else {
            unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        }
    }

    #[test]
    fn test_cache_dir_default() {
        // Clear XDG_CACHE_HOME to test default behavior
        let original = std::env::var("XDG_CACHE_HOME").ok();
        unsafe { std::env::remove_var("XDG_CACHE_HOME") };

        let dir = cache_dir();
        assert!(dir.ends_with(".cache/tl"));

        // Restore
        if let Some(val) = original {
            unsafe { std::env::set_var("XDG_CACHE_HOME", val) };
        }
    }

    #[test]
    fn test_cache_dir_xdg_override() {
        let original = std::env::var("XDG_CACHE_HOME").ok();
        unsafe { std::env::set_var("XDG_CACHE_HOME", "/custom/cache") };

        let dir = cache_dir();
        assert_eq!(dir, PathBuf::from("/custom/cache/tl"));

        // Restore
        if let Some(val) = original {
            unsafe { std::env::set_var("XDG_CACHE_HOME", val) };
        } else {
            unsafe { std::env::remove_var("XDG_CACHE_HOME") };
        }
    }
}
