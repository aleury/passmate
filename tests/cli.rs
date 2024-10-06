use assert_cmd::Command;
use enigo::{Direction, Enigo, InputResult, Key, Keyboard, Settings};
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
fn binary_with_set_command_adds_a_password_to_the_vault() {
    let handle = std::thread::spawn(|| {
        let temp_config =
            TempDir::with_prefix("config-").expect("failed to create temporary config directory");
        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["set", "mypass", "test"])
            .assert()
    });

    enter_password().expect("failed to enter password");

    handle.join().unwrap().success();
}

#[test]
fn binary_with_get_command_retrieves_password_from_the_vault() {
    let handle = std::thread::spawn(|| {
        let temp_config =
            TempDir::with_prefix("config-").expect("failed to create temporary config directory");
        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["set", "mypass", "testpass"])
            .assert();

        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["get", "mypass"])
            .assert()
    });

    enter_password().expect("failed to enter password");
    enter_password().expect("failed to enter password");

    handle
        .join()
        .unwrap()
        .success()
        .stdout(predicates::str::contains("testpass"));
}

#[test]
fn binary_with_get_command_prints_not_found_if_a_password_with_given_name_does_not_exist() {
    let handle = std::thread::spawn(|| {
        let temp_config =
            TempDir::with_prefix("config-").expect("failed to create temporary config directory");

        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["get", "mypass"])
            .assert()
    });
    enter_password().expect("failed to enter password");
    handle
        .join()
        .unwrap()
        .failure()
        .stderr(predicates::str::contains("mypass not found"));
}

#[test]
fn binary_with_remove_command_deletes_a_password_from_the_vault() {
    let handle = std::thread::spawn(|| {
        let temp_config =
            TempDir::with_prefix("config-").expect("failed to create temporary config directory");
        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["set", "mypass", "testpass"])
            .assert()
            .success();

        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["remove", "mypass"])
            .assert()
            .success();

        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["get", "mypass"])
            .assert()
    });

    enter_password().expect("failed to enter password");
    enter_password().expect("failed to enter password");
    enter_password().expect("failed to enter password");

    handle
        .join()
        .unwrap()
        .failure()
        .stderr(predicates::str::contains("mypass not found"));
}

#[test]
fn binary_with_ls_command_lists_the_entry_names_in_a_vault() {
    let handle = std::thread::spawn(|| {
        let temp_config =
            TempDir::with_prefix("config-").expect("failed to create temporary config directory");
        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["set", "pass1", "secretpass1"])
            .assert()
            .success();
        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .args(["set", "pass2", "secretpass2"])
            .assert()
            .success();
        Command::cargo_bin("passmate")
            .unwrap()
            .env(CONFIG_HOME, temp_config.path())
            .arg("ls")
            .assert()
    });

    enter_password().expect("failed to enter password");
    enter_password().expect("failed to enter password");
    enter_password().expect("failed to enter password");

    handle
        .join()
        .unwrap()
        .success()
        .stdout(predicates::str::contains("pass1\npass2\n"));
}

fn enter_password() -> InputResult<()> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.text("testpwd")?;
    enigo.key(Key::Return, Direction::Click)?;
    Ok(())
}
