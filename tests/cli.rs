use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn e2e_changeset_add() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = tempdir()?;

    let mut cmd = Command::cargo_bin("changeset")?;

    cmd.current_dir(&tmp_dir)
        .arg("add")
        .arg("-t")
        .arg("major")
        .arg("-m")
        .arg("'message'");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Changeset created at: "));

    Ok(())
}
