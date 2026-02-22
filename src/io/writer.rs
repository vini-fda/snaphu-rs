#![allow(dead_code)]

//! Output helpers for unwrapped phase products.

use crate::data::raster::Raster;
use crate::io::reader::parse_filename;
use std::fs::File;
use std::io::{self, Write};
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

pub trait NativeWritable {
    fn write_ne<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

macro_rules! impl_native_writable {
    ($ty:ty) => {
        impl NativeWritable for $ty {
            fn write_ne<W: Write>(&self, writer: &mut W) -> io::Result<()> {
                writer.write_all(&self.to_ne_bytes())
            }
        }
    };
}

impl NativeWritable for u8 {
    fn write_ne<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[*self])
    }
}

impl NativeWritable for i8 {
    fn write_ne<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[*self as u8])
    }
}

impl_native_writable!(u16);
impl_native_writable!(i16);
impl_native_writable!(u32);
impl_native_writable!(i32);
impl_native_writable!(u64);
impl_native_writable!(i64);
impl_native_writable!(f32);
impl_native_writable!(f64);

/// Write a contiguous 2-D array to disk in native-endian element order.
///
/// This is the idiomatic Rust equivalent of the C `Write2DArray()` helper.
pub fn write_2d_array<T: NativeWritable>(
    array: &[T],
    nrow: usize,
    ncol: usize,
    filename: &Path,
) -> io::Result<PathBuf> {
    let expected = nrow
        .checked_mul(ncol)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "array dimensions overflow"))?;
    if array.len() != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "array length does not match nrow*ncol",
        ));
    }

    let mut opened = open_output_file(filename)?;
    for row in 0..nrow {
        let start = row * ncol;
        let end = start + ncol;
        for sample in &array[start..end] {
            sample.write_ne(&mut opened.file)?;
        }
    }
    opened.file.flush()?;
    Ok(opened.real_path)
}

/// Write magnitude and phase rasters as alternating full lines:
/// `[mag row][phase row][mag row][phase row]...`.
///
/// This is the idiomatic Rust equivalent of the C `WriteAltLineFile()`.
pub fn write_alt_line_file(
    mag: &Raster<f32>,
    phase: &Raster<f32>,
    outfile: &Path,
) -> io::Result<PathBuf> {
    if mag.width != phase.width || mag.height != phase.height {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "mag and phase raster dimensions must match",
        ));
    }

    let mut opened = open_output_file(outfile)?;
    for row in 0..mag.height {
        let start = row * mag.width;
        let end = start + mag.width;
        for sample in &mag.data[start..end] {
            opened.file.write_all(&sample.to_ne_bytes())?;
        }
        for sample in &phase.data[start..end] {
            opened.file.write_all(&sample.to_ne_bytes())?;
        }
    }
    opened.file.flush()?;
    Ok(opened.real_path)
}

/// Write two rasters as alternating samples per line:
/// `a0, b0, a1, b1, ...`.
///
/// This is the idiomatic Rust equivalent of the C `WriteAltSampFile()`.
pub fn write_alt_samp_file(
    arr1: &Raster<f32>,
    arr2: &Raster<f32>,
    outfile: &Path,
) -> io::Result<PathBuf> {
    if arr1.width != arr2.width || arr1.height != arr2.height {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input raster dimensions must match",
        ));
    }

    let mut opened = open_output_file(outfile)?;
    for row in 0..arr1.height {
        let start = row * arr1.width;
        let end = start + arr1.width;
        for col in 0..arr1.width {
            opened
                .file
                .write_all(&arr1.data[start + col].to_ne_bytes())?;
            opened
                .file
                .write_all(&arr2.data[start + col].to_ne_bytes())?;
        }
        debug_assert_eq!(end, start + arr1.width);
    }
    opened.file.flush()?;
    Ok(opened.real_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::reader::{TileWindow, read_2d_array, read_alt_line_file, read_alt_samp_file};
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

    #[test]
    fn write_2d_array_round_trips_native_values() {
        let path = std::env::temp_dir().join(format!("snaphu_rs_write2d_{}.bin", unique_suffix()));
        let data: Vec<u16> = (0..12u16).collect();
        let written = write_2d_array(&data, 3, 4, &path).unwrap();
        let raster = read_2d_array::<u16>(&written, 4, 3, TileWindow::new(0, 0, 3, 4)).unwrap();
        assert_eq!(raster.data, data);
        fs::remove_file(written).unwrap();
    }

    #[test]
    fn write_alt_line_file_round_trips_mag_phase() {
        let path =
            std::env::temp_dir().join(format!("snaphu_rs_write_altline_{}.bin", unique_suffix()));
        let mag = Raster::new(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let phase = Raster::new(3, 2, vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
        let written = write_alt_line_file(&mag, &phase, &path).unwrap();
        let (rm, rp) = read_alt_line_file(&written, 3, 2, TileWindow::new(0, 0, 2, 3)).unwrap();
        assert_eq!(rm.data, mag.data);
        assert_eq!(rp.data, phase.data);
        fs::remove_file(written).unwrap();
    }

    #[test]
    fn write_alt_samp_file_round_trips_arrays() {
        let path =
            std::env::temp_dir().join(format!("snaphu_rs_write_altsamp_{}.bin", unique_suffix()));
        let a = Raster::new(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b = Raster::new(2, 3, vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
        let written = write_alt_samp_file(&a, &b, &path).unwrap();
        let (ra, rb) = read_alt_samp_file(&written, 2, 3, TileWindow::new(0, 0, 3, 2)).unwrap();
        assert_eq!(ra.data, a.data);
        assert_eq!(rb.data, b.data);
        fs::remove_file(written).unwrap();
    }
}
