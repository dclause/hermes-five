//! This file contains helper to manipulate files from the application.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;

/// Resolves a given file path relative to app root (as opposed to console path) into an absolute path.
///
/// # Parameters
/// * `path`: The path for the file to resolve.
///
/// # Return
/// Returns a `Result<PathBuf>` matching the resolved absolute file path.
///
/// # Example
/// ```
/// // Suppose this project in `/home/pi/user`
///  let file = resolve_file("./Cargo.toml").unwrap();
/// // Resolves to `/home/pi/user/Cargo.toml` if the file exists.
/// ```
#[allow(dead_code)]
pub fn resolve_file<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let file_path = path.as_ref();
    let absolute_path: PathBuf = match file_path.is_absolute() {
        true => file_path.to_path_buf(),
        false => env::current_dir()?.join(file_path),
    };
    Ok(absolute_path)
}

#[cfg(test)]
mod tests {
    use anyhow::{bail, Result};

    use crate::utils::file::resolve_file;

    #[test]
    fn test_resolve_relative_file() -> Result<()> {
        let path = format!("../{}", file!());
        let file = resolve_file(&path).unwrap();
        let output = match std::fs::canonicalize(&file) {
            Ok(output) => output,
            Err(err) => bail!("{}: with path {:?}", err, file),
        };
        let expected = std::fs::canonicalize(&path).unwrap();
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_resolve_absolute_file() -> Result<()> {
        let path = format!("../{}", file!());
        let file = resolve_file(&path).unwrap();
        let output = match std::fs::canonicalize(&file) {
            Ok(output) => output,
            Err(err) => bail!("{}: with path {:?}", err, file),
        };
        let expected = resolve_file(std::fs::canonicalize(&path).unwrap()).unwrap();
        assert_eq!(output, expected);
        Ok(())
    }
}
