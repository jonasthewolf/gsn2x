use anyhow::{Context, Result};
use std::{fs::create_dir_all, path::PathBuf};

///
/// Get a relative path from `source` to `target`.
/// Optionally, change extension of `target` to `target_extension` after existence has been checked.
/// It does not matter if `source` or `target` are relative or absolute.
/// However, they must both exist.
///
pub fn get_relative_path(
    target: &str,
    source: &str,
    target_extension: Option<&str>,
) -> Result<String> {
    let source_canon = &PathBuf::from(source).canonicalize().with_context(|| format!("Could not find relative path between {source} and {target}, because {source} not found"))?;
    let target_canon = &PathBuf::from(target).canonicalize().with_context(|| format!("Could not find relative path between {source} and {target}, because {target} not found"))?;
    let common = find_common_ancestors_in_paths(&[source, target])?;
    let source_canon_stripped = source_canon.strip_prefix(&common)?.to_path_buf();
    let mut target_canon_stripped = target_canon.strip_prefix(&common)?.to_path_buf();
    let mut prefix = match source_canon_stripped
        .parent()
        .map(|p| p.components().count())
        .unwrap_or(0usize)
    {
        0 => "./".to_owned(),
        x => "../".repeat(x), // x > 0
    };
    if let Some(ext) = target_extension {
        target_canon_stripped.set_extension(ext);
    }
    prefix.push_str(&target_canon_stripped.to_string_lossy());
    Ok(prefix)
}

///
/// Get the filename part from `path`.
///
///
pub fn get_filename(path: &str) -> Option<&str> {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|&filename| !(filename.is_empty() || filename == ".." || filename == "."))
}

///
/// Find common ancestors in all paths in `inputs`.
/// The output is an absolute path containing all common ancestors.
///
fn find_common_ancestors_in_paths(inputs: &[&str]) -> Result<PathBuf> {
    let input_paths = inputs
        .iter()
        .map(|i| {
            PathBuf::from(i)
                .canonicalize()
                .with_context(|| format!("Failed to open file {}", i))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let components = input_paths
        .iter()
        .map(|p| p.components().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let mut result = PathBuf::new();

    if let Some(min_components) = components.iter().map(|c| c.len()).min() {
        for component in 1..min_components {
            if components
                .iter()
                .skip(1)
                .scan(components[0][component], |orig, x| {
                    if x[component] == *orig {
                        Some(1)
                    } else {
                        None
                    }
                })
                .count()
                > 0
            {
                if component == 1 {
                    result.push(components[0][0]);
                }
                result.push(components[0][component]);
            } else {
                break;
            }
        }
    }
    Ok(result)
}

///
/// Prefix `input_filename` with `output_path`.
///
/// If `input_filename` is a relative path, append it to output path.
/// If `input_filename` is an absolute path, put it directly in output path.
/// If `input_filename` starts with `output_path`, `input_filename` is used.
/// If directories up to the final path do not exist, they are created.
///
pub fn translate_to_output_path(
    output_path: &str,
    input: &str,
    new_extension: Option<&str>,
) -> Result<String> {
    let mut input_path_buf = std::path::PathBuf::from(input);
    if let Some(new_extension) = new_extension {
        input_path_buf.set_extension(new_extension);
    }
    let mut output_path_buf = std::path::PathBuf::from(&output_path);
    if input_path_buf.is_relative() {
        output_path_buf.push(if input_path_buf.starts_with(&output_path_buf) {
            input_path_buf.strip_prefix(&output_path_buf)?
        } else {
            &input_path_buf
        });
    } else {
        // absolute assumed
        let filename = input_path_buf.file_name().unwrap(); // unwrap ok, since file exists and we already read from it
        output_path_buf.push(filename);
    }
    // TODO move to other place
    if let Some(dir) = output_path_buf.parent() {
        if !dir.exists() {
            create_dir_all(dir)
                .with_context(|| format!("Trying to create directory {}", dir.display()))?;
        }
    }
    Ok(output_path_buf.to_string_lossy().into_owned())
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    #[test]
    fn common_ancestor_single() -> Result<()> {
        let inputs = ["examples/example.gsn.yaml"];
        let result = find_common_ancestors_in_paths(&inputs)?;
        assert_eq!(result, PathBuf::from(""));
        Ok(())
    }

    #[test]
    fn filename() {
        assert_eq!(get_filename("C:\\Temp/test.txt"), Some("test.txt"));
        assert_eq!(get_filename("/var/tmp"), Some("tmp"));
        assert_eq!(get_filename(""), None);
        assert_eq!(get_filename("./"), None);
        assert_eq!(get_filename("./a.svg"), Some("a.svg"));
        assert_eq!(get_filename("./../b.svg"), Some("b.svg"));
        assert_eq!(get_filename("./.."), None);
        assert_eq!(get_filename("a.b"), Some("a.b"));
    }

    #[test]
    fn relative_path() -> Result<()> {
        let rel = get_relative_path("./Cargo.toml", "./examples/modular/index.gsn.yaml", None)?;
        assert_eq!(rel, "../../Cargo.toml");
        Ok(())
    }
}
