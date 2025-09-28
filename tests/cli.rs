mod basics;
mod reg_render;

use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::fixture::PathCopy;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::path::Path;
use std::process::Command;

use crate::basics::*;
use crate::reg_render::regression_renderings;

#[test]
fn file_does_not_exist() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("test/file/does/not/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Failed to open file"));
    Ok(())
}

#[test]
fn index_gsn_does_not_exist() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new()?;
    cmd.current_dir(&temp);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: index.gsn.yaml not found."));
    Ok(())
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
    let expected = temp.child("out1.svg");
    let reference = temp.child("out2.svg");
    // Run program twice with full legend
    cmd.current_dir(&temp);
    cmd.arg(INPUT_YAML);
    cmd.assert().success();
    std::fs::rename(&output_file, &expected)?;
    cmd.assert().success();
    std::fs::rename(&output_file, &reference)?;

    let expected_contents = std::fs::read_to_string(expected)?;
    let reference_contents = std::fs::read_to_string(reference)?;
    let exp_line_count = expected_contents.chars().filter(|&c| c == '\n').count();
    let ref_line_count = reference_contents.chars().filter(|&c| c == '\n').count();

    assert_eq!(exp_line_count, ref_line_count);

    assert!(
        expected_contents
            .lines()
            .zip(reference_contents.lines())
            .any(|(l, r)| l != r)
    );

    temp.close()?;
    Ok(())
}

#[test]
fn argument_view() -> Result<()> {
    regression_renderings(&["examples/example.gsn.yaml"], &[], None)?;
    Ok(())
}

#[test]
fn multi_contexts() -> Result<()> {
    regression_renderings(&["tests/multi_context.gsn.yaml"], &[], None)?;
    Ok(())
}

#[test]
fn entangled() -> Result<()> {
    const SUB_DIR: &str = "examples";
    const INPUT_YAML: &str = "entangled.gsn.yaml";
    const OUTPUT_SVG: &str = "entangled.gsn.svg";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new()?;

    // Copy input into subdirectory and name output file
    let sub = temp.child(SUB_DIR);
    sub.create_dir_all()?;
    sub.copy_from(SUB_DIR, &[INPUT_YAML])?;
    let output_file = sub.child(OUTPUT_SVG);

    // Copy expected output into temp dir (without subdirectory!)
    temp.copy_from(SUB_DIR, &[OUTPUT_SVG])?;

    // Start cargo from temporary directory
    cmd.current_dir(&temp);
    cmd.arg(format!("{SUB_DIR}/{INPUT_YAML}")).arg("-G");
    cmd.assert().success().stdout(predicate::str::contains(
        "Diagram took too many iterations (6).",
    ));
    assert_files_equal(&output_file, temp.child(OUTPUT_SVG).path())?;
    temp.close()?;
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
        .arg("-x=examples/modular/sub2.gsn.yaml");
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
fn arch_view() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new()?;
    temp.copy_from(".", &["examples/modular/*.yaml"])?;
    let output_file = temp.child("examples/modular/architecture.svg");
    cmd.arg("examples/modular/index.gsn.yaml")
        .arg("-o=examples/modular")
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
    assert_files_equal(&output_file, Path::new("examples/modular/architecture.svg"))?;
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
            .arg("-s=https://github.com/jonasthewolf/gsn2x/blob/3439402d093ba54af4771b295e78f2488bd1b978/examples/modular/modular.css")
            .current_dir(&temp);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(concat!(
            "Rendering \"..examples.modular.index.gsn.svg\": OK\n",
            "Rendering \"..examples.modular.sub1.gsn.svg\": OK\n",
            "Rendering \"..examples.modular.sub3.gsn.svg\": OK\n",
        ))?);
    assert_files_equal(&output_file1, Path::new("examples/modular/index.gsn.svg"))?;
    assert_files_equal(&output_file2, Path::new("examples/modular/sub1.gsn.svg"))?;
    assert_files_equal(&output_file3, Path::new("examples/modular/sub3.gsn.svg"))?;
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
    assert_files_equal(&output_file, Path::new("examples/modular/complete.svg"))?;
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
fn dialectic_first() -> Result<()> {
    regression_renderings(&["examples/dialectic/first.gsn.yaml"], &[], None)?;
    Ok(())
}

#[test]
fn dialectic_second() -> Result<()> {
    regression_renderings(&["examples/dialectic/second.gsn.yaml"], &[], None)?;
    Ok(())
}

#[test]
fn min_doc() -> Result<()> {
    regression_renderings(
        &["examples/minimalcss/min.gsn.yaml"],
        &["-s=examples/minimalcss/min.css", "-t"],
        Some(&["examples/minimalcss/min.css"]),
    )?;
    Ok(())
}

#[test]
fn min_doc_copy_css() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new()?;
    temp.copy_from("examples/minimalcss", &["min*"])?;
    cmd.current_dir(&temp)
        .arg("-s=min.css")
        .arg("-o=output")
        .arg("min.gsn.yaml");
    cmd.assert().success();
    temp.child("output")
        .child("min.css")
        .assert(predicate::path::exists());
    Ok(())
}

#[test]
fn additionals() -> Result<()> {
    regression_renderings(
        &["tests/additionals.yaml"],
        &["-E", "-l=add1", "-l=unsupported", "-l=additional"],
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
fn font_metrics() -> Result<()> {
    regression_renderings(&["tests/font_metrics.yaml"], &["-E"], None)?;
    Ok(())
}
