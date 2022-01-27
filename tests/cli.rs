#[cfg(test)]
mod integrations {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::{fs, process::Command};

    #[test]
    fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("test/file/doesnt/exist");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error: Failed to open file"));
        Ok(())
    }

    #[test]
    fn multiple_inputs_stdout() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("file1").arg("file2").arg("-o");
        cmd.assert().failure().stderr(predicate::str::contains(
            "The argument '-o' cannot be used with multiple input files.",
        ));
        Ok(())
    }

    #[test]
    fn argument_view() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("examples/example.gsn.yaml").arg("-o");
        cmd.assert()
            .success()
            .stdout(predicate::path::eq_file(std::path::Path::new(
                "examples/example.gsn.test.dot",
            )));
        Ok(())
    }

    #[test]
    fn validate_multiple_only() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/modular/main.gsn.yaml")
            .arg("examples/modular/sub1.gsn.yaml");
        cmd.assert().success().stdout(predicate::str::is_empty());
        Ok(())
    }

    #[test]
    fn validate_multiple_only_error() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/modular/main.gsn.yaml")
            .arg("examples/modular/sub2.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_multiple_only_error_exclude() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/modular/main.gsn.yaml")
            .arg("examples/modular/sub2.gsn.yaml")
            .arg("-x")
            .arg("examples/modular/sub2.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn no_evidences() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let evidence_file = assert_fs::NamedTempFile::new("evidences.md")?;
        cmd.arg("-n")
            .arg("-e")
            .arg(evidence_file.path())
            .arg("examples/modular/main.gsn.yaml")
            .arg("examples/modular/sub1.gsn.yaml");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
        let predicate_file = predicate::path::eq_file(evidence_file.path())
            .utf8()
            .unwrap();
        assert!(predicate_file.eval("\nList of Evidences\n\nNo evidences found"));
        evidence_file.close()?;
        Ok(())
    }

    #[test]
    fn some_evidences() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let evidence_file = assert_fs::NamedTempFile::new("evidences.md")?;
        cmd.arg("-e")
            .arg(evidence_file.path())
            .arg("examples/example.gsn.yaml")
            .arg("-l")
            .arg("layer1");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
        let predicate_file = predicate::path::eq_file(evidence_file.path())
            .utf8()
            .unwrap();
        assert!(predicate_file.eval(std::path::Path::new("examples/example.gsn.test.md")));
        evidence_file.close()?;
        Ok(())
    }

    #[test]
    fn arch_view() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let arch_file = assert_fs::NamedTempFile::new("evidences.md")?;
        cmd.arg("-n")
            .arg("-a")
            .arg(arch_file.path())
            .arg("examples/modular/main.gsn.yaml")
            .arg("examples/modular/sub1.gsn.yaml");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
        let predicate_file = predicate::path::eq_file(arch_file.path()).utf8().unwrap();
        // Fix path from temporary location
        let expected =
            fs::read_to_string(std::path::Path::new("examples/modular/arch.gsn.test.dot"))?
                .replace(
                    "examples/modular/arch.gsn.test.dot",
                    &format!("{}", arch_file.path().display()),
                );
        assert!(predicate_file.eval(expected.as_str()));
        arch_file.close()?;
        Ok(())
    }

    #[test]
    fn comp_view() -> Result<(), Box<dyn std::error::Error>> {
        // TODO Implement
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let compl_file = assert_fs::NamedTempFile::new("complete.dot")?;
        cmd.arg("-n")
            .arg("-f")
            .arg(compl_file.path())
            .arg("examples/modular/main.gsn.yaml")
            .arg("examples/modular/sub1.gsn.yaml");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
        let predicate_file = predicate::path::eq_file(compl_file.path()).utf8().unwrap();
        // Fix path from temporary location
        let expected = fs::read_to_string(std::path::Path::new(
            "examples/modular/complete.gsn.test.dot",
        ))?
        .replace(
            "examples/modular/complete.gsn.test.dot",
            &format!("{}", compl_file.path().display()),
        );
        assert!(predicate_file.eval(expected.as_str()));
        compl_file.close()?;
        Ok(())
    }
}
