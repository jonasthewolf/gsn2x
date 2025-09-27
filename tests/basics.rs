use anyhow::{Context, Result};
use assert_cmd::prelude::*;
use assert_fs::fixture::PathCopy;
use assert_fs::prelude::*;
use std::process::Command;

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
        let left_c = std::fs::read_to_string(&output_name)
            .with_context(|| format!("Filename {output_name:?}"))?;
        let right_c = std::fs::read_to_string(&output_file)
            .with_context(|| format!("Filename {:?}", output_file.display()))?;
        assert_eq!(left_c, right_c);
    }
    temp.close()?;
    Ok(())
}
