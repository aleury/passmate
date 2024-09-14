use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

const CONFIG_HOME: &str = "XDG_CONFIG_HOME";

#[test]
fn binary_with_version_flag_prints_the_version() {
    Command::cargo_bin("passmate")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn binary_with_help_flag_prints_description() {
    Command::cargo_bin("passmate")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_DESCRIPTION")));
}

#[test]
fn binary_with_help_flag_prints_usage() {
    Command::cargo_bin("passmate")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage: passmate"));
}

#[test]
fn binary_with_init_command_creates_a_config_directory_for_the_applicaton() {
    let temp_config =
        TempDir::with_prefix("config-").expect("failed to create temporary config directory");

    let vault_path = temp_config.path().join("passmate/default.vault");

    Command::cargo_bin("passmate")
        .unwrap()
        .env(CONFIG_HOME, temp_config.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains(format!(
            "Initialized vault at {}",
            vault_path.display(),
        )));

    assert!(vault_path.exists(), "expected vault to exist");
}

#[test]
fn binary_with_set_command_adds_a_password_to_the_vault() {
    let temp_config =
        TempDir::with_prefix("config-").expect("failed to create temporary config directory");
    Command::cargo_bin("passmate")
        .unwrap()
        .env(CONFIG_HOME, temp_config.path())
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("passmate")
        .unwrap()
        .env(CONFIG_HOME, temp_config.path())
        .args(["set", "mypass", "test"])
        .assert()
        .success();

    let want = HashMap::from([("mypass".into(), "test".into())]);
    let got = read_vault_data_from_file(&temp_config.path().join("passmate/default.vault"));
    assert_eq!(want, got);
}

fn read_vault_data_from_file(path: &PathBuf) -> HashMap<String, String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}
