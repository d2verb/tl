//! Server-Sent Events (SSE) parser for OpenAI-compatible streaming responses.
//!
//! This module provides utilities for parsing SSE streams from chat completion APIs.

use anyhow::Result;
use bytes::Bytes;
use futures_util::Stream;
use serde::Deserialize;

/// Response structure for streaming chat completions.
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

/// Converts a raw SSE byte stream into a stream of text chunks.
///
/// Handles buffering, line parsing, and SSE protocol details.
///
/// # Arguments
///
/// * `byte_stream` - A stream of raw bytes from an HTTP response
///
/// # Returns
///
/// A stream that yields extracted text content from each SSE data event.
pub fn sse_to_text_stream(
    byte_stream: impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static,
) -> impl Stream<Item = Result<String>> + Send {
    async_stream::stream! {
        use futures_util::StreamExt;

        let mut byte_stream = std::pin::pin!(byte_stream);
        let mut buffer = String::new();

        while let Some(chunk_result) = byte_stream.next().await {
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

                if let Some(content) = parse_sse_line(line.trim()) {
                    yield Ok(content);
                } else if line.trim() == "data: [DONE]" {
                    return;
                }
            }
        }
    }
}

/// Parses a single SSE line and extracts the text content.
///
/// # Arguments
///
/// * `line` - A trimmed SSE line (e.g., `data: {"choices":[...]}`)
///
/// # Returns
///
/// * `Some(content)` - The extracted text content if the line contains valid data
/// * `None` - For non-data lines, empty content, or parse errors
///
/// # Example
///
/// ```ignore
/// let line = r#"data: {"choices":[{"delta":{"content":"Hello"}}]}"#;
/// assert_eq!(parse_sse_line(line), Some("Hello".to_string()));
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line_with_content() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"}}]}"#;
        assert_eq!(parse_sse_line(line), Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_sse_line_with_empty_content() {
        let line = r#"data: {"choices":[{"delta":{"content":""}}]}"#;
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn test_parse_sse_line_with_null_content() {
        let line = r#"data: {"choices":[{"delta":{}}]}"#;
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn test_parse_sse_line_multiple_choices() {
        let line =
            r#"data: {"choices":[{"delta":{"content":"Hello"}},{"delta":{"content":" World"}}]}"#;
        assert_eq!(parse_sse_line(line), Some("Hello World".to_string()));
    }

    #[test]
    fn test_parse_sse_line_no_data_prefix() {
        let line = r#"{"choices":[{"delta":{"content":"Hello"}}]}"#;
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn test_parse_sse_line_invalid_json() {
        let line = "data: not json";
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn test_parse_sse_line_done_marker() {
        let line = "data: [DONE]";
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn test_parse_sse_line_empty_line() {
        assert_eq!(parse_sse_line(""), None);
    }

    #[test]
    fn test_parse_sse_line_comment() {
        let line = ": this is a comment";
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn test_parse_sse_line_unicode_content() {
        let line = r#"data: {"choices":[{"delta":{"content":"こんにちは"}}]}"#;
        assert_eq!(parse_sse_line(line), Some("こんにちは".to_string()));
    }
}
