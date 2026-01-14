use anyhow::Result;
use assert_cmd::cargo;
use assert_cmd::prelude::*;
use assert_fs::fixture::PathCopy;
use assert_fs::prelude::*;
use std::path::Path;

use crate::basics::assert_files_equal;
use std::process::Command;

pub fn regression_renderings(
    input: &[&str],
    options: &[&str],
    additional_files: Option<&[&str]>,
) -> Result<()> {
    let mut cmd = Command::new(cargo::cargo_bin!());
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
