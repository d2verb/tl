use anyhow::Result;

use crate::chat::{ChatSession, SessionConfig};
use crate::config::ConfigManager;

pub struct ChatOptions {
    pub to: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
}

pub async fn run_chat(options: ChatOptions) -> Result<()> {
    let config = load_session_config(&options)?;
    let mut session = ChatSession::new(config);
    session.run().await
}

fn load_session_config(options: &ChatOptions) -> Result<SessionConfig> {
    let manager = ConfigManager::new()?;
    let file_config = manager.load().unwrap_or_default();

    let endpoint = options
        .endpoint
        .clone()
        .or(file_config.endpoint)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Error: Missing required configuration: 'endpoint'\n\n\
                 Please provide it via:\n  \
                 - CLI option: tl chat --endpoint <url>\n  \
                 - Config file: Run 'tl configure' to set up configuration"
            )
        })?;

    let model = options.model.clone().or(file_config.model).ok_or_else(|| {
        anyhow::anyhow!(
            "Error: Missing required configuration: 'model'\n\n\
             Please provide it via:\n  \
             - CLI option: tl chat --model <name>\n  \
             - Config file: Run 'tl configure' to set up configuration"
        )
    })?;

    let to = options.to.clone().or(file_config.to).ok_or_else(|| {
        anyhow::anyhow!(
            "Error: Missing required configuration: 'to' (target language)\n\n\
             Please provide it via:\n  \
             - CLI option: tl chat --to <lang>\n  \
             - Config file: Run 'tl configure' to set up configuration"
        )
    })?;

    Ok(SessionConfig::new(to, endpoint, model))
}
