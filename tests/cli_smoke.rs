#![allow(clippy::unwrap_used)]
//! CLI smoke tests to verify basic command functionality.
//!
//! These tests ensure that the CLI binary starts correctly and
//! responds to basic commands without crashing.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

#[allow(deprecated)]
fn tl() -> Command {
    Command::cargo_bin("tl").unwrap()
}

/// Create a command with a temporary config directory containing a minimal config.
/// This is needed for tests that require config resolution to succeed.
#[allow(deprecated)]
fn tl_with_config() -> (Command, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join("tl");
    std::fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.toml");
    let mut file = std::fs::File::create(&config_path).unwrap();
    writeln!(
        file,
        r#"
[tl]
provider = "test"
model = "test-model"
to = "ja"

[providers.test]
endpoint = "http://localhost:11434"
models = ["test-model"]
"#
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("tl").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    (cmd, temp_dir)
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

#[test]
fn test_quiet_flag_available() {
    tl().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--quiet"))
        .stdout(predicate::str::contains("-q"));
}

#[test]
fn test_no_color_flag_available() {
    tl().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--no-color"))
        .stdout(predicate::str::contains("NO_COLOR"));
}

#[test]
fn test_quiet_flag_works() {
    // Quiet flag should not cause errors
    tl().args(["--quiet", "providers"]).assert().success();
}

#[test]
fn test_no_color_flag_works() {
    // No-color flag should not cause errors
    tl().args(["--no-color", "providers"]).assert().success();
}

#[test]
fn test_global_flags_with_subcommand() {
    // Global flags should work with subcommands
    tl().args(["--quiet", "--no-color", "languages"])
        .assert()
        .success();
}

#[test]
fn test_exit_code_invalid_language() {
    // Invalid language should return exit code 64 (USAGE - sysexits.h)
    tl().args(["--to", "invalid_xyz"])
        .write_stdin("test")
        .assert()
        .code(exitcode::USAGE);
}

#[test]
fn test_exit_code_file_not_found() {
    // File not found should return exit code 66 (NOINPUT - sysexits.h)
    // Need a valid config so that config resolution doesn't fail first
    let (mut cmd, _temp_dir) = tl_with_config();
    cmd.arg("/nonexistent/path/to/file.txt")
        .assert()
        .code(exitcode::NOINPUT);
}
