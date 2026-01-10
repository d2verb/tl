use anyhow::{Context, Result};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::pin::Pin;

use super::prompt::{SYSTEM_PROMPT_TEMPLATE, build_system_prompt};

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub source_text: String,
    pub target_language: String,
    pub model: String,
    pub endpoint: String,
}

impl TranslationRequest {
    /// Compute cache key for this request
    pub fn cache_key(&self) -> String {
        let prompt_hash = Self::prompt_hash();

        let cache_input = serde_json::json!({
            "source_text": self.source_text,
            "target_language": self.target_language,
            "model": self.model,
            "endpoint": self.endpoint,
            "prompt_hash": prompt_hash
        });

        let mut hasher = Sha256::new();
        hasher.update(cache_input.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Compute hash of the system prompt template
    pub fn prompt_hash() -> String {
        let mut hasher = Sha256::new();
        hasher.update(SYSTEM_PROMPT_TEMPLATE.as_bytes());
        hex::encode(hasher.finalize())
    }
}

// Use Cow to avoid cloning strings that are only borrowed for serialization
#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'static str,
    content: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

pub struct TranslationClient {
    client: Client,
    endpoint: String,
    api_key: Option<String>,
}

impl TranslationClient {
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            api_key,
        }
    }

    pub async fn translate_stream(
        &self,
        request: &TranslationRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let url = format!(
            "{}/v1/chat/completions",
            self.endpoint.trim_end_matches('/')
        );

        // Build system prompt once (returns owned String)
        let system_prompt = build_system_prompt(&request.target_language);

        let chat_request = ChatCompletionRequest {
            model: &request.model,
            messages: vec![
                Message {
                    role: "system",
                    content: Cow::Owned(system_prompt),
                },
                Message {
                    role: "user",
                    content: Cow::Borrowed(&request.source_text),
                },
            ],
            stream: true,
        };

        let mut http_request = self.client.post(&url).json(&chat_request);

        // Add Authorization header if API key is present
        if let Some(api_key) = &self.api_key {
            http_request = http_request.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = http_request
            .send()
            .await
            .with_context(|| format!("Failed to connect to API endpoint: {url}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {status}: {body}");
        }

        let mut stream = response.bytes_stream();

        let mapped_stream = async_stream::stream! {
            use futures_util::StreamExt;
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Stream error: {e}"));
                        continue;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line: String = buffer.drain(..=line_end).collect();
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }
                    if line == "data: [DONE]" {
                        return;
                    }

                    if let Some(content) = parse_sse_line(line) {
                        yield Ok(content);
                    }
                }
            }
        };

        Ok(Box::pin(mapped_stream))
    }
}

fn parse_sse_line(line: &str) -> Option<String> {
    let json_str = line.strip_prefix("data: ")?;

    let response = serde_json::from_str::<StreamResponse>(json_str).ok()?;

    let content: String = response
        .choices
        .into_iter()
        .filter_map(|c| c.delta.content)
        .filter(|c| !c.is_empty())
        .collect();

    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}
