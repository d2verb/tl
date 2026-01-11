//! File system utilities.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Writes content to a file atomically using a temp file and rename.
///
/// This prevents file corruption if the process is interrupted (e.g., Ctrl+C).
/// The temp file is created in the same directory as the target file to ensure
/// the rename operation is atomic (same filesystem).
///
/// # Errors
///
/// Returns an error if the temp file cannot be written or renamed.
pub fn atomic_write(file_path: &str, content: &str) -> Result<()> {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let temp_path = parent.join(format!(".{file_name}.tmp"));

    // Write to temp file first
    fs::write(&temp_path, content)?;

    // Atomic rename (same filesystem)
    fs::rename(&temp_path, file_path)?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_write_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        atomic_write(file_path_str, "Hello, World!").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        fs::write(&file_path, "Original content").unwrap();
        atomic_write(file_path_str, "New content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "New content");
    }

    #[test]
    fn test_atomic_write_no_temp_file_remains() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        atomic_write(file_path_str, "content").unwrap();

        // Temp file should not exist after successful write
        let temp_path = temp_dir.path().join(".test.txt.tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_atomic_write_unicode_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();

        let content = "こんにちは世界！🌍";
        atomic_write(file_path_str, content).unwrap();

        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
    }
}
