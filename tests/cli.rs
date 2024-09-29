use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn changeset_add() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("changeset")?;

    cmd.arg("add")
        .arg("-t")
        .arg("major")
        .arg("-m")
        .arg("'message'");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Changeset created at: "));

    Ok(())
}
