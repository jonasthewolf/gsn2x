use anyhow::{Context, Result};
use std::{fs::create_dir_all, path::PathBuf};

///
/// Get a relative path from `source` to `target`.
/// If `target` is an absolute path, the absolute path to `target` is returned.
/// If `source` is an absolute path, but not `target` we calculate `target`'s absolute path.
/// The files don't need to exist.
///
pub fn get_relative_path(target: &str, source: &str) -> Result<String> {
    let source_path = &PathBuf::from(source);
    let target_path = &PathBuf::from(target);
    if target_path.is_absolute() {
        Ok(target_path.to_string_lossy().to_string())
    } else if source_path.is_absolute() {
        // Target path is relative
        let mut cwd = PathBuf::from(".").canonicalize().unwrap(); // unwrap ok, since current working directory must exist
        cwd.push(target_path);
        Ok(cwd.to_string_lossy().to_string())
    } else {
        // TODO if both (source and target) have some components in common: filter them
        // if both start with the same components (ignore CurDir), zip them
        // both are relative paths
        let mut relative_path = PathBuf::from_iter(
            source_path
                .parent()
                .unwrap() // unwrap ok, since file always has a parent
                .components()
                .filter_map(|c| match c {
                    // Map Normals to Parents
                    std::path::Component::CurDir => None,
                    std::path::Component::ParentDir => Some(c),
                    std::path::Component::Normal(_) => Some(std::path::Component::ParentDir),
                    _ => unreachable!(), // Root and Prefix should not be in relative paths.
                })
                .chain(
                    target_path
                        .parent()
                        .unwrap() // unwrap ok?
                        .components()
                        .filter_map(|c| match c {
                            // Map Normals to Normals
                            std::path::Component::CurDir => None,
                            std::path::Component::ParentDir => Some(c),
                            std::path::Component::Normal(n) => {
                                Some(std::path::Component::Normal(n))
                            }
                            _ => unreachable!(), // Root and Prefix should not be in relative paths.
                        }),
                ),
        );
        relative_path.push(target_path.file_name().unwrap()); // unwrap ok?

        Ok(relative_path.to_string_lossy().to_string())
    }
    // let common = find_common_ancestors_in_paths(&[source, target])?;
    // dbg!(&common);
    // let source_canon_stripped = source_path.strip_prefix(&common)?.to_path_buf();
    // let target_canon_stripped = target_path.strip_prefix(&common)?.to_path_buf();
    // let mut prefix = match source_canon_stripped
    //     .parent()
    //     .map(|p| p.components().count())
    //     .unwrap_or(0usize)
    // {
    //     0 => "./".to_owned(),
    //     x => "../".repeat(x), // x > 0
    // };
    // dbg!(&prefix);
    // prefix.push_str(&target_canon_stripped.to_string_lossy());
    // Ok(prefix)
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
/// Prefix `input_filename` with `output_path`.
///
/// If `input_filename` is a relative path, append it to output path.
/// If `input_filename` is an absolute path, put it directly in output path.
/// If `input_filename` starts with `output_path`, `input_filename` is used.
/// If directories up to the final path do not exist, they are created.
///
/// Function must not assume that output nor input with new extension exists.
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
        let rel = get_relative_path("./Cargo.toml", "examples/modular/index.gsn.yaml")?;
        assert_eq!(rel, "../../Cargo.toml");
        Ok(())
    }

    #[test]
    fn relative_path2() -> Result<()> {
        let rel = get_relative_path("../gsn2x/Cargo.toml", "./examples/modular/index.gsn.yaml")?;
        assert_eq!(rel, "../../../gsn2x/Cargo.toml");
        Ok(())
    }
}
