use anyhow::{Context, Result};
use bytes::Bytes;
use futures_util::Stream;
use reqwest::Client;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::pin::Pin;

use super::prompt::{SYSTEM_PROMPT_TEMPLATE, build_system_prompt_with_style};
use super::sse_parser::sse_to_text_stream;

/// A request to translate text.
///
/// Contains all parameters needed to perform a translation and compute
/// a unique cache key.
#[derive(Debug, Clone)]
pub struct TranslationRequest {
    /// The text to translate.
    pub source_text: String,
    /// The target language (ISO 639-1 code, e.g., "ja", "en").
    pub target_language: String,
    /// The model to use for translation.
    pub model: String,
    /// The API endpoint URL.
    pub endpoint: String,
    /// The translation style prompt (if specified).
    pub style: Option<String>,
}

impl TranslationRequest {
    /// Computes a unique cache key for this request.
    ///
    /// The key is a SHA-256 hash of the source text, target language,
    /// model, endpoint, style, and prompt template hash.
    pub fn cache_key(&self) -> String {
        let prompt_hash = Self::prompt_hash();

        let cache_input = serde_json::json!({
            "source_text": self.source_text,
            "target_language": self.target_language,
            "model": self.model,
            "endpoint": self.endpoint,
            "prompt_hash": prompt_hash,
            "style": self.style
        });

        let mut hasher = Sha256::new();
        hasher.update(cache_input.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Computes a hash of the system prompt template.
    ///
    /// Used to invalidate cache when the prompt changes.
    pub fn prompt_hash() -> String {
        let mut hasher = Sha256::new();
        hasher.update(SYSTEM_PROMPT_TEMPLATE.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Request body for the chat completions API.
#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
}

impl<'a> ChatCompletionRequest<'a> {
    /// Builds a chat completion request for translation.
    fn for_translation(model: &'a str, system_prompt: &'a str, source_text: &'a str) -> Self {
        Self {
            model,
            messages: vec![
                Message {
                    role: "system",
                    content: Cow::Borrowed(system_prompt),
                },
                Message {
                    role: "user",
                    content: Cow::Borrowed(source_text),
                },
            ],
            stream: true,
        }
    }
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'static str,
    content: Cow<'a, str>,
}

/// Client for translating text using OpenAI-compatible APIs.
///
/// Supports streaming responses for real-time output.
///
/// # Example
///
/// ```no_run
/// use tl_cli::translation::{TranslationClient, TranslationRequest};
/// use futures_util::StreamExt;
///
/// # async fn example() -> anyhow::Result<()> {
/// let client = TranslationClient::new(
///     "http://localhost:11434".to_string(),
///     None,
/// );
///
/// let request = TranslationRequest {
///     source_text: "Hello, world!".to_string(),
///     target_language: "ja".to_string(),
///     model: "gemma3:12b".to_string(),
///     endpoint: "http://localhost:11434".to_string(),
///     style: None,
/// };
///
/// let mut stream = client.translate_stream(&request).await?;
/// while let Some(chunk) = stream.next().await {
///     print!("{}", chunk?);
/// }
/// # Ok(())
/// # }
/// ```
pub struct TranslationClient {
    client: Client,
    endpoint: String,
    api_key: Option<String>,
}

impl TranslationClient {
    /// Creates a new translation client.
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            api_key,
        }
    }

    /// Translates text and returns a stream of response chunks.
    ///
    /// The stream yields chunks of the translated text as they arrive,
    /// enabling real-time display of the translation.
    pub async fn translate_stream(
        &self,
        request: &TranslationRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let byte_stream = self
            .send_chat_completion(
                &request.model,
                &request.target_language,
                &request.source_text,
                request.style.as_deref(),
            )
            .await?;

        Ok(Box::pin(sse_to_text_stream(byte_stream)))
    }

    /// Sends a chat completion request and returns the raw byte stream.
    async fn send_chat_completion(
        &self,
        model: &str,
        target_language: &str,
        source_text: &str,
        style: Option<&str>,
    ) -> Result<impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static> {
        let url = self.build_url();
        let system_prompt = build_system_prompt_with_style(target_language, style);
        let chat_request =
            ChatCompletionRequest::for_translation(model, &system_prompt, source_text);

        let response = self.send_request(&url, &chat_request).await?;

        Ok(response.bytes_stream())
    }

    /// Sends an HTTP POST request with optional authorization.
    async fn send_request<T: Serialize + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        let mut request = self.client.post(url).json(body);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to connect to API endpoint: {url}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {status}: {body}");
        }

        Ok(response)
    }

    /// Builds the chat completions API URL.
    fn build_url(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.endpoint.trim_end_matches('/')
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> TranslationRequest {
        TranslationRequest {
            source_text: "Hello, world!".to_string(),
            target_language: "ja".to_string(),
            model: "gemma3:12b".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            style: None,
        }
    }

    #[test]
    fn test_cache_key_is_consistent() {
        let request = create_test_request();
        let key1 = request.cache_key();
        let key2 = request.cache_key();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_is_hex_string() {
        let request = create_test_request();
        let key = request.cache_key();
        // SHA-256 produces 64 hex characters
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cache_key_differs_for_different_source_text() {
        let request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.source_text = "Different text".to_string();
        assert_ne!(request1.cache_key(), request2.cache_key());
    }

    #[test]
    fn test_cache_key_differs_for_different_target_language() {
        let request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.target_language = "en".to_string();
        assert_ne!(request1.cache_key(), request2.cache_key());
    }

    #[test]
    fn test_cache_key_differs_for_different_model() {
        let request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.model = "gpt-4o".to_string();
        assert_ne!(request1.cache_key(), request2.cache_key());
    }

    #[test]
    fn test_cache_key_differs_for_different_endpoint() {
        let request1 = create_test_request();
        let mut request2 = create_test_request();
        request2.endpoint = "https://api.openai.com".to_string();
        assert_ne!(request1.cache_key(), request2.cache_key());
    }

    #[test]
    fn test_prompt_hash_is_consistent() {
        let hash1 = TranslationRequest::prompt_hash();
        let hash2 = TranslationRequest::prompt_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_prompt_hash_is_hex_string() {
        let hash = TranslationRequest::prompt_hash();
        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_translation_client_new() {
        let client = TranslationClient::new(
            "http://localhost:11434".to_string(),
            Some("test-api-key".to_string()),
        );
        assert_eq!(client.endpoint, "http://localhost:11434");
        assert_eq!(client.api_key, Some("test-api-key".to_string()));
    }

    #[test]
    fn test_translation_client_new_without_api_key() {
        let client = TranslationClient::new("http://localhost:11434".to_string(), None);
        assert_eq!(client.endpoint, "http://localhost:11434");
        assert!(client.api_key.is_none());
    }

    #[test]
    fn test_build_url_without_trailing_slash() {
        let client = TranslationClient::new("http://localhost:11434".to_string(), None);
        assert_eq!(
            client.build_url(),
            "http://localhost:11434/v1/chat/completions"
        );
    }

    #[test]
    fn test_build_url_with_trailing_slash() {
        let client = TranslationClient::new("http://localhost:11434/".to_string(), None);
        assert_eq!(
            client.build_url(),
            "http://localhost:11434/v1/chat/completions"
        );
    }
}
