use anyhow::{Context, Error, Result};
use std::{fs::create_dir_all, path::PathBuf};

///
///
/// Find relative path names for input files (if they are e.g. given )
///
/// The output is a list of tuples.
/// 1. element of tuple: the filename as provided on the command line
/// 2. element of tuple: the part that all input files have in common as relative paths
/// This allows using the 2. prefixed with the output directory to make out-of-tree builds.
///
pub fn prepare_input_paths(inputs: Vec<&str>) -> Result<Vec<(String, String)>> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    let relative_inputs = inputs
        .iter()
        .map(|&i| {
            let x = PathBuf::from(i);
            if x.is_relative() {
                Ok(i.to_owned())
            } else {
                let x = x.canonicalize().unwrap();
                if x.starts_with(&cwd) {
                    x.strip_prefix(&cwd)
                        .map(|i| i.to_string_lossy().into_owned())
                        .map_err(Error::from)
                } else {
                    get_relative_path(&x, &cwd, None)
                }
            }
        })
        .collect::<Result<Vec<String>, Error>>()?;
    let all_inputs = relative_inputs
        .into_iter()
        .zip(inputs)
        .map(|(r, i)| (i.to_owned(), r))
        .collect::<Vec<_>>();
    Ok(all_inputs)
}

///
/// Get a relative path from `source` to `target`.
/// Optionally, change extension of `target` to `target_extension`.
/// It does not matter if `source` or `target` are relative or absolute.
/// However, they must both exist.
///
pub fn get_relative_path(
    target: &PathBuf,
    source: &PathBuf,
    target_extension: Option<&str>,
) -> Result<String> {
    let source_canon = &source.canonicalize()?;
    let target_canon = &target.canonicalize()?;
    let common = find_common_ancestors_in_paths(&[source.to_owned(), target.to_owned()])?;
    let source_canon_stripped = source_canon.strip_prefix(&common)?.to_path_buf();
    let mut target_canon_stripped = target_canon.strip_prefix(&common)?.to_path_buf();
    let mut prefix = match source_canon_stripped
        .parent()
        .map(|p| p.components().count())
        .unwrap_or(0)
    {
        x if x == 0 => "./".to_owned(),
        x if x > 0 => "../".repeat(x),
        _ => unreachable!(),
    };
    if let Some(ext) = target_extension {
        target_canon_stripped.set_extension(ext);
    }
    prefix.push_str(&target_canon_stripped.to_string_lossy());
    Ok(prefix)
}

///
/// Find common ancestors in all paths in `inputs`.
/// The output is an absolute path containing all common ancestors.
///
pub fn find_common_ancestors_in_paths(inputs: &[PathBuf]) -> Result<PathBuf> {
    let input_paths = inputs
        .iter()
        .map(|i| {
            PathBuf::from(i)
                .canonicalize()
                .with_context(|| format!("Failed to open file {}", i.display()))
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
/// `input_filename` is a relative path or only a filename.
///
///
pub fn translate_to_output_path(output_path: &str, input_filename: &PathBuf) -> Result<PathBuf> {
    let mut output_path = std::path::PathBuf::from(&output_path);
    output_path.push(input_filename);
    if let Some(dir) = output_path.parent() {
        if !dir.exists() {
            create_dir_all(dir)
                .with_context(|| format!("Trying to create directory {}", dir.to_string_lossy()))?;
        }
    }
    Ok(output_path)
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use std::path::PathBuf;

    use crate::file_utils::find_common_ancestors_in_paths;

    #[test]
    fn common_ancestor_many() -> Result<()> {
        let inputs = [
            PathBuf::from("examples/modular/sub1.gsn.yaml"),
            PathBuf::from("examples/modular/main.gsn.yaml"),
        ];
        let mut result = find_common_ancestors_in_paths(&inputs)?;
        let cwd = std::env::current_dir()?.canonicalize()?;
        if result.starts_with(&cwd) {
            result = result.strip_prefix(cwd)?.to_path_buf();
        }
        assert_eq!(result, PathBuf::from("examples/modular"));
        Ok(())
    }

    #[test]
    fn common_ancestor_single() -> Result<()> {
        let inputs = [PathBuf::from("examples/example.gsn.yaml")];
        let result = find_common_ancestors_in_paths(&inputs)?;
        assert_eq!(result, PathBuf::from(""));
        Ok(())
    }
}
