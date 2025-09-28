use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::fixture::PathCopy;
use assert_fs::prelude::*;
use std::path::Path;

use std::process::Command;

pub fn assert_files_equal(expected: &Path, reference: &Path) -> Result<()> {
    let expected_contents = std::fs::read_to_string(expected)?;
    let reference_contents = std::fs::read_to_string(reference)?;
    let exp_line_count = expected_contents.chars().filter(|&c| c == '\n').count();
    let ref_line_count = reference_contents.chars().filter(|&c| c == '\n').count();

    assert_eq!(
        exp_line_count, ref_line_count,
        "Files '{:?}' and '{:?}' differ in line count: {} vs {}",
        expected, reference, exp_line_count, ref_line_count
    );

    for (i, (e_line, r_line)) in expected_contents
        .lines()
        .zip(reference_contents.lines())
        .enumerate()
    {
        if e_line != r_line {
            println!(
                "Files '{:?}' and '{:?}' differ at line {}:",
                expected,
                reference,
                i + 1
            );
            println!("- {}", e_line);
            println!("+ {}", r_line);
            panic!(
                "Files '{:?}' and '{:?}' differ at line {}.",
                expected,
                reference,
                i + 1
            );
        }
    }
    Ok(())
}

// Needed for outputs.rs, since there only markdown files are compared.
#[allow(dead_code)]
pub fn regression_renderings(
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
        assert_files_equal(&output_file, Path::new(&output_name))?;
    }
    temp.close()?;
    Ok(())
}
