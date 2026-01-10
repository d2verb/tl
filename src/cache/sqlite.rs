use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

use crate::translation::TranslationRequest;

pub struct CacheManager {
    db_path: PathBuf,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to determine cache directory")?
            .join("tl");

        std::fs::create_dir_all(&cache_dir).with_context(|| {
            format!("Failed to create cache directory: {}", cache_dir.display())
        })?;

        let db_path = cache_dir.join("translations.db");
        let manager = Self { db_path };

        manager.init_db()?;

        Ok(manager)
    }

    fn init_db(&self) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS translations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cache_key TEXT UNIQUE NOT NULL,
                source_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                target_language TEXT NOT NULL,
                model TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                prompt_hash TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .context("Failed to create translations table")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cache_key ON translations(cache_key)",
            [],
        )
        .context("Failed to create index")?;

        Ok(())
    }

    fn connect(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
            .with_context(|| format!("Failed to open cache database: {}", self.db_path.display()))
    }

    pub fn get(&self, request: &TranslationRequest) -> Result<Option<String>> {
        let cache_key = request.cache_key();
        let conn = self.connect()?;

        let mut stmt =
            conn.prepare("SELECT translated_text FROM translations WHERE cache_key = ?1")?;

        let result: Option<String> = stmt.query_row([&cache_key], |row| row.get(0)).ok();

        if result.is_some() {
            conn.execute(
                "UPDATE translations SET accessed_at = CURRENT_TIMESTAMP WHERE cache_key = ?1",
                [&cache_key],
            )?;
        }

        Ok(result)
    }

    pub fn put(&self, request: &TranslationRequest, translated_text: &str) -> Result<()> {
        let cache_key = request.cache_key();
        let prompt_hash = TranslationRequest::prompt_hash();
        let conn = self.connect()?;

        conn.execute(
            "INSERT OR REPLACE INTO translations
             (cache_key, source_text, translated_text, target_language, model, endpoint, prompt_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &cache_key,
                &request.source_text,
                translated_text,
                &request.target_language,
                &request.model,
                &request.endpoint,
                &prompt_hash,
            ],
        )
        .context("Failed to insert translation into cache")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager(temp_dir: &TempDir) -> CacheManager {
        let db_path = temp_dir.path().join("translations.db");
        let manager = CacheManager { db_path };
        manager.init_db().unwrap();
        manager
    }

    fn create_test_request() -> TranslationRequest {
        TranslationRequest {
            source_text: "Hello, World!".to_string(),
            target_language: "ja".to_string(),
            model: "gpt-oss:20b".to_string(),
            endpoint: "http://localhost:11434".to_string(),
        }
    }

    #[test]
    fn test_cache_miss() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);
        let request = create_test_request();

        let result = manager.get(&request).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_hit() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);
        let request = create_test_request();

        manager.put(&request, "こんにちは、世界！").unwrap();

        let result = manager.get(&request).unwrap();
        assert_eq!(result, Some("こんにちは、世界！".to_string()));
    }

    #[test]
    fn test_different_requests_different_keys() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);

        let request1 = TranslationRequest {
            source_text: "Hello".to_string(),
            target_language: "ja".to_string(),
            model: "model1".to_string(),
            endpoint: "http://localhost:11434".to_string(),
        };

        let request2 = TranslationRequest {
            source_text: "Hello".to_string(),
            target_language: "en".to_string(),
            model: "model1".to_string(),
            endpoint: "http://localhost:11434".to_string(),
        };

        manager.put(&request1, "Translation 1").unwrap();
        manager.put(&request2, "Translation 2").unwrap();

        assert_eq!(
            manager.get(&request1).unwrap(),
            Some("Translation 1".to_string())
        );
        assert_eq!(
            manager.get(&request2).unwrap(),
            Some("Translation 2".to_string())
        );
    }

    #[test]
    fn test_cache_key_includes_endpoint() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(&temp_dir);

        let request1 = TranslationRequest {
            source_text: "Hello".to_string(),
            target_language: "ja".to_string(),
            model: "model1".to_string(),
            endpoint: "http://localhost:11434".to_string(),
        };

        let request2 = TranslationRequest {
            source_text: "Hello".to_string(),
            target_language: "ja".to_string(),
            model: "model1".to_string(),
            endpoint: "http://production:11434".to_string(),
        };

        manager.put(&request1, "Local Translation").unwrap();
        manager.put(&request2, "Production Translation").unwrap();

        assert_eq!(
            manager.get(&request1).unwrap(),
            Some("Local Translation".to_string())
        );
        assert_eq!(
            manager.get(&request2).unwrap(),
            Some("Production Translation".to_string())
        );
    }
}
