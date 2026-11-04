//! End-to-end tests that run the compiled binary.
//!
//! Every test clears the environment and runs in a fresh temporary directory
//! so nothing from the developer's machine (real `~/.config`, `/etc`, a
//! local `.env` or `config.toml`) can leak in.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const NAME: &str = env!("CARGO_PKG_NAME");

/// Upper-case environment prefix derived from the package name.
fn prefix() -> String {
    NAME.to_ascii_uppercase().replace('-', "_")
}

/// A command with a cleared environment, `HOME` pointing at `home`, and the
/// working directory set to `home`. `LLVM_PROFILE_FILE` is preserved so
/// coverage still records the child process.
fn cmd(home: &Path) -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!();
    cmd.env_clear().env("HOME", home).current_dir(home);
    if let Ok(profile) = std::env::var("LLVM_PROFILE_FILE") {
        cmd.env("LLVM_PROFILE_FILE", profile);
    }
    cmd
}

/// Writes `~/.config/<name>/config.toml` under `home`. Helpers return
/// `Result` because `unwrap()` is only allowed inside `#[test]` functions.
fn write_user_config(home: &Path, contents: &str) -> std::io::Result<()> {
    let dir = home.join(".config").join(NAME);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("config.toml"), contents)
}

#[test]
fn greets_with_defaults() {
    let home = TempDir::new().unwrap();
    cmd(home.path())
        .assert()
        .success()
        .stdout("Hello, world!\n");
}

#[test]
fn prints_version() {
    let home = TempDir::new().unwrap();
    cmd(home.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn reads_user_config_from_home() {
    let home = TempDir::new().unwrap();
    write_user_config(home.path(), "[greeting]\nmessage = \"Hi\"\n").unwrap();
    cmd(home.path()).assert().success().stdout("Hi, world!\n");
}

#[test]
fn xdg_config_home_wins_over_home() {
    let home = TempDir::new().unwrap();
    write_user_config(home.path(), "[greeting]\nmessage = \"From home\"\n").unwrap();
    let xdg = home.path().join("xdg");
    fs::create_dir_all(xdg.join(NAME)).unwrap();
    fs::write(
        xdg.join(NAME).join("config.toml"),
        "[greeting]\nmessage = \"From XDG\"\n",
    )
    .unwrap();
    cmd(home.path())
        .env("XDG_CONFIG_HOME", &xdg)
        .assert()
        .success()
        .stdout("From XDG, world!\n");
}

#[test]
fn explicit_config_flag_is_used() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("custom.toml");
    fs::write(&file, "[greeting]\nnames = [\"Ada\"]\n").unwrap();
    cmd(home.path())
        .arg("--config")
        .arg(&file)
        .assert()
        .success()
        .stdout("Hello, Ada!\n");
}

#[test]
fn missing_explicit_config_exits_with_ex_config() {
    let home = TempDir::new().unwrap();
    cmd(home.path())
        .args(["--config", "nope.toml"])
        .assert()
        .code(78)
        .stderr(predicate::str::contains(
            "nope.toml (from --config) does not exist",
        ));
}

#[test]
fn env_override_and_cli_flag_apply() {
    let home = TempDir::new().unwrap();
    cmd(home.path())
        .env(format!("{}__GREETING__REPEAT", prefix()), "2")
        .args(["greet", "--name", "Linus"])
        .assert()
        .success()
        .stdout("Hello, Linus!\nHello, Linus!\n");
}

#[test]
fn unknown_override_exits_with_ex_config() {
    let home = TempDir::new().unwrap();
    let var = format!("{}__GREETING__COLOUR", prefix());
    cmd(home.path())
        .env(&var, "red")
        .assert()
        .code(78)
        .stderr(predicate::str::contains(var));
}

#[test]
fn dotenv_supplies_values_but_process_env_wins() {
    let home = TempDir::new().unwrap();
    let var = format!("{}__GREETING__MESSAGE", prefix());
    fs::write(home.path().join(".env"), format!("{var}=Dotenv\n")).unwrap();
    cmd(home.path())
        .assert()
        .success()
        .stdout("Dotenv, world!\n");
    cmd(home.path())
        .env(&var, "Process")
        .assert()
        .success()
        .stdout("Process, world!\n");
}

#[test]
fn malformed_dotenv_exits_with_ex_config() {
    let home = TempDir::new().unwrap();
    fs::write(home.path().join(".env"), "NOT VALID LINE\n").unwrap();
    cmd(home.path())
        .assert()
        .code(78)
        .stderr(predicate::str::contains("cannot read .env"));
}

#[test]
fn config_show_reports_sources_and_hides_secrets() {
    let home = TempDir::new().unwrap();
    write_user_config(home.path(), "[greeting]\nmessage = \"Hi\"\n").unwrap();
    let token_var = format!("{}_API_TOKEN", prefix());
    cmd(home.path())
        .env(&token_var, "do-not-print-me")
        .env(format!("{}__GREETING__REPEAT", prefix()), "3")
        .args(["--log-level", "warn", "config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("message = \"Hi\"  # file "))
        .stdout(predicate::str::contains(format!(
            "repeat = 3  # env {}__GREETING__REPEAT",
            prefix()
        )))
        .stdout(predicate::str::contains(
            "level = \"warn\"  # cli --log-level",
        ))
        .stdout(predicate::str::contains(format!("# {token_var}: set")))
        .stdout(predicate::str::contains("do-not-print-me").not());
}

#[test]
fn config_paths_lists_search_order() {
    let home = TempDir::new().unwrap();
    write_user_config(home.path(), "").unwrap();
    cmd(home.path())
        .args(["config", "paths"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("/etc/{NAME}/config.toml")))
        .stdout(predicate::str::contains("config.toml  <- selected"));
}

#[test]
fn debug_logging_goes_to_stderr() {
    let home = TempDir::new().unwrap();
    cmd(home.path())
        .args(["--log-level", "debug"])
        .assert()
        .success()
        .stdout("Hello, world!\n")
        .stderr(predicate::str::contains("greeting"));
}
