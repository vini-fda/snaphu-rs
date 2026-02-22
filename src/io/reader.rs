#![allow(dead_code)]

//! File-reading helpers for rasters and metadata.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn read_phase_file(_path: &std::path::Path) {
    // TODO: implement.
}

/// Split a file path into its parent directory and base filename.
///
/// This is the idiomatic Rust equivalent of the C `ParseFilename()` function,
/// which manually tokenised on `"/"` to separate a path from its basename.
/// Rust's [`std::path::Path`] handles this natively and portably.
///
/// The returned directory always ends with a path separator (matching the C
/// behaviour of appending `"/"`). If the input is a bare filename with no
/// directory component, the returned path is empty.
///
/// # Errors
///
/// Returns an error if `filename` is empty or has no file-name component
/// (e.g. a bare `"/"`).
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use std::ffi::OsString;
/// use snaphu_rs::io::reader::parse_filename;
///
/// let (dir, base) = parse_filename(Path::new("/data/output/result.bin")).unwrap();
/// assert_eq!(dir, Path::new("/data/output/"));
/// assert_eq!(base, OsString::from("result.bin"));
/// ```
pub fn parse_filename(filename: &Path) -> std::io::Result<(PathBuf, OsString)> {
    let filename_str = filename.as_os_str();
    if filename_str.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "zero-length filename passed to parse_filename()",
        ));
    }

    let basename = filename
        .file_name()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "zero-length base filename found in parse_filename()",
            )
        })?
        .to_os_string();

    // Build the directory path, ensuring a trailing separator just like the C
    // version appends "/".
    let dir = match filename.parent() {
        Some(p) if !p.as_os_str().is_empty() => {
            let mut d = p.to_path_buf().into_os_string();
            d.push(std::path::MAIN_SEPARATOR_STR);
            PathBuf::from(d)
        }
        _ => PathBuf::new(),
    };

    Ok((dir, basename))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::Path;

    #[test]
    fn absolute_path() {
        let (dir, base) = parse_filename(Path::new("/data/output/result.bin")).unwrap();
        assert_eq!(dir, Path::new("/data/output/"));
        assert_eq!(base, OsString::from("result.bin"));
    }

    #[test]
    fn relative_path() {
        let (dir, base) = parse_filename(Path::new("some/dir/file.dat")).unwrap();
        assert_eq!(dir, Path::new("some/dir/"));
        assert_eq!(base, OsString::from("file.dat"));
    }

    #[test]
    fn bare_filename() {
        let (dir, base) = parse_filename(Path::new("output.bin")).unwrap();
        assert_eq!(dir, PathBuf::new());
        assert_eq!(base, OsString::from("output.bin"));
    }

    #[test]
    fn root_relative_file() {
        let (dir, base) = parse_filename(Path::new("/file.bin")).unwrap();
        assert_eq!(dir, Path::new("/"));
        assert_eq!(base, OsString::from("file.bin"));
    }

    #[test]
    fn empty_filename_errors() {
        let err = parse_filename(Path::new("")).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("zero-length filename"));
    }

    #[test]
    fn root_only_errors() {
        // "/" has no file_name component — mirrors the C code rejecting
        // a zero-length basename.
        let err = parse_filename(Path::new("/")).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("zero-length base filename"));
    }

    #[test]
    fn deeply_nested() {
        let (dir, base) = parse_filename(Path::new("/a/b/c/d/e.f")).unwrap();
        assert_eq!(dir, Path::new("/a/b/c/d/"));
        assert_eq!(base, OsString::from("e.f"));
    }
}
