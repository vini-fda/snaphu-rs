#![allow(dead_code)]

//! Output helpers for unwrapped phase products.

use crate::io::reader::parse_filename;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_DUMP_PATH: &str = "/tmp/";

#[derive(Debug)]
pub struct OpenedOutputFile {
    pub file: File,
    pub real_path: PathBuf,
    pub fell_back: bool,
}

/// Open a file for writing, falling back to `/tmp/<basename>` if needed.
///
/// This is the idiomatic Rust equivalent of the C `OpenOutputFile()` helper.
/// It returns both the opened file and the effective path used (matching
/// C's `realoutfile` out-parameter behavior).
pub fn open_output_file(outfile: &Path) -> io::Result<OpenedOutputFile> {
    match File::create(outfile) {
        Ok(file) => Ok(OpenedOutputFile {
            file,
            real_path: outfile.to_path_buf(),
            fell_back: false,
        }),
        Err(primary_err) => {
            // Keep the C behavior: if primary open fails, reuse the basename
            // and retry under the dump directory.
            let (_, basename) = parse_filename(outfile)?;
            let dumpfile = Path::new(DEFAULT_DUMP_PATH).join(basename);
            match File::create(&dumpfile) {
                Ok(file) => {
                    log::warn!(
                        "Can't write to file {}. Dumping to file {}",
                        outfile.display(),
                        dumpfile.display()
                    );
                    Ok(OpenedOutputFile {
                        file,
                        real_path: dumpfile,
                        fell_back: true,
                    })
                }
                Err(fallback_err) => Err(io::Error::new(
                    fallback_err.kind(),
                    format!(
                        "unable to write to '{}' ({}), and unable to dump to '{}' ({})",
                        outfile.display(),
                        primary_err,
                        dumpfile.display(),
                        fallback_err
                    ),
                )),
            }
        }
    }
}

pub fn write_phase_file(_path: &std::path::Path) {
    // TODO: implement.
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_suffix() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}_{}", std::process::id(), nanos)
    }

    #[test]
    fn open_output_file_uses_requested_path_when_writable() {
        let dir = std::env::temp_dir().join(format!("snaphu_rs_open_ok_{}", unique_suffix()));
        fs::create_dir_all(&dir).unwrap();
        let requested = dir.join("out.bin");

        let opened = open_output_file(&requested).unwrap();
        assert_eq!(opened.real_path, requested);
        assert!(!opened.fell_back);
        drop(opened.file);

        assert!(requested.exists());
        fs::remove_file(&requested).unwrap();
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn open_output_file_falls_back_to_tmp_with_basename() {
        let requested = std::env::temp_dir()
            .join(format!("snaphu_rs_missing_parent_{}", unique_suffix()))
            .join(format!("out_{}.bin", unique_suffix()));
        let expected_fallback =
            Path::new(DEFAULT_DUMP_PATH).join(requested.file_name().unwrap().to_os_string());

        let opened = open_output_file(&requested).unwrap();
        assert!(opened.fell_back);
        assert_eq!(opened.real_path, expected_fallback);
        drop(opened.file);

        assert!(expected_fallback.exists());
        fs::remove_file(expected_fallback).unwrap();
    }

    #[test]
    fn open_output_file_invalid_root_path_errors() {
        let err = open_output_file(Path::new("/")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }
}
