use anyhow::{Context, Result};
use assert_cmd::prelude::*;
use assert_fs::fixture::PathCopy;
use assert_fs::prelude::*;
use regex::Regex;
use std::process::Command;

pub fn split_keep<'a>(regex: &Regex, input: &'a str) -> Vec<&'a str> {
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

pub fn check_within_tolerance(l: i64, r: i64) -> bool {
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

pub fn compare_lines_with_replace(
    left: &std::ffi::OsStr,
    right: &std::ffi::OsStr,
    replace_regex: Option<Vec<(Regex, &str)>>,
) -> Result<bool> {
    let left: &std::path::Path = left.as_ref();
    let right: &std::path::Path = right.as_ref();
    let left_c = std::fs::read_to_string(left).with_context(|| format!("Filename {left:?}"))?;
    let right_c = std::fs::read_to_string(right).with_context(|| format!("Filename {right:?}"))?;
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
                println!("Splitted Line: {l_split:?} {r_split:?}");
                same = false;
                break;
            }
        }
    } else {
        same = false;
    }

    Ok(same)
}

pub fn are_struct_similar_svgs(left: &std::ffi::OsStr, right: &std::ffi::OsStr) -> Result<bool> {
    // Order is important.
    let replaces = vec![(
        Regex::new(r" gsn_module_\w+").unwrap(),
        " gsn_module_replaced",
    )];

    compare_lines_with_replace(left, right, Some(replaces))
}

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
        assert!(are_struct_similar_svgs(
            std::path::Path::new(&output_name).as_os_str(),
            output_file.as_os_str(),
        )?);
    }
    temp.close()?;
    Ok(())
}
