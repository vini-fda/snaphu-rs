#![allow(dead_code)]

//! Output helpers for unwrapped phase products.

use crate::costs::types::IncrCost;
use crate::data::raster::Raster;
use crate::io::reader::parse_filename;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const DEFAULT_DUMP_PATH: &str = "/tmp/";
const DUMP_INITFILE: &str = "snaphu.init";
const DUMP_FLOWFILE: &str = "snaphu.flow";
const DUMP_EIFILE: &str = "snaphu.ei";
const DUMP_ROWCOSTFILE: &str = "snaphu.rowcost";
const DUMP_COLCOSTFILE: &str = "snaphu.colcost";
const DUMP_MSTROWCOSTFILE: &str = "snaphu.mstrowcost";
const DUMP_MSTCOLCOSTFILE: &str = "snaphu.mstcolcost";
const DUMP_MSTCOSTSFILE: &str = "snaphu.mstcosts";
const DUMP_CORRDUMPFILE: &str = "snaphu.corr";
const DUMP_RAWCORRDUMPFILE: &str = "snaphu.rawcorr";
const INCRCOSTFILEPOS: &str = "snaphu.incrcostpos";
const INCRCOSTFILENEG: &str = "snaphu.incrcostneg";

#[derive(Debug)]
pub struct OpenedOutputFile {
    pub file: File,
    pub real_path: PathBuf,
    pub fell_back: bool,
}

/// Output format selector for final unwrapped products.
///
/// Mirrors C `outfileformat` values used by `WriteOutputFile()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFileFormat {
    AltLineData,
    AltSampleData,
    FloatData,
    Unknown,
}

/// Collection of optional intermediate dump-file names.
///
/// Mirrors the fields that C `SetDumpAll()` populates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DumpOutputFiles {
    pub initfile: Option<PathBuf>,
    pub flowfile: Option<PathBuf>,
    pub eifile: Option<PathBuf>,
    pub rowcostfile: Option<PathBuf>,
    pub colcostfile: Option<PathBuf>,
    pub mstrowcostfile: Option<PathBuf>,
    pub mstcolcostfile: Option<PathBuf>,
    pub mstcostsfile: Option<PathBuf>,
    pub corrdumpfile: Option<PathBuf>,
    pub rawcorrdumpfile: Option<PathBuf>,
}

/// Populate dump-file names when debug dumping is enabled.
///
/// This is the idiomatic Rust equivalent of C `SetDumpAll()`.
pub fn set_dump_all(outfiles: &mut DumpOutputFiles, dumpall: bool) {
    if !dumpall {
        return;
    }
    if outfiles.initfile.is_none() {
        outfiles.initfile = Some(PathBuf::from(DUMP_INITFILE));
    }
    if outfiles.flowfile.is_none() {
        outfiles.flowfile = Some(PathBuf::from(DUMP_FLOWFILE));
    }
    if outfiles.eifile.is_none() {
        outfiles.eifile = Some(PathBuf::from(DUMP_EIFILE));
    }
    if outfiles.rowcostfile.is_none() {
        outfiles.rowcostfile = Some(PathBuf::from(DUMP_ROWCOSTFILE));
    }
    if outfiles.colcostfile.is_none() {
        outfiles.colcostfile = Some(PathBuf::from(DUMP_COLCOSTFILE));
    }
    if outfiles.mstrowcostfile.is_none() {
        outfiles.mstrowcostfile = Some(PathBuf::from(DUMP_MSTROWCOSTFILE));
    }
    if outfiles.mstcolcostfile.is_none() {
        outfiles.mstcolcostfile = Some(PathBuf::from(DUMP_MSTCOLCOSTFILE));
    }
    if outfiles.mstcostsfile.is_none() {
        outfiles.mstcostsfile = Some(PathBuf::from(DUMP_MSTCOSTSFILE));
    }
    if outfiles.corrdumpfile.is_none() {
        outfiles.corrdumpfile = Some(PathBuf::from(DUMP_CORRDUMPFILE));
    }
    if outfiles.rawcorrdumpfile.is_none() {
        outfiles.rawcorrdumpfile = Some(PathBuf::from(DUMP_RAWCORRDUMPFILE));
    }
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

/// Write the final unwrapped output in the requested encoding.
///
/// This is the idiomatic Rust equivalent of C `WriteOutputFile()`.
pub fn write_output_file(
    mag: &Raster<f32>,
    unwrapped_phase: &Raster<f32>,
    outfile: &Path,
    outfile_format: OutputFileFormat,
) -> io::Result<PathBuf> {
    match outfile_format {
        OutputFileFormat::AltLineData => write_alt_line_file(mag, unwrapped_phase, outfile),
        OutputFileFormat::AltSampleData => write_alt_samp_file(mag, unwrapped_phase, outfile),
        OutputFileFormat::FloatData => write_2d_array(
            &unwrapped_phase.data,
            unwrapped_phase.height,
            unwrapped_phase.width,
            outfile,
        ),
        OutputFileFormat::Unknown => {
            log::warn!("Illegal format specified for output file; using default float format");
            write_2d_array(
                &unwrapped_phase.data,
                unwrapped_phase.height,
                unwrapped_phase.width,
                outfile,
            )
        }
    }
}

/// Output filenames produced by [`dump_incr_cost_files`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrCostDumpPaths {
    pub poscost_file: PathBuf,
    pub negcost_file: PathBuf,
}

/// Dump positive/negative incremental arc costs into two row/col-packed files.
///
/// This is the idiomatic Rust equivalent of C `DumpIncrCostFiles()`.
pub fn dump_incr_cost_files(
    incrcosts: &[Vec<IncrCost>],
    iincrcostfile: i64,
    nflow: i64,
    nrow: usize,
    ncol: usize,
    output_dir: Option<&Path>,
) -> io::Result<IncrCostDumpPaths> {
    if nrow < 1 || ncol < 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "nrow and ncol must be at least 1",
        ));
    }
    let expected_rows = 2 * nrow - 1;
    if incrcosts.len() != expected_rows {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "incrcost row count does not match row/col arc layout",
        ));
    }

    let mut pos_row = Vec::with_capacity((nrow - 1) * ncol);
    let mut pos_col = Vec::with_capacity(nrow * (ncol - 1));
    let mut neg_row = Vec::with_capacity((nrow - 1) * ncol);
    let mut neg_col = Vec::with_capacity(nrow * (ncol - 1));

    for (arcrow, row) in incrcosts.iter().enumerate() {
        let maxcol = if arcrow < nrow - 1 { ncol } else { ncol - 1 };
        if row.len() != maxcol {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "incrcost row {} has {} columns; expected {}",
                    arcrow,
                    row.len(),
                    maxcol
                ),
            ));
        }
        for cost in row {
            if arcrow < nrow - 1 {
                pos_row.push(cost.poscost);
                neg_row.push(cost.negcost);
            } else {
                pos_col.push(cost.poscost);
                neg_col.push(cost.negcost);
            }
        }
    }

    let pos_name = format!("{INCRCOSTFILEPOS}.{iincrcostfile}_{nflow}");
    let neg_name = format!("{INCRCOSTFILENEG}.{iincrcostfile}_{nflow}");
    let pos_path = output_dir
        .map(|dir| dir.join(&pos_name))
        .unwrap_or_else(|| PathBuf::from(&pos_name));
    let neg_path = output_dir
        .map(|dir| dir.join(&neg_name))
        .unwrap_or_else(|| PathBuf::from(&neg_name));

    let pos_written = write_2d_row_col_array(
        &Raster::new(ncol, nrow - 1, pos_row),
        &Raster::new(ncol - 1, nrow, pos_col),
        &pos_path,
    )?;
    let neg_written = write_2d_row_col_array(
        &Raster::new(ncol, nrow - 1, neg_row),
        &Raster::new(ncol - 1, nrow, neg_col),
        &neg_path,
    )?;

    Ok(IncrCostDumpPaths {
        poscost_file: pos_written,
        negcost_file: neg_written,
    })
}

/// File-format values accepted by C `LogFileFormat()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoggedFileFormat {
    ComplexData,
    FloatData,
    AltLineData,
    AltSampleData,
}

/// Write one key/value line for a string parameter.
///
/// This is the idiomatic Rust equivalent of C `LogStringParam()`.
/// Empty values are logged as a comment line.
pub fn log_string_param<W: Write>(writer: &mut W, key: &str, value: &str) -> io::Result<()> {
    if value.is_empty() {
        writeln!(writer, "# Empty value for parameter {key}")?;
    } else {
        writeln!(writer, "{key}  {value}")?;
        writer.flush()?;
    }
    Ok(())
}

/// Write one key/value line for a boolean parameter.
///
/// This is the idiomatic Rust equivalent of C `LogBoolParam()`.
pub fn log_bool_param<W: Write>(writer: &mut W, key: &str, bool_value: bool) -> io::Result<()> {
    let rendered = if bool_value { "TRUE" } else { "FALSE" };
    writeln!(writer, "{key}  {rendered}")?;
    Ok(())
}

/// Write one key/value line for an on-disk file format.
///
/// This is the idiomatic Rust equivalent of C `LogFileFormat()`.
pub fn log_file_format<W: Write>(
    writer: &mut W,
    key: &str,
    file_format: LoggedFileFormat,
) -> io::Result<()> {
    let rendered = match file_format {
        LoggedFileFormat::ComplexData => "COMPLEX_DATA",
        LoggedFileFormat::FloatData => "FLOAT_DATA",
        LoggedFileFormat::AltLineData => "ALT_LINE_DATA",
        LoggedFileFormat::AltSampleData => "ALT_SAMPLE_DATA",
    };
    writeln!(writer, "{key}  {rendered}")?;
    Ok(())
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

/// Write row/column arc arrays to the packed RowCol on-disk layout.
///
/// Equivalent of the C `Write2DRowColArray()` helper:
/// - row-arc block first: `(nrow-1) x ncol`
/// - col-arc block second: `nrow x (ncol-1)`
pub fn write_2d_row_col_array<T: NativeWritable>(
    row_arcs: &Raster<T>,
    col_arcs: &Raster<T>,
    filename: &Path,
) -> io::Result<PathBuf> {
    if row_arcs.height + 1 != col_arcs.height || row_arcs.width != col_arcs.width + 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "row/col arc dimensions are inconsistent",
        ));
    }

    let expected_row = row_arcs
        .width
        .checked_mul(row_arcs.height)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "row-arc size overflow"))?;
    if row_arcs.data.len() != expected_row {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "row_arcs data length does not match width*height",
        ));
    }

    let expected_col = col_arcs
        .width
        .checked_mul(col_arcs.height)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "col-arc size overflow"))?;
    if col_arcs.data.len() != expected_col {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "col_arcs data length does not match width*height",
        ));
    }

    let mut opened = open_output_file(filename)?;
    for row in 0..row_arcs.height {
        let start = row * row_arcs.width;
        let end = start + row_arcs.width;
        for sample in &row_arcs.data[start..end] {
            sample.write_ne(&mut opened.file)?;
        }
    }

    for row in 0..col_arcs.height {
        let start = row * col_arcs.width;
        let end = start + col_arcs.width;
        for sample in &col_arcs.data[start..end] {
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
    use crate::io::reader::{
        TileWindow, read_2d_array, read_2d_row_col_file, read_alt_line_file, read_alt_samp_file,
    };
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
    fn set_dump_all_sets_defaults_only_when_enabled() {
        let mut out = DumpOutputFiles::default();
        set_dump_all(&mut out, false);
        assert!(out.initfile.is_none());

        set_dump_all(&mut out, true);
        assert_eq!(out.initfile, Some(PathBuf::from("snaphu.init")));
        assert_eq!(out.flowfile, Some(PathBuf::from("snaphu.flow")));
        assert_eq!(out.rawcorrdumpfile, Some(PathBuf::from("snaphu.rawcorr")));
    }

    #[test]
    fn set_dump_all_preserves_existing_names() {
        let mut out = DumpOutputFiles {
            flowfile: Some(PathBuf::from("custom.flow")),
            ..DumpOutputFiles::default()
        };
        set_dump_all(&mut out, true);
        assert_eq!(out.flowfile, Some(PathBuf::from("custom.flow")));
    }

    #[test]
    fn log_string_param_writes_value_or_empty_comment() {
        let mut buf = Vec::<u8>::new();
        log_string_param(&mut buf, "OUTFILE", "snaphu.out").unwrap();
        log_string_param(&mut buf, "LOGFILE", "").unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(
            text,
            "OUTFILE  snaphu.out\n# Empty value for parameter LOGFILE\n"
        );
    }

    #[test]
    fn log_bool_param_writes_true_false_literals() {
        let mut buf = Vec::<u8>::new();
        log_bool_param(&mut buf, "VERBOSE", true).unwrap();
        log_bool_param(&mut buf, "INITONLY", false).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(text, "VERBOSE  TRUE\nINITONLY  FALSE\n");
    }

    #[test]
    fn log_file_format_writes_expected_symbolic_name() {
        let mut buf = Vec::<u8>::new();
        log_file_format(&mut buf, "INFILEFORMAT", LoggedFileFormat::ComplexData).unwrap();
        log_file_format(&mut buf, "OUTFILEFORMAT", LoggedFileFormat::AltLineData).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(
            text,
            "INFILEFORMAT  COMPLEX_DATA\nOUTFILEFORMAT  ALT_LINE_DATA\n"
        );
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

    #[test]
    fn write_2d_row_col_array_round_trips_row_and_col_blocks() {
        let path =
            std::env::temp_dir().join(format!("snaphu_rs_write_rowcol_{}.bin", unique_suffix()));
        let row_arcs = Raster::new(3, 2, vec![1i16, 2, 3, 4, 5, 6]);
        let col_arcs = Raster::new(2, 3, vec![10i16, 11, 12, 13, 14, 15]);

        let written = write_2d_row_col_array(&row_arcs, &col_arcs, &path).unwrap();
        let tile =
            read_2d_row_col_file::<i16>(&written, 3, 3, TileWindow::new(0, 0, 3, 3)).unwrap();
        assert_eq!(tile.row_arcs.data, row_arcs.data);
        assert_eq!(tile.col_arcs.data, col_arcs.data);

        fs::remove_file(written).unwrap();
    }

    #[test]
    fn write_output_file_routes_by_format() {
        let mag = Raster::new(2, 2, vec![1.0f32, 2.0, 3.0, 4.0]);
        let phase = Raster::new(2, 2, vec![10.0f32, 20.0, 30.0, 40.0]);

        let path1 =
            std::env::temp_dir().join(format!("snaphu_write_output_{}.f32", unique_suffix()));
        let p1 = write_output_file(&mag, &phase, &path1, OutputFileFormat::FloatData).unwrap();
        let round = read_2d_array::<f32>(&p1, 2, 2, TileWindow::new(0, 0, 2, 2)).unwrap();
        assert_eq!(round.data, phase.data);
        fs::remove_file(p1).unwrap();

        let path2 =
            std::env::temp_dir().join(format!("snaphu_write_output_{}.alt", unique_suffix()));
        let p2 = write_output_file(&mag, &phase, &path2, OutputFileFormat::AltLineData).unwrap();
        let (rm, rp) = read_alt_line_file(&p2, 2, 2, TileWindow::new(0, 0, 2, 2)).unwrap();
        assert_eq!(rm.data, mag.data);
        assert_eq!(rp.data, phase.data);
        fs::remove_file(p2).unwrap();
    }

    #[test]
    fn write_2d_row_col_array_rejects_inconsistent_dimensions() {
        let path = std::env::temp_dir().join(format!(
            "snaphu_rs_write_rowcol_dims_{}.bin",
            unique_suffix()
        ));
        let row_arcs = Raster::new(4, 2, vec![0i16; 8]);
        let col_arcs = Raster::new(2, 3, vec![0i16; 6]);

        let err = write_2d_row_col_array(&row_arcs, &col_arcs, &path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("inconsistent"));
    }

    #[test]
    fn dump_incr_cost_files_writes_pos_and_neg_rowcol_files() {
        let dir = std::env::temp_dir().join(format!("snaphu_dump_incr_{}", unique_suffix()));
        fs::create_dir_all(&dir).unwrap();

        let incr = vec![
            vec![
                IncrCost::new(10, -10),
                IncrCost::new(11, -11),
                IncrCost::new(12, -12),
            ],
            vec![IncrCost::new(20, -20), IncrCost::new(21, -21)],
            vec![IncrCost::new(30, -30), IncrCost::new(31, -31)],
        ];

        let out = dump_incr_cost_files(&incr, 7, 3, 2, 3, Some(&dir)).unwrap();
        assert!(out.poscost_file.exists());
        assert!(out.negcost_file.exists());

        let window = TileWindow::new(0, 0, 2, 3);
        let pos = read_2d_row_col_file::<i16>(&out.poscost_file, 3, 2, window).unwrap();
        let neg = read_2d_row_col_file::<i16>(&out.negcost_file, 3, 2, window).unwrap();

        assert_eq!(pos.row_arcs.data, vec![10, 11, 12]);
        assert_eq!(pos.col_arcs.data, vec![20, 21, 30, 31]);
        assert_eq!(neg.row_arcs.data, vec![-10, -11, -12]);
        assert_eq!(neg.col_arcs.data, vec![-20, -21, -30, -31]);

        fs::remove_file(&out.poscost_file).unwrap();
        fs::remove_file(&out.negcost_file).unwrap();
        fs::remove_dir_all(dir).unwrap();
    }
}
