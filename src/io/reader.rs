#![allow(dead_code)]

//! File-reading helpers for rasters and metadata.

use crate::data::ops::{non_neg_data_array, valid_data_array};
use crate::data::raster::Raster;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub fn read_phase_file(_path: &std::path::Path) {
    // TODO: implement.
}

/// Supported on-disk raster encodings used by the legacy SNAPHU readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterFileFormat {
    FloatData,
    AltSampleData,
    AltLineData,
}

/// File configuration for intensity reads (`ReadIntensity` equivalent).
#[derive(Debug, Clone)]
pub struct IntensityFiles {
    pub ampfile: PathBuf,
    pub ampfile2: Option<PathBuf>,
    pub ampfile_format: RasterFileFormat,
}

/// File configuration for correlation reads (`ReadCorrelation` equivalent).
#[derive(Debug, Clone)]
pub struct CorrelationFile {
    pub corrfile: PathBuf,
    pub corrfile_format: RasterFileFormat,
}

/// Output bundle for intensity reads.
#[derive(Debug, Clone)]
pub struct IntensityData {
    pub pwr: Raster<f32>,
    pub pwr1: Option<Raster<f32>>,
    pub pwr2: Option<Raster<f32>>,
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

/// Read data from a file containing alternating float samples from two images:
/// `a0, b0, a1, b1, ...`.
///
/// This is the idiomatic Rust equivalent of the C `ReadAltSampFile()`
/// function.
pub fn read_alt_samp_file(
    infile: &Path,
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

    let start_byte = (window
        .first_row
        .checked_mul(line_len)
        .and_then(|v| v.checked_add(window.first_col))
        .and_then(|v| v.checked_mul(2)))
    .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek offset overflow"))?;
    fp.seek(SeekFrom::Start(start_byte as u64))?;

    let interleaved_row_bytes = window
        .ncol
        .checked_mul(2)
        .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row read width overflow"))?;
    let pad_bytes = (line_len - window.ncol)
        .checked_mul(2)
        .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row padding overflow"))?;
    let pad_skip = i64::try_from(pad_bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "row padding too large to seek")
    })?;

    let mut arr1_data = Vec::with_capacity(window.nrow * window.ncol);
    let mut arr2_data = Vec::with_capacity(window.nrow * window.ncol);
    let mut rowbuf = vec![0u8; interleaved_row_bytes];
    for _ in 0..window.nrow {
        if interleaved_row_bytes > 0 {
            fp.read_exact(&mut rowbuf)?;
            for pair in rowbuf.chunks_exact(2 * std::mem::size_of::<f32>()) {
                let a: [u8; 4] = pair[0..4].try_into().expect("invalid f32 sample chunk");
                let b: [u8; 4] = pair[4..8].try_into().expect("invalid f32 sample chunk");
                arr1_data.push(f32::from_ne_bytes(a));
                arr2_data.push(f32::from_ne_bytes(b));
            }
        }

        if pad_skip > 0 {
            fp.seek(SeekFrom::Current(pad_skip))?;
        }
    }

    Ok((
        Raster::new(window.ncol, window.nrow, arr1_data),
        Raster::new(window.ncol, window.nrow, arr2_data),
    ))
}

/// Read brightness/intensity inputs, optionally from two files.
///
/// This is the idiomatic Rust equivalent of the C `ReadIntensity()`
/// function.
pub fn read_intensity(
    files: &IntensityFiles,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
    amplitude_input: bool,
) -> io::Result<IntensityData> {
    let mut pwr: Option<Raster<f32>> = None;
    let mut pwr1: Option<Raster<f32>> = None;
    let mut pwr2: Option<Raster<f32>> = None;

    if let Some(ampfile2) = &files.ampfile2 {
        if files.ampfile_format != RasterFileFormat::FloatData {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "illegal file formats specified for '{}' and '{}'",
                    files.ampfile.display(),
                    ampfile2.display()
                ),
            ));
        }
        pwr1 = Some(read_2d_array::<f32>(
            &files.ampfile,
            line_len,
            nlines,
            window,
        )?);
        pwr2 = Some(read_2d_array::<f32>(ampfile2, line_len, nlines, window)?);
    } else {
        match files.ampfile_format {
            RasterFileFormat::AltSampleData => {
                let (a, b) = read_alt_samp_file(&files.ampfile, line_len, nlines, window)?;
                pwr1 = Some(a);
                pwr2 = Some(b);
            }
            RasterFileFormat::AltLineData => {
                let (a, b) = read_alt_line_file(&files.ampfile, line_len, nlines, window)?;
                pwr1 = Some(a);
                pwr2 = Some(b);
            }
            RasterFileFormat::FloatData => {
                pwr = Some(read_2d_array::<f32>(
                    &files.ampfile,
                    line_len,
                    nlines,
                    window,
                )?);
            }
        }
    }

    let check_valid = |r: &Raster<f32>| {
        valid_data_array(&r.data, r.height, r.width)
            && non_neg_data_array(&r.data, r.height, r.width)
    };
    if pwr1.as_ref().is_some_and(|r| !check_valid(r))
        || pwr2.as_ref().is_some_and(|r| !check_valid(r))
        || pwr.as_ref().is_some_and(|r| !check_valid(r))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "amplitude/power data must be finite and non-negative",
        ));
    }

    if amplitude_input {
        if let Some(r) = pwr1.as_mut() {
            for v in &mut r.data {
                *v *= *v;
            }
        }
        if let Some(r) = pwr2.as_mut() {
            for v in &mut r.data {
                *v *= *v;
            }
        }
        if let Some(r) = pwr.as_mut() {
            for v in &mut r.data {
                *v *= *v;
            }
        }
    }

    if let (Some(a), Some(b)) = (&pwr1, &pwr2) {
        let mut avg = Vec::with_capacity(a.data.len());
        for i in 0..a.data.len() {
            avg.push((a.data[i] + b.data[i]) * 0.5);
        }
        pwr = Some(Raster::new(a.width, a.height, avg));
    }

    let pwr = pwr.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "intensity read did not produce an average power raster",
        )
    })?;
    Ok(IntensityData { pwr, pwr1, pwr2 })
}

/// Read correlation from file.
///
/// This is the idiomatic Rust equivalent of the C `ReadCorrelation()`
/// function.
pub fn read_correlation(
    file: &CorrelationFile,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
) -> io::Result<Raster<f32>> {
    match file.corrfile_format {
        RasterFileFormat::AltSampleData => {
            let (_dummy, corr) = read_alt_samp_file(&file.corrfile, line_len, nlines, window)?;
            Ok(corr)
        }
        RasterFileFormat::AltLineData => {
            read_alt_line_file_phase(&file.corrfile, line_len, nlines, window)
        }
        RasterFileFormat::FloatData => {
            read_2d_array::<f32>(&file.corrfile, line_len, nlines, window)
        }
    }
}

/// Row/column arc tile extracted from a packed RowCol file.
///
/// The source file stores row arcs first (`(nlines-1) x line_len`), followed
/// by column arcs (`nlines x (line_len-1)`).
#[derive(Debug, Clone)]
pub struct RowColTile<T> {
    pub row_arcs: Raster<T>,
    pub col_arcs: Raster<T>,
}

/// Read row and column arc arrays from a packed RowCol file.
///
/// This is the idiomatic Rust equivalent of the C `Read2DRowColFile()`
/// function.
pub fn read_2d_row_col_file<T: NativeSample>(
    filename: &Path,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
) -> io::Result<RowColTile<T>> {
    if window.nrow == 0 || window.ncol == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile window must be at least 1x1",
        ));
    }
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

    let expected_elements = 2usize
        .checked_mul(line_len)
        .and_then(|v| v.checked_mul(nlines))
        .and_then(|v| v.checked_sub(nlines))
        .and_then(|v| v.checked_sub(line_len))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "expected element overflow"))?;
    let expected_size = expected_elements.checked_mul(T::SIZE).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "expected file size overflow")
    })?;

    let mut fp = File::open(filename)?;
    let filelen = fp.metadata()?.len() as usize;
    if filelen != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} wrong size ({} elements expected)",
                filename.display(),
                expected_elements
            ),
        ));
    }

    let row_arc_rows = window.nrow - 1;
    let row_arc_cols = window.ncol;
    let col_arc_rows = window.nrow;
    let col_arc_cols = window.ncol - 1;

    let row_start = (line_len
        .checked_mul(window.first_row)
        .and_then(|v| v.checked_add(window.first_col)))
    .and_then(|v| v.checked_mul(T::SIZE))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row seek offset overflow"))?;
    fp.seek(SeekFrom::Start(row_start as u64))?;

    let pad_bytes = (line_len - window.ncol)
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row padding overflow"))?;
    let pad_skip = i64::try_from(pad_bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "row padding too large to seek")
    })?;

    let row_bytes = row_arc_cols
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row read width overflow"))?;
    let mut rowbuf = vec![0u8; row_bytes];
    let mut row_data = Vec::with_capacity(row_arc_rows * row_arc_cols);
    for _ in 0..row_arc_rows {
        if row_bytes > 0 {
            fp.read_exact(&mut rowbuf)?;
            for chunk in rowbuf.chunks_exact(T::SIZE) {
                row_data.push(T::from_ne_bytes(chunk));
            }
        }
        if pad_skip > 0 {
            fp.seek(SeekFrom::Current(pad_skip))?;
        }
    }

    let col_start = (line_len
        .checked_mul(nlines - 1)
        .and_then(|v| {
            (line_len - 1)
                .checked_mul(window.first_row)
                .and_then(|k| v.checked_add(k))
        })
        .and_then(|v| v.checked_add(window.first_col)))
    .and_then(|v| v.checked_mul(T::SIZE))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "col seek offset overflow"))?;
    fp.seek(SeekFrom::Start(col_start as u64))?;

    let col_bytes = col_arc_cols
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "col read width overflow"))?;
    let mut colbuf = vec![0u8; col_bytes];
    let mut col_data = Vec::with_capacity(col_arc_rows * col_arc_cols);
    for _ in 0..col_arc_rows {
        if col_bytes > 0 {
            fp.read_exact(&mut colbuf)?;
            for chunk in colbuf.chunks_exact(T::SIZE) {
                col_data.push(T::from_ne_bytes(chunk));
            }
        }
        if pad_skip > 0 {
            fp.seek(SeekFrom::Current(pad_skip))?;
        }
    }

    Ok(RowColTile {
        row_arcs: Raster::new(row_arc_cols, row_arc_rows, row_data),
        col_arcs: Raster::new(col_arc_cols, col_arc_rows, col_data),
    })
}

/// Read only the row-arc block from a packed RowCol file.
///
/// This is the idiomatic Rust equivalent of the C `Read2DRowColFileRows()`
/// helper, where `window.nrow` is interpreted directly as the number of row
/// rows to read from the row-arc block.
pub fn read_2d_row_col_file_rows<T: NativeSample>(
    filename: &Path,
    line_len: usize,
    nlines: usize,
    window: TileWindow,
) -> io::Result<Raster<T>> {
    if window.nrow == 0 || window.ncol == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile window must be at least 1x1",
        ));
    }
    if window.first_row > nlines.saturating_sub(1)
        || window.first_col > line_len
        || window.first_row + window.nrow > nlines.saturating_sub(1)
        || window.first_col + window.ncol > line_len
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile window exceeds row-arc block bounds",
        ));
    }

    let expected_elements = 2usize
        .checked_mul(line_len)
        .and_then(|v| v.checked_mul(nlines))
        .and_then(|v| v.checked_sub(nlines))
        .and_then(|v| v.checked_sub(line_len))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "expected element overflow"))?;
    let expected_size = expected_elements.checked_mul(T::SIZE).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "expected file size overflow")
    })?;

    let mut fp = File::open(filename)?;
    let filelen = fp.metadata()?.len() as usize;
    if filelen != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "file {} wrong size ({} elements expected)",
                filename.display(),
                expected_elements
            ),
        ));
    }

    let start = (line_len
        .checked_mul(window.first_row)
        .and_then(|v| v.checked_add(window.first_col)))
    .and_then(|v| v.checked_mul(T::SIZE))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek offset overflow"))?;
    fp.seek(SeekFrom::Start(start as u64))?;

    let row_bytes = window
        .ncol
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row read width overflow"))?;
    let pad_bytes = (line_len - window.ncol)
        .checked_mul(T::SIZE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row padding overflow"))?;
    let pad_skip = i64::try_from(pad_bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "row padding too large to seek")
    })?;

    let mut rowbuf = vec![0u8; row_bytes];
    let mut row_data = Vec::with_capacity(window.nrow * window.ncol);
    for _ in 0..window.nrow {
        if row_bytes > 0 {
            fp.read_exact(&mut rowbuf)?;
            for chunk in rowbuf.chunks_exact(T::SIZE) {
                row_data.push(T::from_ne_bytes(chunk));
            }
        }
        if pad_skip > 0 {
            fp.seek(SeekFrom::Current(pad_skip))?;
        }
    }

    Ok(Raster::new(window.ncol, window.nrow, row_data))
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
    fn read_alt_samp_file_splits_interleaved_samples() {
        let path = temp_file("snaphu_rs_read_alt_samp");
        let mut fp = File::create(&path).unwrap();
        for row in 0..3u32 {
            for col in 0..4u32 {
                let a = (row * 10 + col) as f32;
                let b = (row * 100 + col) as f32;
                fp.write_all(&a.to_ne_bytes()).unwrap();
                fp.write_all(&b.to_ne_bytes()).unwrap();
            }
        }
        drop(fp);

        let (arr1, arr2) = read_alt_samp_file(&path, 4, 3, TileWindow::new(1, 1, 2, 2)).unwrap();
        assert_eq!(arr1.width, 2);
        assert_eq!(arr1.height, 2);
        assert_eq!(arr1.data, vec![11.0, 12.0, 21.0, 22.0]);
        assert_eq!(arr2.data, vec![101.0, 102.0, 201.0, 202.0]);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_samp_file_rejects_wrong_file_size() {
        let path = temp_file("snaphu_rs_read_alt_samp_badsize");
        let mut fp = File::create(&path).unwrap();
        fp.write_all(&1.0f32.to_ne_bytes()).unwrap();
        drop(fp);

        let err = read_alt_samp_file(&path, 4, 3, TileWindow::new(0, 0, 1, 1)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("wrong size"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_alt_samp_file_rejects_out_of_bounds_window() {
        let path = temp_file("snaphu_rs_read_alt_samp_bounds");
        let mut fp = File::create(&path).unwrap();
        for _ in 0..(2 * 3 * 4) {
            fp.write_all(&0.0f32.to_ne_bytes()).unwrap();
        }
        drop(fp);

        let err = read_alt_samp_file(&path, 4, 3, TileWindow::new(2, 3, 2, 2)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("tile window"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_row_col_file_reads_both_arc_blocks() {
        let path = temp_file("snaphu_rs_read_row_col");
        let mut fp = File::create(&path).unwrap();

        // Row-arc block: (nlines-1) x line_len = 3 x 5.
        for row in 0..3u32 {
            for col in 0..5u32 {
                let value = (row * 10 + col) as f32;
                fp.write_all(&value.to_ne_bytes()).unwrap();
            }
        }
        // Col-arc block: nlines x (line_len-1) = 4 x 4.
        for row in 0..4u32 {
            for col in 0..4u32 {
                let value = (1000 + row * 10 + col) as f32;
                fp.write_all(&value.to_ne_bytes()).unwrap();
            }
        }
        drop(fp);

        let tile = read_2d_row_col_file::<f32>(&path, 5, 4, TileWindow::new(1, 1, 2, 3)).unwrap();
        assert_eq!(tile.row_arcs.width, 3);
        assert_eq!(tile.row_arcs.height, 1);
        assert_eq!(tile.row_arcs.data, vec![11.0, 12.0, 13.0]);
        assert_eq!(tile.col_arcs.width, 2);
        assert_eq!(tile.col_arcs.height, 2);
        assert_eq!(tile.col_arcs.data, vec![1011.0, 1012.0, 1021.0, 1022.0]);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_row_col_file_rejects_wrong_file_size() {
        let path = temp_file("snaphu_rs_read_row_col_badsize");
        let mut fp = File::create(&path).unwrap();
        fp.write_all(&1u16.to_ne_bytes()).unwrap();
        drop(fp);

        let err =
            read_2d_row_col_file::<u16>(&path, 5, 4, TileWindow::new(0, 0, 2, 3)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("wrong size"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_row_col_file_rejects_out_of_bounds_window() {
        let path = temp_file("snaphu_rs_read_row_col_bounds");
        let mut fp = File::create(&path).unwrap();
        // Valid full-size payload for line_len=5, nlines=4:
        // 2*5*4 - 4 - 5 = 31 elements.
        for _ in 0..31 {
            fp.write_all(&0u16.to_ne_bytes()).unwrap();
        }
        drop(fp);

        let err =
            read_2d_row_col_file::<u16>(&path, 5, 4, TileWindow::new(3, 4, 2, 2)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("tile window"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_row_col_file_rows_reads_row_arc_slice_only() {
        let path = temp_file("snaphu_rs_read_row_col_rows_only");
        let mut fp = File::create(&path).unwrap();

        // Row-arc block: (nlines-1) x line_len = 3 x 5.
        for row in 0..3u32 {
            for col in 0..5u32 {
                let value = (row * 10 + col) as f32;
                fp.write_all(&value.to_ne_bytes()).unwrap();
            }
        }
        // Col-arc block data should not affect this reader.
        for row in 0..4u32 {
            for col in 0..4u32 {
                let value = (1000 + row * 10 + col) as f32;
                fp.write_all(&value.to_ne_bytes()).unwrap();
            }
        }
        drop(fp);

        let rows =
            read_2d_row_col_file_rows::<f32>(&path, 5, 4, TileWindow::new(1, 1, 2, 3)).unwrap();
        assert_eq!(rows.width, 3);
        assert_eq!(rows.height, 2);
        assert_eq!(rows.data, vec![11.0, 12.0, 13.0, 21.0, 22.0, 23.0]);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_row_col_file_rows_rejects_wrong_file_size() {
        let path = temp_file("snaphu_rs_read_row_col_rows_badsize");
        let mut fp = File::create(&path).unwrap();
        fp.write_all(&1u16.to_ne_bytes()).unwrap();
        drop(fp);

        let err =
            read_2d_row_col_file_rows::<u16>(&path, 5, 4, TileWindow::new(0, 0, 1, 1)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("wrong size"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_2d_row_col_file_rows_rejects_out_of_bounds_window() {
        let path = temp_file("snaphu_rs_read_row_col_rows_bounds");
        let mut fp = File::create(&path).unwrap();
        for _ in 0..31 {
            fp.write_all(&0u16.to_ne_bytes()).unwrap();
        }
        drop(fp);

        let err =
            read_2d_row_col_file_rows::<u16>(&path, 5, 4, TileWindow::new(3, 0, 1, 1)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("row-arc block bounds"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_intensity_from_single_float_file() {
        let path = temp_file("snaphu_rs_read_intensity_float");
        let mut fp = File::create(&path).unwrap();
        for value in 1..=12u32 {
            fp.write_all(&(value as f32).to_ne_bytes()).unwrap();
        }
        drop(fp);

        let files = IntensityFiles {
            ampfile: path.clone(),
            ampfile2: None,
            ampfile_format: RasterFileFormat::FloatData,
        };
        let intensity = read_intensity(&files, 4, 3, TileWindow::new(0, 0, 3, 4), false).unwrap();
        assert_eq!(intensity.pwr.data.len(), 12);
        assert!(intensity.pwr1.is_none());
        assert!(intensity.pwr2.is_none());

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn read_intensity_two_files_with_amplitude_squares_and_averages() {
        let p1 = temp_file("snaphu_rs_read_intensity_a");
        let p2 = temp_file("snaphu_rs_read_intensity_b");
        let mut f1 = File::create(&p1).unwrap();
        let mut f2 = File::create(&p2).unwrap();
        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [2.0f32, 3.0, 4.0, 5.0];
        for v in a {
            f1.write_all(&v.to_ne_bytes()).unwrap();
        }
        for v in b {
            f2.write_all(&v.to_ne_bytes()).unwrap();
        }
        drop(f1);
        drop(f2);

        let files = IntensityFiles {
            ampfile: p1.clone(),
            ampfile2: Some(p2.clone()),
            ampfile_format: RasterFileFormat::FloatData,
        };
        let out = read_intensity(&files, 2, 2, TileWindow::new(0, 0, 2, 2), true).unwrap();
        // (a^2+b^2)/2
        assert_eq!(out.pwr.data, vec![2.5, 6.5, 12.5, 20.5]);
        assert!(out.pwr1.is_some());
        assert!(out.pwr2.is_some());

        fs::remove_file(p1).unwrap();
        fs::remove_file(p2).unwrap();
    }

    #[test]
    fn read_correlation_from_alt_sample_uses_second_channel() {
        let path = temp_file("snaphu_rs_read_corr_altsamp");
        let mut fp = File::create(&path).unwrap();
        // 2 lines x 2 cols -> 8 f32 samples interleaved
        // channel A: 10,11,12,13 ; channel B: 20,21,22,23
        let samples = [10.0f32, 20.0, 11.0, 21.0, 12.0, 22.0, 13.0, 23.0];
        for v in samples {
            fp.write_all(&v.to_ne_bytes()).unwrap();
        }
        drop(fp);

        let corr = read_correlation(
            &CorrelationFile {
                corrfile: path.clone(),
                corrfile_format: RasterFileFormat::AltSampleData,
            },
            2,
            2,
            TileWindow::new(0, 0, 2, 2),
        )
        .unwrap();
        assert_eq!(corr.data, vec![20.0, 21.0, 22.0, 23.0]);

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
