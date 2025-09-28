use anyhow::Result;
use std::path::Path;

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
