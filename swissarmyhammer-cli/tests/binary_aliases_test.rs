use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_swissarmyhammer_binary_exists() {
    Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("swissarmyhammer"));
}

#[test]
fn test_sah_binary_exists() {
    Command::cargo_bin("sah")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("swissarmyhammer"));
}

#[test]
fn test_both_binaries_have_same_commands() {
    // Both binaries should have the same available commands
    Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("serve"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("prompt"))
        .stdout(predicate::str::contains("flow"))
        .stdout(predicate::str::contains("completion"))
        .stdout(predicate::str::contains("validate"));

    Command::cargo_bin("sah")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("serve"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("prompt"))
        .stdout(predicate::str::contains("flow"))
        .stdout(predicate::str::contains("completion"))
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_both_binaries_same_version() {
    let swissarmyhammer_output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .arg("--version")
        .output()
        .expect("Failed to execute swissarmyhammer");

    let sah_output = Command::cargo_bin("sah")
        .unwrap()
        .arg("--version")
        .output()
        .expect("Failed to execute sah");

    // Both should report the same version
    assert_eq!(
        String::from_utf8_lossy(&swissarmyhammer_output.stdout),
        String::from_utf8_lossy(&sah_output.stdout),
        "Both binaries should report the same version"
    );
}
