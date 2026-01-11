//! Config priority contract tests.
//!
//! These tests verify that CLI options take priority over config file settings.
//! Priority order (highest to lowest):
//! 1. CLI arguments
//! 2. Config file defaults
//! 3. Built-in defaults

use std::collections::HashMap;
use tl_cli::config::{
    ConfigFile, CustomStyle, ProviderConfig, ResolveOptions, TlConfig, resolve_config,
};

fn make_config_with_defaults() -> ConfigFile {
    let mut providers = HashMap::new();
    providers.insert(
        "test_provider".to_string(),
        ProviderConfig {
            endpoint: "http://test.local".to_string(),
            api_key: Some("test_key".to_string()),
            api_key_env: None,
            models: vec!["test_model".to_string()],
        },
    );

    let mut styles = HashMap::new();
    styles.insert(
        "custom_style".to_string(),
        CustomStyle {
            description: "Test custom style".to_string(),
            prompt: "Test prompt".to_string(),
        },
    );

    ConfigFile {
        tl: TlConfig {
            provider: Some("test_provider".to_string()),
            model: Some("config_model".to_string()),
            to: Some("ja".to_string()),
            style: Some("formal".to_string()),
        },
        providers,
        styles,
    }
}

#[test]
fn test_cli_style_overrides_config_style() {
    let config = make_config_with_defaults();
    let options = ResolveOptions {
        to: None,
        provider: None,
        model: None,
        style: Some("casual".to_string()), // CLI specifies casual
    };

    let resolved = resolve_config(&options, &config).unwrap();

    // CLI style should override config style
    assert_eq!(resolved.style_name, Some("casual".to_string()));
    assert!(resolved.style_prompt.is_some());
    assert!(resolved.style_prompt.as_ref().unwrap().contains("casual"));
}

#[test]
fn test_cli_style_can_use_custom_style() {
    let config = make_config_with_defaults();
    let options = ResolveOptions {
        to: None,
        provider: None,
        model: None,
        style: Some("custom_style".to_string()), // CLI specifies custom style
    };

    let resolved = resolve_config(&options, &config).unwrap();

    assert_eq!(resolved.style_name, Some("custom_style".to_string()));
    assert_eq!(resolved.style_prompt, Some("Test prompt".to_string()));
}

#[test]
fn test_config_style_used_when_cli_not_specified() {
    let config = make_config_with_defaults();
    let options = ResolveOptions {
        to: None,
        provider: None,
        model: None,
        style: None, // CLI doesn't specify style
    };

    let resolved = resolve_config(&options, &config).unwrap();

    // Should use config's default style (formal)
    assert_eq!(resolved.style_name, Some("formal".to_string()));
    assert!(resolved.style_prompt.is_some());
}

#[test]
fn test_cli_to_overrides_config_to() {
    let config = make_config_with_defaults();
    let options = ResolveOptions {
        to: Some("en".to_string()), // CLI specifies English
        provider: None,
        model: None,
        style: None,
    };

    let resolved = resolve_config(&options, &config).unwrap();

    // CLI target language should override config
    assert_eq!(resolved.target_language, "en");
}

#[test]
fn test_cli_model_overrides_config_model() {
    let config = make_config_with_defaults();
    let options = ResolveOptions {
        to: None,
        provider: None,
        model: Some("cli_model".to_string()), // CLI specifies model
        style: None,
    };

    let resolved = resolve_config(&options, &config).unwrap();

    // CLI model should override config
    assert_eq!(resolved.model, "cli_model");
}

#[test]
fn test_cli_provider_overrides_config_provider() {
    let mut config = make_config_with_defaults();
    config.providers.insert(
        "other_provider".to_string(),
        ProviderConfig {
            endpoint: "http://other.local".to_string(),
            api_key: Some("other_key".to_string()),
            api_key_env: None,
            models: vec!["other_model".to_string()],
        },
    );

    let options = ResolveOptions {
        to: None,
        provider: Some("other_provider".to_string()), // CLI specifies different provider
        model: None,
        style: None,
    };

    let resolved = resolve_config(&options, &config).unwrap();

    // CLI provider should override config
    assert_eq!(resolved.provider_name, "other_provider");
    assert_eq!(resolved.endpoint, "http://other.local");
}

#[test]
fn test_invalid_style_returns_error() {
    let config = make_config_with_defaults();
    let options = ResolveOptions {
        to: None,
        provider: None,
        model: None,
        style: Some("nonexistent_style".to_string()),
    };

    let result = resolve_config(&options, &config);
    assert!(result.is_err());
}

#[test]
fn test_all_cli_options_override_config() {
    let mut config = make_config_with_defaults();
    config.providers.insert(
        "cli_provider".to_string(),
        ProviderConfig {
            endpoint: "http://cli.local".to_string(),
            api_key: Some("cli_key".to_string()),
            api_key_env: None,
            models: vec!["cli_model".to_string()],
        },
    );

    let options = ResolveOptions {
        to: Some("zh".to_string()),
        provider: Some("cli_provider".to_string()),
        model: Some("cli_specified_model".to_string()),
        style: Some("literal".to_string()),
    };

    let resolved = resolve_config(&options, &config).unwrap();

    // All CLI options should override config
    assert_eq!(resolved.target_language, "zh");
    assert_eq!(resolved.provider_name, "cli_provider");
    assert_eq!(resolved.model, "cli_specified_model");
    assert_eq!(resolved.style_name, Some("literal".to_string()));
}
