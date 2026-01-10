use anyhow::{Context, Result};
use futures_util::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use super::prompt::build_system_prompt;

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub source_text: String,
    pub target_language: String,
    pub model: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
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

        let chat_request = ChatCompletionRequest {
            model: request.model.clone(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: build_system_prompt(&request.target_language),
                },
                Message {
                    role: "user".into(),
                    content: request.source_text.clone(),
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
