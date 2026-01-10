//! Chat mode command handler.

use anyhow::Result;

use super::translate::{TranslateOptions, resolve_config};
use crate::chat::{ChatSession, SessionConfig};
use crate::config::ConfigManager;

/// Options for the chat command.
pub struct ChatOptions {
    /// Target language code.
    pub to: Option<String>,
    /// Provider name.
    pub provider: Option<String>,
    /// Model name.
    pub model: Option<String>,
}

/// Runs the interactive chat mode.
///
/// Starts a REPL-style session for translating text interactively.
pub async fn run_chat(options: ChatOptions) -> Result<()> {
    let manager = ConfigManager::new()?;
    let config_file = manager.load_or_default();

    // Reuse resolve_config from translate
    let translate_options = TranslateOptions {
        file: None,
        to: options.to,
        provider: options.provider,
        model: options.model,
        no_cache: false,
        write: false,
    };

    let resolved = resolve_config(&translate_options, &config_file)?;

    let session_config = SessionConfig::new(
        resolved.provider_name,
        resolved.endpoint,
        resolved.model,
        resolved.api_key,
        resolved.target_language,
    );

    let mut session = ChatSession::new(session_config);
    session.run().await
}
