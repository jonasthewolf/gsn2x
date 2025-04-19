#[cfg(test)]
mod integrations {
    use anyhow::{Context, Result};
    use assert_cmd::assert::Assert;
    use assert_cmd::prelude::*;
    use assert_fs::fixture::PathCopy;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use regex::Regex;
    use std::process::Command;

    fn split_keep<'a>(regex: &Regex, input: &'a str) -> Vec<&'a str> {
        let mut result = vec![];
        let mut last = 0;
        for rmatch in regex.find_iter(input) {
            if last != rmatch.start() {
                result.push(&input[last..rmatch.start()]);
            }
            result.push(rmatch.as_str());
            last = rmatch.end();
        }
        if last < input.len() {
            result.push(&input[last..]);
        }
        result
    }

    fn check_within_tolerance(l: i64, r: i64) -> bool {
        if l == r {
            true
        } else {
            const TOLERANCE: f64 = 0.1;
            let m = (l + r) as f64 / 2.0;
            let min = std::cmp::min(
                (m * (1.0 - TOLERANCE)) as i64,
                (m * (1.0 + TOLERANCE)) as i64,
            );
            let max = std::cmp::max(
                (m * (1.0 - TOLERANCE)) as i64,
                (m * (1.0 + TOLERANCE)) as i64,
            );
            min <= l && max >= l && min <= r && max >= r
        }
    }

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

        let num_regex = Regex::new(r"-?\d+").unwrap(); // unwrap ok, since static regex

        let l_line_count = left_c.chars().filter(|&c| c == '\n').count();
        let r_line_count = right_c.chars().filter(|&c| c == '\n').count();
        println!("Lines: {l_line_count} {r_line_count}");
        if l_line_count == r_line_count {
            for (l, r) in left_c.lines().zip(right_c.lines()) {
                let l_r = replace_regex
                    .iter()
                    .flatten()
                    .fold(l.to_owned(), |replaced, (r, rp)| {
                        r.replace_all(&replaced, *rp).to_string()
                    });
                let l_split = split_keep(&num_regex, &l_r);
                let r_r = replace_regex
                    .iter()
                    .flatten()
                    .fold(r.to_owned(), |replaced, (r, rp)| {
                        r.replace_all(&replaced, *rp).to_string()
                    });
                let r_split = split_keep(&num_regex, &r_r);
                if l_split.len() == r_split.len() {
                    for (l_m, r_m) in l_split.into_iter().zip(r_split) {
                        let l_num = l_m.parse::<i64>();
                        let r_num = r_m.parse::<i64>();
                        if !match (l_num, r_num) {
                            (Ok(l), Ok(r)) => check_within_tolerance(l, r),
                            (Ok(_), Err(_)) => false,
                            (Err(_), Ok(_)) => false,
                            (Err(l), Err(r)) => l == r,
                        } {
                            println!("Match: {} {}", &l_m, &r_m);
                            same = false;
                            break;
                        }
                    }
                } else {
                    println!("Splitted Line: {:?} {:?}", l_split, r_split);
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
                Regex::new(r" gsn_module_\w+").unwrap(),
                " gsn_module_replaced",
            ),
            // (
            //     Regex::new(r#" (?P<attr>(([rc]?(x|y))|width|height|textLength|viewbox|viewBox))="[\d\s]+""#)
            //         .unwrap(),
            //     " $attr=\"\"",
            // ),
            (
                Regex::new(r#" font-family="([0-9A-Za-z-_]|\\.|\\u[0-9a-fA-F]{1,4})+""#).unwrap(),
                " font-family=\"\"",
            ),
            // (Regex::new(r"(-?\d+,-?\d+[, ]?)+").unwrap(), ""),
            // (
            //     Regex::new(r#"d="((?P<cmd>[A-Za-z]+)(:?-?\d+(:?,-?\d+)?)? ?(?P<cmd2>z?))+""#)
            //         .unwrap(),
            //     "d=\"$cmd$cmd2\"",
            // ),
        ];

        compare_lines_with_replace(left, right, Some(replaces))
    }

    #[test]
    fn file_does_not_exist() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("test/file/does/not/exist");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error: Failed to open file"));
        Ok(())
    }

    ///
    /// This tricky setup is needed since the relative path is used for the module name.
    /// The module name is used as SVG class.
    /// The reference output is locally generated with the same setup.
    ///
    fn check_if_outputs_are_similar(
        sub_dir: &str,
        input: &str,
        expected_output: &str,
    ) -> Result<Assert> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;

        // Copy input into subdirectory and name output file
        let sub = temp.child(sub_dir);
        sub.create_dir_all()?;
        sub.copy_from(sub_dir, &[input])?;
        let output_file = sub.child(expected_output);

        // Copy expected output into temp dir (without subdirectory!)
        temp.copy_from(sub_dir, &[expected_output])?;

        // Start cargo from temporary directory
        cmd.current_dir(&temp);
        cmd.arg(format!("{sub_dir}/{input}")).arg("-G");
        let result = cmd.assert().success();
        assert!(are_struct_similar_svgs(
            output_file.as_os_str(),
            temp.child(expected_output).as_os_str(),
        )?);
        temp.close()?;
        Ok(result)
    }

    #[test]
    fn legend_is_different() -> Result<()> {
        const SUB_DIR: &str = "examples";
        const INPUT_YAML: &str = "example.gsn.yaml";
        const OUTPUT_SVG: &str = "example.gsn.svg";

        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;

        temp.copy_from(SUB_DIR, &[INPUT_YAML])?;
        let output_file = temp.child(OUTPUT_SVG);
        let output_file1 = temp.child("out1.svg");
        let output_file2 = temp.child("out2.svg");
        // Run program twice with full legend
        cmd.current_dir(&temp);
        cmd.arg(INPUT_YAML);
        cmd.assert().success();
        std::fs::rename(&output_file, &output_file1)?;
        cmd.assert().success();
        std::fs::rename(&output_file, &output_file2)?;

        output_file1.assert(predicate::path::eq_file(output_file2.path()).not());

        temp.close()?;
        Ok(())
    }

    #[test]
    fn argument_view() -> Result<()> {
        const SUB_DIR: &str = "examples";
        const INPUT_YAML: &str = "example.gsn.yaml";
        const OUTPUT_SVG: &str = "example.gsn.svg";
        let _ = check_if_outputs_are_similar(SUB_DIR, INPUT_YAML, OUTPUT_SVG)?;
        Ok(())
    }

    #[test]
    fn multi_contexts() -> Result<()> {
        const SUB_DIR: &str = "tests";
        const INPUT_YAML: &str = "multi_context.gsn.yaml";
        const OUTPUT_SVG: &str = "multi_context.gsn.svg";

        let _ = check_if_outputs_are_similar(SUB_DIR, INPUT_YAML, OUTPUT_SVG)?;
        Ok(())
    }

    #[test]
    fn entangled() -> Result<()> {
        const SUB_DIR: &str = "examples";
        const INPUT_YAML: &str = "entangled.gsn.yaml";
        const OUTPUT_SVG: &str = "entangled.gsn.svg";

        let res = check_if_outputs_are_similar(SUB_DIR, INPUT_YAML, OUTPUT_SVG)?;
        res.stdout(predicate::str::contains(
            "Diagram took too many iterations (6).",
        ));
        Ok(())
    }

    #[test]
    fn validate_multiple_only() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c").arg("examples/modular/index.gsn.yaml");
        cmd.assert()
            .success()
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::is_empty());
        Ok(())
    }

    #[test]
    fn validate_multiple_only_error() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("examples/modular/index.gsn.yaml")
            .arg("examples/modular/sub2.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 2 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_multiple_only_error_exclude() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("examples/modular/index.gsn.yaml")
            .arg("examples/modular/sub2.gsn.yaml")
            .arg("-x")
            .arg("examples/modular/sub2.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 2 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_template_instance() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("examples/template/instance.gsn.yaml");
        cmd.assert().success();
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance1() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval1_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 2 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance2() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval2_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance3() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval3_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn validate_template_invalid_instance4() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("examples/template/template.gsn.yaml")
            .arg("tests/inval4_instance.gsn.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: 1 errors and 0 warnings detected.",
        ));
        Ok(())
    }

    #[test]
    fn no_evidence() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;
        temp.copy_from("tests", &["no_evidence.gsn.test.*"])?;
        cmd.arg("-N")
            .arg("-e")
            .arg("my_evidence.md")
            .arg("no_evidence.gsn.test.yaml")
            .current_dir(&temp);
        cmd.assert()
            .success()
            .stdout(predicate::str::is_match(
                "Writing evidence \"..my_evidence.md\": No evidence found.",
            )?)
            .stderr(predicate::str::is_empty());
        assert!(compare_lines_with_replace(
            temp.child("my_evidence.md").as_os_str(),
            temp.child("no_evidence.gsn.test.md").as_os_str(),
            None
        )?);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn some_evidence() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;
        temp.copy_from("tests", &["example.gsn.test.md"])?;
        temp.copy_from(".", &["examples/example.gsn.yaml"])?;

        cmd.arg("-e")
            .arg("my_evidence.md")
            .arg("examples/example.gsn.yaml")
            .arg("-l")
            .arg("layer1")
            .arg("-N")
            .current_dir(&temp);
        cmd.assert()
            .success()
            .stdout(predicate::str::is_match(
                "Writing evidence \"..my_evidence.md\": OK",
            )?)
            .stderr(predicate::str::is_empty());
        assert!(compare_lines_with_replace(
            temp.child("my_evidence.md").as_os_str(),
            temp.child("example.gsn.test.md").as_os_str(),
            None
        )?);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn arch_view() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;
        temp.copy_from(".", &["examples/modular/*.yaml"])?;
        let output_file = temp.child("examples/modular/architecture.svg");
        cmd.arg("examples/modular/index.gsn.yaml")
            .arg("-o")
            .arg("examples/modular")
            .arg("-E")
            .arg("-F")
            .arg("-G")
            .current_dir(&temp);
        cmd.assert()
            .success()
            .stdout(predicate::str::is_match(concat!(
                "Rendering \"examples.modular.index.gsn.svg\": OK\n",
                "Rendering \"examples.modular.sub1.gsn.svg\": OK\n",
                "Rendering \"examples.modular.sub3.gsn.svg\": OK\n",
                "Rendering \"examples.modular.architecture.svg\": OK\n",
            ))?);
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/architecture.svg").as_os_str(),
            output_file.as_os_str(),
        )?);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn multiple_view() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;
        temp.copy_from(".", &["examples/modular/*.yaml"])?;
        let output_file1 = temp.child("examples/modular/index.gsn.svg");
        let output_file2 = temp.child("examples/modular/sub1.gsn.svg");
        let output_file3 = temp.child("examples/modular/sub3.gsn.svg");
        cmd.arg("examples/modular/index.gsn.yaml")
            .arg("-A")
            .arg("-E")
            .arg("-F")
            .arg("-G")
            .arg("-s")
            .arg("https://github.com/jonasthewolf/gsn2x/blob/3439402d093ba54af4771b295e78f2488bd1b978/examples/modular/modular.css")
            .current_dir(&temp);
        cmd.assert()
            .success()
            .stdout(predicate::str::is_match(concat!(
                "Rendering \"..examples.modular.index.gsn.svg\": OK\n",
                "Rendering \"..examples.modular.sub1.gsn.svg\": OK\n",
                "Rendering \"..examples.modular.sub3.gsn.svg\": OK\n",
            ))?);
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/index.gsn.svg").as_os_str(),
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
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;
        temp.copy_from(".", &["examples/modular/*.yaml"])?;
        let output_file = temp.child("complete.svg");
        cmd.arg("examples/modular/index.gsn.yaml")
            .arg("-N")
            .arg("-E")
            .arg("-A")
            .arg("-G")
            .current_dir(&temp);
        cmd.assert().success().stdout(predicate::str::is_match(
            "Rendering \"..complete.svg\": OK\n",
        )?);
        assert!(are_struct_similar_svgs(
            std::path::Path::new("examples/modular/complete.svg").as_os_str(),
            output_file.as_os_str(),
        )?);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn empty_input() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("tests/empty.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: No input elements are found.",
        ));
        Ok(())
    }

    #[test]
    fn invalid_input1() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("tests/invalid1.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: Failed to parse YAML from file",
        ));
        Ok(())
    }

    #[test]
    fn invalid_input2() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("tests/invalid2.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: Failed to parse YAML from file",
        ));
        Ok(())
    }

    fn regression_renderings(
        input: &[&str],
        options: &[&str],
        additional_files: Option<&[&str]>,
    ) -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let temp = assert_fs::TempDir::new()?;
        temp.copy_from(".", input)?;
        if let Some(additional_files) = additional_files {
            temp.copy_from(".", additional_files)?;
        }
        let output_names = input
            .iter()
            .map(|i| i.to_string().replace(".yaml", ".svg"))
            .collect::<Vec<_>>();
        cmd.args(options).args(input).arg("-G").current_dir(&temp);
        cmd.assert().success();
        for output_name in output_names {
            let output_file = temp.child(&output_name);
            assert!(are_struct_similar_svgs(
                std::path::Path::new(&output_name).as_os_str(),
                output_file.as_os_str(),
            )?);
        }
        temp.close()?;
        Ok(())
    }

    #[test]
    fn multi_parents() -> Result<()> {
        regression_renderings(&["tests/multi_parents.gsn.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn multi_children() -> Result<()> {
        regression_renderings(&["tests/multi_children.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn multi_children_min() -> Result<()> {
        regression_renderings(&["tests/multi_children_min.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn dialectic() -> Result<()> {
        regression_renderings(&["examples/dialectic/index.gsn.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn min_doc() -> Result<()> {
        regression_renderings(
            &["examples/minimalcss/min.gsn.yaml"],
            &["-s", "examples/minimalcss/min.css", "-t"],
            Some(&["examples/minimalcss/min.css"]),
        )?;
        Ok(())
    }

    #[test]
    fn issue250() -> Result<()> {
        regression_renderings(&["tests/issue250.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn issue249() -> Result<()> {
        regression_renderings(&["tests/issue249.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn issue313() -> Result<()> {
        regression_renderings(&["tests/issue313.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn issue339() -> Result<()> {
        regression_renderings(&["tests/issue339.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn issue84() -> Result<()> {
        regression_renderings(&["tests/issue84_1.yaml"], &[], None)?;
        regression_renderings(&["tests/issue84_2.yaml"], &[], None)?;
        regression_renderings(&["tests/issue84_3.yaml"], &[], None)?;
        regression_renderings(&["tests/issue84_4.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn issue358() -> Result<()> {
        regression_renderings(
            &["tests/issue358.yaml"],
            &["-l", "layer1", "-l", "layer2"],
            None,
        )?;
        Ok(())
    }

    #[test]
    fn issue365() -> Result<()> {
        regression_renderings(&["tests/issue365.yaml"], &["-w", "35"], None)?;
        Ok(())
    }

    #[test]
    fn issue371() -> Result<()> {
        regression_renderings(&["tests/issue371.yaml"], &[], None)?;
        Ok(())
    }

    #[test]
    fn issue372() -> Result<()> {
        regression_renderings(&["tests/issue372.yaml"], &["-w", "35"], None)?;
        Ok(())
    }

    #[test]
    fn issue377() -> Result<()> {
        regression_renderings(&["tests/issue377.yaml"], &["-w", "35"], None)?;
        Ok(())
    }

    #[test]
    fn issue389() -> Result<()> {
        regression_renderings(&["tests/issue389.yaml"], &["-w", "20"], None)?;
        Ok(())
    }

    #[test]
    fn issue391() -> Result<()> {
        regression_renderings(
            &["tests/issue391_1.yaml", "tests/issue391_2.yaml"],
            &[],
            None,
        )?;
        Ok(())
    }

    #[test]
    fn issue433() -> Result<()> {
        regression_renderings(
            &["tests/issue433_1.yaml"],
            &["-w", "20", "-F", "-A"],
            Some(&["tests/issue433_2.yaml"]),
        )?;
        Ok(())
    }

    #[test]
    fn issue393() -> Result<()> {
        regression_renderings(&["tests/issue393_1.yaml"], &["-w", "20"], None)?;
        regression_renderings(&["tests/issue393_2.yaml"], &["-w", "20"], None)?;
        Ok(())
    }

    #[test]
    fn issue396() -> Result<()> {
        regression_renderings(
            &["tests/issue396.yaml"],
            &["-w", "20", "-l", "layer1"],
            None,
        )?;
        Ok(())
    }

    #[test]
    fn issue407() -> Result<()> {
        regression_renderings(
            &["tests/issue407.yaml"],
            &["-w", "20", "-l", "layer2"],
            None,
        )?;
        Ok(())
    }

    #[test]
    fn issue453() -> Result<()> {
        regression_renderings(&["tests/issue453.yaml"], &["-l", "layer2"], None)?;
        Ok(())
    }

    #[test]
    fn additionals() -> Result<()> {
        regression_renderings(
            &["tests/additionals.yaml"],
            &["-E", "-l", "add1", "-l", "unsupported", "-l", "additional"],
            None,
        )?;
        Ok(())
    }

    #[test]
    fn confidence_extension() -> Result<()> {
        regression_renderings(&["examples/confidence.gsn.yaml"], &["-E"], None)?;
        Ok(())
    }

    #[test]
    fn uses_circle_detection() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("tests/circle1.yaml");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error: (A) C06: Module in tests/circle1.yaml was already present in tests/circle1.yaml provided by command line."));
        Ok(())
    }

    #[test]
    fn issue467() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-G").arg("-E").arg("tests/issue467.yaml");
        cmd.assert().success().stderr(predicate::str::contains(
            "Warning: C01: There is more than one unreferenced element: GA, GB.",
        ));
        Ok(())
    }

    #[test]
    fn issue472() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("tests/issue472.yaml");
        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: (Test) The module does not contain elements.",
        ));
        Ok(())
    }

    #[test]
    fn statistics() -> Result<()> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("-c")
            .arg("--statistics")
            .arg("examples/modular/index.gsn.yaml");
        cmd.assert().success().stdout(predicate::str::contains(
            r#"Statistics
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
  Defeated Elements: 0"#,
        ));
        Ok(())
    }
}
