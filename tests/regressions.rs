use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

mod basics;
mod reg_render;

use crate::reg_render::regression_renderings;

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
    regression_renderings(&["tests/issue358.yaml"], &["-l=layer1", "-l=layer2"], None)?;
    Ok(())
}

#[test]
fn issue365() -> Result<()> {
    regression_renderings(&["tests/issue365.yaml"], &["-w=35"], None)?;
    Ok(())
}

#[test]
fn issue371() -> Result<()> {
    regression_renderings(&["tests/issue371.yaml"], &[], None)?;
    Ok(())
}

#[test]
fn issue372() -> Result<()> {
    regression_renderings(&["tests/issue372.yaml"], &["-w=35"], None)?;
    Ok(())
}

#[test]
fn issue377() -> Result<()> {
    regression_renderings(&["tests/issue377.yaml"], &["-w=35"], None)?;
    Ok(())
}

#[test]
fn issue389() -> Result<()> {
    regression_renderings(&["tests/issue389.yaml"], &["-w=20"], None)?;
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
        &["-w=20", "-F", "-A"],
        Some(&["tests/issue433_2.yaml"]),
    )?;
    Ok(())
}

#[test]
fn issue393() -> Result<()> {
    regression_renderings(&["tests/issue393_1.yaml"], &["-w=20"], None)?;
    regression_renderings(&["tests/issue393_2.yaml"], &["-w=20"], None)?;
    Ok(())
}

#[test]
fn issue396() -> Result<()> {
    regression_renderings(&["tests/issue396.yaml"], &["-w=20", "-l=layer1"], None)?;
    Ok(())
}

#[test]
fn issue407() -> Result<()> {
    regression_renderings(&["tests/issue407.yaml"], &["-w=20", "-l=layer2"], None)?;
    Ok(())
}

#[test]
fn issue453() -> Result<()> {
    regression_renderings(&["tests/issue453.yaml"], &["-l=layer2"], None)?;
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
