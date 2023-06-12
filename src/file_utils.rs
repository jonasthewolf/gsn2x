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
    let cwd = std::env::current_dir()?
        .canonicalize()?
        .to_string_lossy()
        .into_owned();
    let relative_inputs = inputs
        .iter()
        .map(|&i| {
            let x = PathBuf::from(i);
            if x.is_relative() {
                Ok(i.to_owned())
            } else {
                let x = x.canonicalize()?.to_string_lossy().into_owned();
                if x.starts_with(&cwd) {
                    Ok(strip_prefix(&x, &cwd))
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
/// Optionally, change extension of `target` to `target_extension` after existence has been checked.
/// It does not matter if `source` or `target` are relative or absolute.
/// However, they must both exist.
///
pub fn get_relative_path(
    target: &str,
    source: &str,
    target_extension: Option<&str>,
) -> Result<String> {
    let source_canon = &PathBuf::from(source).canonicalize()?;
    let target_canon = &PathBuf::from(target).canonicalize()?;
    let common = find_common_ancestors_in_paths(&[source, target])?;
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
/// Get the filename part from `path`.
///
///
pub fn get_filename(path: &str) -> Option<&str> {
    match path.rsplit(['/', '\\']).next() {
        None => None,
        Some(x) if x == ".." || x == "." => None,
        Some(x) if x.is_empty() => None,
        Some(x) => Some(x),
    }
}

///
/// Set extension `ext` for file in `path`.
/// If no extension in `path` is found, new `ext` is added.
///
pub fn set_extension(path: &str, ext: &str) -> String {
    let split: Vec<_> = path.rsplitn(2, '.').collect();
    match split.len() {
        0 => unreachable!(),
        1..=2 => format!("{}.{}", split.last().unwrap(), ext),
        _ => unreachable!(), // because of rsplitn(2)
    }
}

///
/// Find common ancestors in all paths in `inputs`.
/// The output is an absolute path containing all common ancestors.
///
pub fn find_common_ancestors_in_paths(inputs: &[&str]) -> Result<String> {
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
    Ok(result.to_string_lossy().into_owned())
}

///
/// Strip `prefix` from `path`
/// If prefix is not part of `path`, `path` is returned.
///
pub fn strip_prefix(path: &str, prefix: &str) -> String {
    let path_buf = PathBuf::from(path);
    if path_buf.starts_with(prefix) {
        path_buf
            .strip_prefix(prefix)
            .unwrap()
            .to_string_lossy()
            .into_owned()
    } else {
        path.to_owned()
    }
}

///
/// Prefix `input_filename` with `output_path`.
///
/// `input_filename` is a relative path or only a filename.
/// If `common_ancestors` are provided, they are added between `output_path` and `input_filename`.
/// If directories up to the final path do not exist, they are created.
///
pub fn translate_to_output_path(
    output_path: &str,
    input_filename: &str,
    common_ancestors: Option<&str>,
) -> Result<String> {
    if PathBuf::from(input_filename).is_absolute() {
        Ok(input_filename.to_owned())
    } else {
        let mut output_path = std::path::PathBuf::from(&output_path);
        if let Some(common_ancestors) = common_ancestors {
            output_path.push(common_ancestors);
        }
        output_path.push(input_filename);
        if let Some(dir) = output_path.parent() {
            if !dir.exists() {
                create_dir_all(dir)
                    .with_context(|| format!("Trying to create directory {}", dir.to_string_lossy()))?;
            }
        }
        Ok(output_path.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    #[test]
    fn common_ancestor_many() -> Result<()> {
        let inputs = [
            "examples/modular/sub1.gsn.yaml",
            "examples/modular/main.gsn.yaml",
        ];
        let mut result = find_common_ancestors_in_paths(&inputs)?;
        let cwd = std::env::current_dir()?
            .canonicalize()?
            .to_string_lossy()
            .into_owned();
        result = strip_prefix(&result, &cwd);
        assert_eq!(result.replace('\\', "/"), "examples/modular");
        Ok(())
    }

    #[test]
    fn common_ancestor_single() -> Result<()> {
        let inputs = ["examples/example.gsn.yaml"];
        let result = find_common_ancestors_in_paths(&inputs)?;
        assert_eq!(result, "");
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
    }

    #[test]
    fn extension() {
        assert_eq!(set_extension("path", "ext"), "path.ext".to_owned());
        assert_eq!(
            set_extension("/var/log/some_text.txt", "svg"),
            "/var/log/some_text.svg".to_owned()
        );
        assert_eq!(
            set_extension("examples/example.gsn.yaml", "svg"),
            "examples/example.gsn.svg".to_owned()
        );
        assert_eq!(set_extension("", "test"), ".test".to_owned());
    }
}
