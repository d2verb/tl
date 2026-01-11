//! Chat mode command handler.

use anyhow::Result;

use super::load_config;
use crate::chat::{ChatSession, SessionConfig};
use crate::config::{ResolveOptions, resolve_config};

/// Options for the chat command.
pub struct ChatOptions {
    /// Target language code.
    pub to: Option<String>,
    /// Provider name.
    pub provider: Option<String>,
    /// Model name.
    pub model: Option<String>,
    /// Translation style.
    pub style: Option<String>,
}

/// Runs the interactive chat mode.
///
/// Starts a REPL-style session for translating text interactively.
pub async fn run_chat(options: ChatOptions) -> Result<()> {
    let (_manager, config_file) = load_config()?;

    let resolve_options = ResolveOptions {
        to: options.to,
        provider: options.provider,
        model: options.model,
        style: options.style,
    };

    let resolved = resolve_config(&resolve_options, &config_file)?;

    let session_config = SessionConfig::new(resolved, config_file.styles.clone());

    let mut session = ChatSession::new(session_config);
    session.run().await
}
