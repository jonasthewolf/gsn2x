use anyhow::{anyhow, Context, Result};
use std::{fs::create_dir_all, path::PathBuf};

///
/// Check that all inputs are relative path names.
/// Replace backslashes with slashes.
/// Find common ancestors.
///
///
pub fn prepare_and_check_input_paths(inputs: &mut [String]) -> Result<String> {
    if inputs.iter().all(|i| PathBuf::from(i).is_relative()) {
        // Replace backslash with slash
        inputs.iter_mut().for_each(|i| {
            *i = i.replace('\\', "/");
        });
        let common_ancestors = find_common_ancestors_in_paths(inputs)?;
        let cwd = std::env::current_dir()?.canonicalize()?;
        let result = strip_prefix(&common_ancestors, &cwd)
            .to_string_lossy()
            .into_owned();
        Ok(result)
    } else {
        Err(anyhow!("All input paths must be relative."))
    }
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
    let common = find_common_ancestors_in_paths(&[source.to_owned(), target.to_owned()])?;
    let source_canon_stripped = source_canon.strip_prefix(&common)?.to_path_buf();
    let mut target_canon_stripped = target_canon.strip_prefix(&common)?.to_path_buf();
    let mut prefix = match source_canon_stripped
        .parent()
        .map(|p| p.components().count())
        .unwrap_or(0)
    {
        0 => "./".to_owned(),
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
    let filename = path.rsplit(['/', '\\']).next().unwrap();
    if filename.is_empty() || filename == ".." || filename == "." {
        None
    } else {
        Some(filename)
    }
}

///
/// Set extension `ext` for file in `path`.
/// If no extension in `path` is found, `ext` is added.
///
pub fn set_extension(path: &str, ext: &str) -> String {
    let split: Vec<_> = path.rsplitn(2, '.').collect();
    format!("{}.{}", split.last().unwrap(), ext) // unwrap ok, since there is always a last()
}

///
/// Find common ancestors in all paths in `inputs`.
/// The output is an absolute path containing all common ancestors.
///
fn find_common_ancestors_in_paths(inputs: &[String]) -> Result<PathBuf> {
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
/// Strip `prefix` from `path`
/// If prefix is not part of `path`, `path` is returned.
///
pub fn strip_prefix(path: &PathBuf, prefix: &PathBuf) -> PathBuf {
    if path.starts_with(prefix) {
        path.strip_prefix(prefix).unwrap().to_owned()
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

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    #[test]
    fn common_ancestor_many() -> Result<()> {
        let inputs = [
            "examples/modular/sub1.gsn.yaml".to_owned(),
            "examples/modular/main.gsn.yaml".to_owned(),
        ];
        let mut result = find_common_ancestors_in_paths(&inputs)?;
        let cwd = std::env::current_dir()?.canonicalize()?;
        result = strip_prefix(&result, &cwd);
        assert_eq!(result, PathBuf::from("examples/modular"));
        Ok(())
    }

    #[test]
    fn common_ancestor_single() -> Result<()> {
        let inputs = ["examples/example.gsn.yaml".to_owned()];
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
