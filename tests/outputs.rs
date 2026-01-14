use anyhow::Result;
use assert_cmd::Command;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

mod basics;
use basics::assert_files_equal;

#[test]
fn no_evidence() -> Result<()> {
    let mut cmd = Command::new(cargo::cargo_bin!());
    let temp = assert_fs::TempDir::new()?;
    temp.copy_from("tests", &["no_evidence.gsn.test.*"])?;
    cmd.arg("-N")
        .arg("-e=my_evidence.md")
        .arg("no_evidence.gsn.test.yaml")
        .current_dir(&temp);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(
            "Writing evidence \"..my_evidence.md\": No evidence found.",
        )?)
        .stderr(predicate::str::is_empty());

    assert_files_equal(
        &temp.child("no_evidence.gsn.test.md"),
        temp.child("my_evidence.md").path(),
    )?;

    temp.close()?;
    Ok(())
}

#[test]
fn some_evidence() -> Result<()> {
    let mut cmd = Command::new(cargo::cargo_bin!());
    let temp = assert_fs::TempDir::new()?;
    temp.copy_from("tests", &["example.gsn.test.md"])?;
    temp.copy_from(".", &["examples/example.gsn.yaml"])?;

    cmd.arg("-e=my_evidence.md")
        .arg("examples/example.gsn.yaml")
        .arg("-l=layer1")
        .arg("-N")
        .current_dir(&temp);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(
            "Writing evidence \"..my_evidence.md\": OK",
        )?)
        .stderr(predicate::str::is_empty());

    assert_files_equal(
        &temp.child("example.gsn.test.md"),
        temp.child("my_evidence.md").path(),
    )?;
    temp.close()?;
    Ok(())
}

const STATISTICS_OUTPUT: &str = r#"Statistics
==========
Number of modules:   3
Number of nodes:     8
  Goals:             3
  Strategies:        1
  Solutions:         1
  Assumptions:       1
  Justifications:    1
  Contexts:          1
  Counter Goals:     0
  Counter Solutions: 0
  Defeated Elements: 0
"#;

#[test]
fn statistics() -> Result<()> {
    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-c")
        .arg("examples/modular/index.gsn.yaml")
        .arg("--statistics");
    cmd.assert()
        .success()
        .stdout(predicate::eq(STATISTICS_OUTPUT));
    Ok(())
}

#[test]
fn statistics_file() -> Result<()> {
    let mut cmd = Command::new(cargo::cargo_bin!());
    let temp = assert_fs::TempDir::new()?;
    temp.copy_from(".", &["examples/modular/*.yaml"])?;
    cmd.arg("-c")
        .arg("examples/modular/index.gsn.yaml")
        .arg("--statistics=statistics.md")
        .current_dir(&temp);
    cmd.assert().success();

    temp.child("statistics.md")
        .assert(predicate::eq(STATISTICS_OUTPUT));

    Ok(())
}

#[test]
fn yaml_dump() -> Result<()> {
    let mut cmd = Command::new(cargo::cargo_bin!());
    cmd.arg("-c")
        .arg("examples/modular/index.gsn.yaml")
        .arg("--dump-yaml");
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty().not().from_utf8());
    let output = cmd.output()?.stdout;

    let mut cmd = Command::new(cargo::cargo_bin!());
    let temp = assert_fs::TempDir::new()?;
    temp.copy_from(".", &["examples/modular/*.yaml"])?;
    cmd.arg("-c")
        .arg("examples/modular/index.gsn.yaml")
        .arg("--dump-yaml=out.yaml")
        .current_dir(&temp);
    cmd.assert().success();

    temp.child("out.yaml")
        .assert(predicate::eq(output).from_file_path());

    Ok(())
}
