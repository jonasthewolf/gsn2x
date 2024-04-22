use anyhow::{Context, Result};
use std::{
    fs::create_dir_all,
    path::{Component, Path, PathBuf},
};

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
        // both are relative paths
        let common = get_stripped_components(source_path)
            .zip(get_stripped_components(target_path))
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| a)
            .collect::<Vec<_>>();
        let mut relative_path = PathBuf::from_iter(
            get_stripped_components(source_path)
                .skip(common.len())
                .filter_map(|c| match c {
                    // Map Normals to Parents
                    std::path::Component::ParentDir => Some(c),
                    std::path::Component::Normal(_) => Some(std::path::Component::ParentDir),
                    _ => unreachable!(), // Root and Prefix should not be in relative paths.
                })
                .chain(
                    get_stripped_components(target_path)
                        .skip(common.len())
                        .filter_map(|c| match c {
                            // Map Normals to Normals
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
    .map(|i| i.replace('\\', "/"))
}

///
/// Removes filename and CurDir if there.
///
///
fn get_stripped_components(path: &Path) -> impl Iterator<Item = Component> {
    path.parent()
        .unwrap() // TODO unwrap ok?
        .components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
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

    #[test]
    fn relative_path3() -> Result<()> {
        let rel = get_relative_path(
            "./examples/modular/sub1.gsn.yaml",
            "./examples/modular/sub3.gsn.yaml",
        )?;
        assert_eq!(rel, "sub1.gsn.yaml");
        Ok(())
    }
}
