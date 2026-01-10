use anyhow::{Result, bail};
use futures_util::StreamExt;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::cache::CacheManager;
use crate::config::{ConfigManager, ResolveOptions, resolve_config};
use crate::input::InputReader;
use crate::translation::{TranslationClient, TranslationRequest};
use crate::ui::Spinner;

/// Write content to file atomically using a temp file and rename.
/// This prevents file corruption if interrupted (e.g., Ctrl+C).
fn atomic_write(file_path: &str, content: &str) -> Result<()> {
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

/// Options for the translate command.
pub struct TranslateOptions {
    /// Input file path (reads from stdin if `None`).
    pub file: Option<String>,
    /// Target language code.
    pub to: Option<String>,
    /// Provider name.
    pub provider: Option<String>,
    /// Model name.
    pub model: Option<String>,
    /// Whether to bypass the cache.
    pub no_cache: bool,
    /// Whether to overwrite the input file with the translation.
    pub write: bool,
}

/// Runs the translate command.
///
/// Translates input from a file or stdin and outputs the result.
/// Supports caching and streaming output.
pub async fn run_translate(options: TranslateOptions) -> Result<()> {
    // Validate -w option requires a file
    if options.write && options.file.is_none() {
        bail!("Error: --write requires a file argument (cannot write to stdin)");
    }

    let manager = ConfigManager::new()?;
    let config_file = manager.load_or_default();
    let resolve_options = ResolveOptions {
        to: options.to.clone(),
        provider: options.provider.clone(),
        model: options.model.clone(),
    };
    let resolved = resolve_config(&resolve_options, &config_file)?;

    let source_text = InputReader::read(options.file.as_deref())?;

    if source_text.is_empty() {
        bail!("Error: Input is empty");
    }

    let cache_manager = CacheManager::new()?;

    // Create request first, moving values where possible
    // Only endpoint needs clone (used by both client and request)
    let request = TranslationRequest {
        source_text,
        target_language: resolved.target_language,
        model: resolved.model,
        endpoint: resolved.endpoint.clone(),
    };

    // Create client with remaining values (endpoint cloned, api_key moved)
    let client = TranslationClient::new(resolved.endpoint, resolved.api_key);

    if !options.no_cache
        && let Some(cached) = cache_manager.get(&request)?
    {
        if options.write {
            if let Some(ref file_path) = options.file {
                atomic_write(file_path, &cached)?;
            }
        } else {
            print!("{cached}");
            io::stdout().flush()?;
        }
        return Ok(());
    }

    let spinner_msg = if options.write {
        format!(
            "Translating {}...",
            options.file.as_deref().unwrap_or("file")
        )
    } else {
        "Translating...".to_string()
    };
    let spinner = Spinner::new(&spinner_msg);

    let mut stream = client.translate_stream(&request).await?;
    let mut full_response = String::new();
    let mut spinner_active = true;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;

        // When streaming to stdout, stop spinner on first chunk to show output
        // When writing to file, keep spinner until completion
        if spinner_active && !options.write {
            spinner.stop();
            spinner_active = false;
        }

        if !options.write {
            print!("{chunk}");
            io::stdout().flush()?;
        }
        full_response.push_str(&chunk);
    }

    if spinner_active {
        spinner.stop();
    }

    if !options.write && !full_response.is_empty() {
        println!();
    }

    if !options.no_cache && !full_response.is_empty() {
        cache_manager.put(&request, &full_response)?;
    }

    // Write to file if -w is specified
    if options.write
        && !full_response.is_empty()
        && let Some(ref file_path) = options.file
    {
        atomic_write(file_path, &full_response)?;
    }

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
