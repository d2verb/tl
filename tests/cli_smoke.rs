#![allow(clippy::unwrap_used)]
//! CLI smoke tests to verify basic command functionality.
//!
//! These tests ensure that the CLI binary starts correctly and
//! responds to basic commands without crashing.

use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn tl() -> Command {
    Command::cargo_bin("tl").unwrap()
}

#[test]
fn test_help_displays_usage() {
    tl().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI-powered translation CLI tool"))
        .stdout(predicate::str::contains("--to"))
        .stdout(predicate::str::contains("--style"))
        .stdout(predicate::str::contains("--provider"));
}

#[test]
fn test_version_displays_version() {
    tl().arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_styles_list_shows_presets() {
    tl().arg("styles")
        .assert()
        .success()
        .stdout(predicate::str::contains("Preset styles"))
        .stdout(predicate::str::contains("casual"))
        .stdout(predicate::str::contains("formal"))
        .stdout(predicate::str::contains("literal"))
        .stdout(predicate::str::contains("natural"));
}

#[test]
fn test_styles_show_preset() {
    tl().args(["styles", "show", "casual"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Preset style"))
        .stdout(predicate::str::contains("casual"))
        .stdout(predicate::str::contains("Prompt:"));
}

#[test]
fn test_styles_show_nonexistent() {
    tl().args(["styles", "show", "nonexistent_style_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_languages_list() {
    tl().arg("languages")
        .assert()
        .success()
        .stdout(predicate::str::contains("en"))
        .stdout(predicate::str::contains("ja"))
        .stdout(predicate::str::contains("zh"));
}

#[test]
fn test_providers_list_without_config() {
    // Without config, should show "No providers configured"
    tl().arg("providers").assert().success();
}

#[test]
fn test_invalid_language_code() {
    tl().args(["--to", "invalid_lang_xyz"])
        .write_stdin("hello")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid language code"));
}

#[test]
fn test_chat_help() {
    tl().args(["chat", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--to"))
        .stdout(predicate::str::contains("--style"))
        .stdout(predicate::str::contains("--provider"))
        .stdout(predicate::str::contains("--model"));
}

#[test]
fn test_styles_help() {
    tl().args(["styles", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("edit"))
        .stdout(predicate::str::contains("remove"));
}
