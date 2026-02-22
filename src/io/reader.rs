#![allow(dead_code)]

//! File-reading helpers for rasters and metadata.

use crate::data::raster::Raster;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub fn read_phase_file(_path: &std::path::Path) {
    // TODO: implement.
}

/// Window describing the subset of a larger raster to read from disk.
///
/// Mirrors the relevant `tileparamT` fields used by the C reader helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileWindow {
    pub first_row: usize,
    pub first_col: usize,
    pub nrow: usize,
    pub ncol: usize,
}

impl TileWindow {
    pub fn new(first_row: usize, first_col: usize, nrow: usize, ncol: usize) -> Self {
        Self {
            first_row,
            first_col,
            nrow,
            ncol,
        }
    }
}

/// Binary element type stored in native-endian raw rasters.
///
/// C's `Read2DArray()` accepted a raw element size and treated values as
/// native byte order. This trait gives that behavior a typed Rust API.
pub trait NativeSample: Copy {
    const SIZE: usize;
    fn from_ne_bytes(bytes: &[u8]) -> Self;
}

macro_rules! impl_native_sample {
    ($ty:ty, $size:expr) => {
        impl NativeSample for $ty {
            const SIZE: usize = $size;
            fn from_ne_bytes(bytes: &[u8]) -> Self {
                let arr: [u8; $size] = bytes.try_into().expect("invalid sample width");
                <$ty>::from_ne_bytes(arr)
            }
        }
    };
}

impl_native_sample!(i16, 2);
impl_native_sample!(u16, 2);
impl_native_sample!(i32, 4);
impl_native_sample!(u32, 4);
impl_native_sample!(f32, 4);
impl_native_sample!(i64, 8);
impl_native_sample!(u64, 8);
impl_native_sample!(f64, 8);

impl NativeSample for u8 {
    const SIZE: usize = 1;
    fn from_ne_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}

impl NativeSample for i8 {
    const SIZE: usize = 1;
    fn from_ne_bytes(bytes: &[u8]) -> Self {
        bytes[0] as i8
    }
}

/// Reads file of real data of size `T::SIZE` in native byte order.
///
/// This is the idiomatic Rust equivalent of the C `Read2DArray()` function.
/// It validates the exact on-disk dimensions and then reads only the tile
/// window requested by `window`.
pub fn read_2d_array<T: NativeSample>(
    infile: &Path,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
) -> io::Result<Raster<T>> {
    if window.first_row > nlines
        || window.first_col > line_len
        || window.first_row + window.nrow > nlines
        || window.first_col + window.ncol > line_len
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile window exceeds source raster bounds",
        ));
    }

    let expected_size = nlines
        .checked_mul(line_len)
        .and_then(|v| v.checked_mul(T::SIZE))
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "expected file size overflow")
        })?;

    let mut fp = File::open(infile)?;
    let filesize = fp.metadata()?.len() as usize;
    if filesize != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} wrong size ({}x{} array expected)",
                infile.display(),
                nlines,
                line_len
            ),
        ));
    }

    let start_byte = (line_len * window.first_row + window.first_col)
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek offset overflow"))?;
    fp.seek(SeekFrom::Start(start_byte as u64))?;

    let row_bytes = window
        .ncol
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row read width overflow"))?;
    let pad_bytes = (line_len - window.ncol)
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row padding overflow"))?;

    let mut data = Vec::with_capacity(window.nrow * window.ncol);
    let mut rowbuf = vec![0u8; row_bytes];
    for _ in 0..window.nrow {
        if row_bytes > 0 {
            fp.read_exact(&mut rowbuf)?;
            for chunk in rowbuf.chunks_exact(T::SIZE) {
                data.push(T::from_ne_bytes(chunk));
            }
        }

        if pad_bytes > 0 {
            let skip = i64::try_from(pad_bytes).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "row padding too large to seek")
            })?;
            fp.seek(SeekFrom::Current(skip))?;
        }
    }

    Ok(Raster::new(window.ncol, window.nrow, data))
}

/// Read data from a file containing alternating full lines of magnitude and
/// phase floats: `[mag line][phase line][mag line][phase line]...`.
///
/// This is the idiomatic Rust equivalent of the C `ReadAltLineFile()`
/// function.
pub fn read_alt_line_file(
    alfile: &Path,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
) -> io::Result<(Raster<f32>, Raster<f32>)> {
    if window.first_row > nlines
        || window.first_col > line_len
        || window.first_row + window.nrow > nlines
        || window.first_col + window.ncol > line_len
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile window exceeds source raster bounds",
        ));
    }

    let expected_size = 2usize
        .checked_mul(nlines)
        .and_then(|v| v.checked_mul(line_len))
        .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "expected file size overflow")
        })?;

    let mut fp = File::open(alfile)?;
    let filesize = fp.metadata()?.len() as usize;
    if filesize != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} wrong size ({}x{} array expected)",
                alfile.display(),
                nlines,
                line_len
            ),
        ));
    }

    let start_byte = (window
        .first_row
        .checked_mul(2)
        .and_then(|v| v.checked_mul(line_len))
        .and_then(|v| v.checked_add(window.first_col)))
    .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek offset overflow"))?;
    fp.seek(SeekFrom::Start(start_byte as u64))?;

    let row_bytes = window
        .ncol
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row read width overflow"))?;
    let pad_bytes = (line_len - window.ncol)
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row padding overflow"))?;
    let pad_skip = i64::try_from(pad_bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "row padding too large to seek")
    })?;

    let mut mag_data = Vec::with_capacity(window.nrow * window.ncol);
    let mut phase_data = Vec::with_capacity(window.nrow * window.ncol);
    let mut rowbuf = vec![0u8; row_bytes];
    for _ in 0..window.nrow {
        if row_bytes > 0 {
            fp.read_exact(&mut rowbuf)?;
            for chunk in rowbuf.chunks_exact(std::mem::size_of::<f32>()) {
                let arr: [u8; 4] = chunk.try_into().expect("invalid f32 row chunk");
                mag_data.push(f32::from_ne_bytes(arr));
            }
        }

        if pad_skip > 0 {
            fp.seek(SeekFrom::Current(pad_skip))?;
        }

        if row_bytes > 0 {
            fp.read_exact(&mut rowbuf)?;
            for chunk in rowbuf.chunks_exact(std::mem::size_of::<f32>()) {
                let arr: [u8; 4] = chunk.try_into().expect("invalid f32 row chunk");
                phase_data.push(f32::from_ne_bytes(arr));
            }
        }

        if pad_skip > 0 {
            fp.seek(SeekFrom::Current(pad_skip))?;
        }
    }

    Ok((
        Raster::new(window.ncol, window.nrow, mag_data),
        Raster::new(window.ncol, window.nrow, phase_data),
    ))
}

/// Read only the phase data from a file containing alternating full lines of
/// magnitude and phase floats.
///
/// This is the idiomatic Rust equivalent of the C `ReadAltLineFilePhase()`
/// function.
pub fn read_alt_line_file_phase(
    alfile: &Path,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
) -> io::Result<Raster<f32>> {
    if window.first_row > nlines
        || window.first_col > line_len
        || window.first_row + window.nrow > nlines
        || window.first_col + window.ncol > line_len
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile window exceeds source raster bounds",
        ));
    }

    let expected_size = 2usize
        .checked_mul(nlines)
        .and_then(|v| v.checked_mul(line_len))
        .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "expected file size overflow")
        })?;

    let mut fp = File::open(alfile)?;
    let filesize = fp.metadata()?.len() as usize;
    if filesize != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} wrong size ({}x{} array expected)",
                alfile.display(),
                nlines,
                line_len
            ),
        ));
    }

    let start_byte = (window
        .first_row
        .checked_mul(2)
        .and_then(|v| v.checked_mul(line_len))
        .and_then(|v| v.checked_add(line_len))
        .and_then(|v| v.checked_add(window.first_col)))
    .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek offset overflow"))?;
    fp.seek(SeekFrom::Start(start_byte as u64))?;

    let row_bytes = window
        .ncol
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row read width overflow"))?;
    let pad_bytes = (2usize
        .checked_mul(line_len)
        .and_then(|v| v.checked_sub(window.ncol)))
    .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row padding overflow"))?;
    let pad_skip = i64::try_from(pad_bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "row padding too large to seek")
    })?;

    let mut phase_data = Vec::with_capacity(window.nrow * window.ncol);
    let mut rowbuf = vec![0u8; row_bytes];
    for _ in 0..window.nrow {
        if row_bytes > 0 {
            fp.read_exact(&mut rowbuf)?;
            for chunk in rowbuf.chunks_exact(std::mem::size_of::<f32>()) {
                let arr: [u8; 4] = chunk.try_into().expect("invalid f32 row chunk");
                phase_data.push(f32::from_ne_bytes(arr));
            }
        }

        if pad_skip > 0 {
            fp.seek(SeekFrom::Current(pad_skip))?;
        }
    }

    Ok(Raster::new(window.ncol, window.nrow, phase_data))
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
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_suffix() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}_{}", std::process::id(), nanos)
    }

    fn temp_file(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{}_{}", name, unique_suffix()))
    }

    #[test]
    fn read_2d_array_reads_selected_tile_window() {
        let path = temp_file("snaphu_rs_read2d_f32");
        let mut fp = File::create(&path).unwrap();
        for value in 0..12u32 {
            fp.write_all(&(value as f32).to_ne_bytes()).unwrap();
        }
        drop(fp);

        let window = TileWindow::new(1, 1, 2, 2);
        let raster = read_2d_array::<f32>(&path, 4, 3, window).unwrap();
        assert_eq!(raster.width, 2);
        assert_eq!(raster.height, 2);
        assert_eq!(raster.data, vec![5.0, 6.0, 9.0, 10.0]);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_array_rejects_wrong_file_size() {
        let path = temp_file("snaphu_rs_read2d_badsize");
        let mut fp = File::create(&path).unwrap();
        // 3 u16 elements only, but 2x2 expects 4.
        for value in 0..3u16 {
            fp.write_all(&value.to_ne_bytes()).unwrap();
        }
        drop(fp);

        let err = read_2d_array::<u16>(&path, 2, 2, TileWindow::new(0, 0, 2, 2)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("wrong size"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_array_rejects_out_of_bounds_window() {
        let path = temp_file("snaphu_rs_read2d_bounds");
        let mut fp = File::create(&path).unwrap();
        for value in 0..12u8 {
            fp.write_all(&[value]).unwrap();
        }
        drop(fp);

        let err = read_2d_array::<u8>(&path, 4, 3, TileWindow::new(2, 3, 2, 2)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("tile window"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_line_file_reads_mag_and_phase_tiles() {
        let path = temp_file("snaphu_rs_read_alt_line");
        let mut fp = File::create(&path).unwrap();
        for row in 0..3u32 {
            for col in 0..4u32 {
                let mag = (row * 10 + col) as f32;
                fp.write_all(&mag.to_ne_bytes()).unwrap();
            }
            for col in 0..4u32 {
                let phase = (row * 100 + col) as f32;
                fp.write_all(&phase.to_ne_bytes()).unwrap();
            }
        }
        drop(fp);

        let (mag, phase) = read_alt_line_file(&path, 4, 3, TileWindow::new(1, 1, 2, 2)).unwrap();
        assert_eq!(mag.width, 2);
        assert_eq!(mag.height, 2);
        assert_eq!(mag.data, vec![11.0, 12.0, 21.0, 22.0]);
        assert_eq!(phase.data, vec![101.0, 102.0, 201.0, 202.0]);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_line_file_rejects_wrong_file_size() {
        let path = temp_file("snaphu_rs_read_alt_line_badsize");
        let mut fp = File::create(&path).unwrap();
        // Too short for 2*nlines*line_len f32 values.
        fp.write_all(&1.0f32.to_ne_bytes()).unwrap();
        drop(fp);

        let err = read_alt_line_file(&path, 4, 3, TileWindow::new(0, 0, 1, 1)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("wrong size"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_line_file_rejects_out_of_bounds_window() {
        let path = temp_file("snaphu_rs_read_alt_line_bounds");
        let mut fp = File::create(&path).unwrap();
        for _ in 0..(2 * 3 * 4) {
            fp.write_all(&0.0f32.to_ne_bytes()).unwrap();
        }
        drop(fp);

        let err = read_alt_line_file(&path, 4, 3, TileWindow::new(2, 3, 2, 2)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("tile window"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_line_file_phase_reads_phase_tile_only() {
        let path = temp_file("snaphu_rs_read_alt_line_phase");
        let mut fp = File::create(&path).unwrap();
        for row in 0..3u32 {
            for col in 0..4u32 {
                let mag = (row * 10 + col) as f32;
                fp.write_all(&mag.to_ne_bytes()).unwrap();
            }
            for col in 0..4u32 {
                let phase = (row * 100 + col) as f32;
                fp.write_all(&phase.to_ne_bytes()).unwrap();
            }
        }
        drop(fp);

        let phase = read_alt_line_file_phase(&path, 4, 3, TileWindow::new(1, 1, 2, 2)).unwrap();
        assert_eq!(phase.width, 2);
        assert_eq!(phase.height, 2);
        assert_eq!(phase.data, vec![101.0, 102.0, 201.0, 202.0]);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_line_file_phase_rejects_wrong_file_size() {
        let path = temp_file("snaphu_rs_read_alt_line_phase_badsize");
        let mut fp = File::create(&path).unwrap();
        fp.write_all(&1.0f32.to_ne_bytes()).unwrap();
        drop(fp);

        let err = read_alt_line_file_phase(&path, 4, 3, TileWindow::new(0, 0, 1, 1)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("wrong size"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_line_file_phase_rejects_out_of_bounds_window() {
        let path = temp_file("snaphu_rs_read_alt_line_phase_bounds");
        let mut fp = File::create(&path).unwrap();
        for _ in 0..(2 * 3 * 4) {
            fp.write_all(&0.0f32.to_ne_bytes()).unwrap();
        }
        drop(fp);

        let err = read_alt_line_file_phase(&path, 4, 3, TileWindow::new(2, 3, 2, 2)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("tile window"));

        fs::remove_file(path).unwrap();
    }

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
