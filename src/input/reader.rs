use anyhow::{Context, Result, bail};
use std::fs;
use std::io::{self, Read};

const MAX_INPUT_SIZE: usize = 1024 * 1024; // 1MB

pub struct InputReader;

impl InputReader {
    pub fn read(file_path: Option<&str>) -> Result<String> {
        file_path.map_or_else(Self::read_stdin, Self::read_file)
    }

    fn read_file(path: &str) -> Result<String> {
        let metadata =
            fs::metadata(path).with_context(|| format!("Failed to access file: {path}"))?;

        let size = metadata.len() as usize;
        if size > MAX_INPUT_SIZE {
            bail!(
                "Error: Input size ({:.1} MB) exceeds maximum allowed size (1 MB).\n\n\
                 Consider splitting the file into smaller parts.",
                size as f64 / 1024.0 / 1024.0
            );
        }

        fs::read_to_string(path).with_context(|| format!("Failed to read file: {path}"))
    }

    #[allow(clippy::significant_drop_tightening)]
    fn read_stdin() -> Result<String> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];
        let mut stdin = io::stdin().lock();

        loop {
            let bytes_read = stdin
                .read(&mut chunk)
                .context("Failed to read from stdin")?;

            if bytes_read == 0 {
                break;
            }

            buffer.extend_from_slice(&chunk[..bytes_read]);

            if buffer.len() > MAX_INPUT_SIZE {
                bail!(
                    "Error: Input size ({:.1} MB) exceeds maximum allowed size (1 MB).\n\n\
                     Consider splitting the input into smaller parts.",
                    buffer.len() as f64 / 1024.0 / 1024.0
                );
            }
        }

        String::from_utf8(buffer).context("Input is not valid UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_read_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();

        let content = InputReader::read(Some(temp_file.path().to_str().unwrap())).unwrap();
        assert_eq!(content.trim(), "Hello, World!");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = InputReader::read(Some("/nonexistent/path/to/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_max_input_size_constant() {
        assert_eq!(MAX_INPUT_SIZE, 1024 * 1024);
    }

    #[test]
    fn test_read_file_unicode() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "こんにちは世界！🌍\n日本語テスト";
        write!(temp_file, "{}", content).unwrap();

        let result = InputReader::read(Some(temp_file.path().to_str().unwrap())).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_read_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();

        let content = InputReader::read(Some(temp_file.path().to_str().unwrap())).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_read_file_exceeds_max_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large_file.txt");

        // Create a file larger than MAX_INPUT_SIZE (1MB + 1 byte)
        let large_content = "x".repeat(MAX_INPUT_SIZE + 1);
        fs::write(&file_path, &large_content).unwrap();

        let result = InputReader::read(Some(file_path.to_str().unwrap()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_read_file_at_max_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("max_file.txt");

        // Create a file exactly at MAX_INPUT_SIZE
        let content = "x".repeat(MAX_INPUT_SIZE);
        fs::write(&file_path, &content).unwrap();

        let result = InputReader::read(Some(file_path.to_str().unwrap()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), MAX_INPUT_SIZE);
    }

    #[test]
    fn test_read_file_multiline() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Line 1\nLine 2\nLine 3";
        write!(temp_file, "{}", content).unwrap();

        let result = InputReader::read(Some(temp_file.path().to_str().unwrap())).unwrap();
        assert_eq!(result, content);
    }
}
