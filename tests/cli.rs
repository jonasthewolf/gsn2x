#[cfg(test)]
mod integrations {
    use anyhow::{Context, Result};
    use assert_cmd::prelude::*;
    use assert_fs::fixture::PathCopy;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use regex::Regex;
    use std::process::Command;

    fn compare_lines_with_replace(
        left: &std::ffi::OsStr,
        right: &std::ffi::OsStr,
        replace_regex: Option<Vec<(Regex, &str)>>,
    ) -> Result<bool> {
        let left: &std::path::Path = left.as_ref();
        let right: &std::path::Path = right.as_ref();
        let left_c =
            std::fs::read_to_string(left).with_context(|| format!("Filename {:?}", left))?;
        let right_c =
            std::fs::read_to_string(right).with_context(|| format!("Filename {:?}", right))?;
        let mut same = true;

        if dbg!(left_c.chars().filter(|&c| c == '\n').count())
            == dbg!(right_c.chars().filter(|&c| c == '\n').count())
        {
            for (l, r) in left_c.lines().zip(right_c.lines()) {
                let l_r = replace_regex
                    .iter()
                    .flatten()
                    .fold(l.to_owned(), |replaced, (r, rp)| {
                        r.replace_all(&replaced, *rp).to_string()
                    });
                let r_r = replace_regex
                    .iter()
                    .flatten()
                    .fold(r.to_owned(), |replaced, (r, rp)| {
                        r.replace_all(&replaced, *rp).to_string()
                    });
                if l_r != r_r {
                    dbg!(&l);
                    dbg!(&l_r);
                    dbg!(&r_r);
                    dbg!(&r);
                    same = false;
                    break;
                }
            }
        } else {
            same = false;
        }

        Ok(same)
    }

    fn are_struct_similar_svgs(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> Result<bool> {
        // Order is important.
        let replaces = vec![
            (
                Regex::new(r#" gsn_module_\w+"#).unwrap(),
                " gsn_module_replaced",
            ),
            (
                Regex::new(r#" (?P<attr>(([rc]?(x|y))|width|height|textLength|viewbox|viewBox))="[\d\s]+""#)
                    .unwrap(),
                " $attr=\"\"",
            ),
            (
                Regex::new(r#" font-family="([0-9A-Za-z-_]|\\.|\\u[0-9a-fA-F]{1,4})+""#).unwrap(),
                " font-family=\"\"",
            ),
            (Regex::new(r#"(-?\d+,-?\d+[, ]?)+"#).unwrap(), ""),
            (
                Regex::new(r#"d="((?P<cmd>[A-Za-z]+)(:?-?\d+(:?,-?\d+)?)? ?(?P<cmd2>z?))+""#)
                    .unwrap(),
                "d=\"$cmd$cmd2\"",
            ),
        ];

        compare_lines_with_replace(left, right, Some(replaces))
    }

    #[test]
    fn file_does_not_exist() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("test/file/does/not/exist");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error: Failed to open file"));
        Ok(())
    }

    #[test]
    fn argument_view() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let temp = assert_fs::TempDir::new()?;
        let sub = temp.child("examples");
        sub.create_dir_all()?;
        sub.copy_from("examples", &["example.gsn.yaml"])?;
        let output_file = sub.child("example.gsn.svg");
        cmd.current_dir(&temp);
        cmd.arg("examples/example.gsn.yaml").arg("-G");
        cmd.assert().success();
        assert!(are_struct_similar_svgs(
            temp.child("examples/example.gsn.svg").as_os_str(),
            output_file.as_os_str()
        )?);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn validate_multiple_only() -> Result<()> {
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
    fn validate_multiple_only_error() -> Result<()> {
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
    fn validate_multiple_only_error_exclude() -> Result<()> {
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
    fn validate_template_instance() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("examples/template/instance.gsn.yaml");
        cmd.assert().success();
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance1() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval1_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 2 errors and 1 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance2() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval2_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 1 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance3() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval3_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 1 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance4() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval4_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 1 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn no_evidences() -> Result<()> {
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
        assert!(compare_lines_with_replace(
            evidence_file.as_os_str(),
            std::path::Path::new("tests/no_evidences.gsn.test.md").as_os_str(),
            None
        )?);
        evidence_file.close()?;
        Ok(())
    }

    #[test]
    fn some_evidences() -> Result<()> {
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
        assert!(compare_lines_with_replace(
            evidence_file.as_os_str(),
            std::path::Path::new("tests/example.gsn.test.md").as_os_str(),
            None
        )?);
        evidence_file.close()?;
        Ok(())
    }

    #[test]
    fn arch_view() -> Result<()> {
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
        temp.close()?;
        Ok(())
    }

    #[test]
    fn multiple_view() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        let temp = assert_fs::TempDir::new()?.into_persistent();
        temp.copy_from("examples/modular", &["*.yaml"])?;
        let input_file1 = temp.child("main.gsn.yaml");
        let input_file2 = temp.child("sub1.gsn.yaml");
        let input_file3 = temp.child("sub3.gsn.yaml");
        let output_file1 = temp.child("main.gsn.svg");
        let output_file2 = temp.child("sub1.gsn.svg");
        let output_file3 = temp.child("sub3.gsn.svg");
        cmd.arg(input_file1.as_os_str())
            .arg(input_file2.as_os_str())
            .arg(input_file3.as_os_str())
            .arg("-A")
            .arg("-E")
            .arg("-F")
            .arg("-G")
            .arg("-s")
            .arg("modular.css");
        cmd.assert().success();
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/main.gsn.svg").as_os_str(),
            output_file1.as_os_str(),
        )?);
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/sub1.gsn.svg").as_os_str(),
            output_file2.as_os_str(),
        )?);
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/sub3.gsn.svg").as_os_str(),
            output_file3.as_os_str(),
        )?);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn complete_view() -> Result<()> {
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
        temp.close()?;
        Ok(())
    }

    #[test]
    fn empty_input() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("tests/empty.yaml");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error: No input elements found"));
        Ok(())
    }

    #[test]
    fn invalid_input1() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("tests/invalid1.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: Failed to parse YAML from file",
        ));
        Ok(())
    }

    #[test]
    fn invalid_input2() -> Result<()> {
        let mut cmd = Command::cargo_bin("gsn2x")?;
        cmd.arg("tests/invalid2.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: Failed to parse YAML from file",
        ));
        Ok(())
    }
}
