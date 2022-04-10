#[cfg(test)]
mod integrations {
    use assert_cmd::prelude::*;
    use assert_fs::fixture::PathCopy;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use regex::Regex;
    use std::process::Command;

    fn are_struct_similar_svgs(
        left: &std::ffi::OsStr,
        right: &std::ffi::OsStr,
    ) -> Result<bool, std::io::Error> {
        let left: &std::path::Path = left.as_ref();
        let right: &std::path::Path = right.as_ref();
        let left_c = std::fs::read_to_string(left)?;
        let right_c = std::fs::read_to_string(right)?;
        let mut same = true;
        let coords = Regex::new(r#" (([rc]?(x|y))|width|height|textLength)="\d+""#).unwrap();
        let paths = Regex::new(r#"(-?\d+,-?\d+[, ]?)+"#).unwrap();

        if left_c.chars().filter(|&c| c == '\n').count()
            == right_c.chars().filter(|&c| c == '\n').count()
        {
            for (l, r) in left_c.lines().zip(right_c.lines()) {
                if paths.replace_all(&coords.replace_all(l, ""), "")
                    != paths.replace_all(&coords.replace_all(r, ""), "")
                {
                    same = false;
                    break;
                }
            }
        } else {
            same = false;
        }

        Ok(same)
    }

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
    fn argument_view() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let temp = assert_fs::TempDir::new()?;
        temp.copy_from("examples", &["example.gsn.yaml"])?;
        let input_file = temp.child("example.gsn.yaml");
        let output_file = temp.child("example.gsn.svg");
        cmd.arg(input_file.as_os_str()).arg("-G");
        cmd.assert().success();
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/example.gsn.svg").as_os_str(),
            output_file.as_os_str()
        )?);
        output_file.assert(predicate::path::eq_file(std::path::Path::new(
            "examples/example.gsn.svg",
        )));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn validate_multiple_only() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/modular/main.gsn.yaml")
            .arg("examples/modular/sub1.gsn.yaml")
            .arg("examples/modular/sub3.gsn.yaml");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
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
        cmd.arg("-N")
            .arg("-e")
            .arg(evidence_file.path())
            .arg("tests/no_evidences.gsn.test.yaml");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
        let predicate_file = predicate::path::eq_file(evidence_file.path())
            .utf8()
            .unwrap();
        assert!(predicate_file.eval(std::path::Path::new("tests/no_evidences.gsn.test.md")));
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
            .arg("layer1")
            .arg("-N");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
        let predicate_file = predicate::path::eq_file(evidence_file.path())
            .utf8()
            .unwrap();
        assert!(predicate_file.eval(std::path::Path::new("tests/example.gsn.test.md")));
        evidence_file.close()?;
        Ok(())
    }

    #[test]
    fn arch_view() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let temp = assert_fs::TempDir::new()?.into_persistent();
        temp.copy_from("examples/modular", &["*.yaml"])?;
        let input_file1 = temp.child("main.gsn.yaml");
        let input_file2 = temp.child("sub1.gsn.yaml");
        let input_file3 = temp.child("sub3.gsn.yaml");
        let output_file = temp.child("architecture.svg");
        cmd.arg(input_file1.as_os_str())
            .arg(input_file2.as_os_str())
            .arg(input_file3.as_os_str())
            .arg("-N")
            .arg("-E")
            .arg("-F")
            .arg("-G");
        cmd.assert().success();
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/architecture.svg").as_os_str(),
            output_file.as_os_str(),
        )?);
        output_file.assert(predicate::path::eq_file(std::path::Path::new(
            "examples/modular/architecture.svg",
        )));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn comp_view() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let temp = assert_fs::TempDir::new()?.into_persistent();
        temp.copy_from("examples/modular", &["*.yaml"])?;
        let input_file1 = temp.child("main.gsn.yaml");
        let input_file2 = temp.child("sub1.gsn.yaml");
        let input_file3 = temp.child("sub3.gsn.yaml");
        let output_file = temp.child("complete.svg");
        cmd.arg(input_file1.as_os_str())
            .arg(input_file2.as_os_str())
            .arg(input_file3.as_os_str())
            .arg("-N")
            .arg("-E")
            .arg("-A")
            .arg("-G");
        cmd.assert().success();
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/complete.svg").as_os_str(),
            output_file.as_os_str(),
        )?);
        output_file.assert(predicate::path::eq_file(std::path::Path::new(
            "examples/modular/complete.svg",
        )));
        temp.close()?;
        Ok(())
    }
}
