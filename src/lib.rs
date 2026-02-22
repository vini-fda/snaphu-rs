//! Public API surface for the snaphu-rs crate.
//!
//! The real translation work will progressively replace the temporary
//! `legacy-cli` feature, which still shells out to the original C pipeline
//! through the auto-generated `snaphu_full` module.

#[cfg(feature = "legacy-cli")]
extern crate libc;

#[cfg(feature = "legacy-cli")]
mod snaphu_full;

pub mod cli;
pub mod config;
pub mod constants;
pub mod context;
pub mod costs;
pub mod data;
pub mod io;
pub mod network;
pub mod unwrapping;

/// Run the SNAPHU CLI.
#[cfg(feature = "legacy-cli")]
pub fn run_cli<I, S>(args: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int};

    let cstrings: Vec<CString> = args
        .into_iter()
        .map(|s| CString::new(s.as_ref()))
        .collect::<Result<_, _>>()
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "argument contains interior NUL byte",
            )
        })?;

    let mut argv: Vec<*mut c_char> = cstrings.iter().map(|s| s.as_ptr() as *mut c_char).collect();
    let argc = argv.len() as c_int;

    let exit_code: i32 = unsafe { snaphu_sys::run_main(argc, argv.as_mut_ptr()) };
    if exit_code == 0 {
        Ok(())
    } else {
        Err(std::io::Error::from_raw_os_error(exit_code))
    }
}

/// Placeholder while the idiomatic Rust translation is under construction.
#[cfg(not(feature = "legacy-cli"))]
pub fn run_cli<I, S>(_args: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use crate::cli::{ProcessArgsError, process_args};
    use crate::config::{InputFiles, OutputFiles, check_params};
    use crate::data::ops::integrate_phase;
    use crate::data::raster::Raster;
    use crate::io::reader::{
        CorrelationFile, EdgeMaskParams, InputFileFormat, InputReadSpec, IntensityFiles,
        MagnitudeFileFormat, RasterFileFormat, TileWindow, get_n_lines, read_byte_mask,
        read_correlation, read_input_file, read_intensity, read_magnitude,
    };
    use crate::io::writer::{OutputFileFormat, write_output_file};
    use crate::unwrapping::flow::{UnwrapTileParams, unwrap_tile};
    use std::io;
    use std::path::{Path, PathBuf};

    fn usage() -> &'static str {
        "usage: snaphu [options] infile linelen\n\
         native Rust path currently supports single-tile runs"
    }

    fn to_grid_f32(r: &Raster<f32>) -> Vec<Vec<f32>> {
        r.data
            .chunks_exact(r.width)
            .map(|row| row.to_vec())
            .collect::<Vec<_>>()
    }

    fn row_col_widths(nrow: usize, ncol: usize) -> Vec<usize> {
        (0..(2 * nrow - 1))
            .map(|row| if row < nrow - 1 { ncol } else { ncol - 1 })
            .collect()
    }

    fn flat_to_row_col_flows(flows: &[i16], nrow: usize, ncol: usize) -> io::Result<Vec<Vec<i16>>> {
        let widths = row_col_widths(nrow, ncol);
        let expected = widths.iter().sum::<usize>();
        if flows.len() != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "flow array length {} does not match expected {}",
                    flows.len(),
                    expected
                ),
            ));
        }
        let mut out = Vec::with_capacity(widths.len());
        let mut cursor = 0usize;
        for width in widths {
            out.push(flows[cursor..cursor + width].to_vec());
            cursor += width;
        }
        Ok(out)
    }

    fn row_col_to_flat_flows(flows: &[Vec<i16>], nrow: usize, ncol: usize) -> io::Result<Vec<i16>> {
        let widths = row_col_widths(nrow, ncol);
        if flows.len() != widths.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "row/col flow row count mismatch",
            ));
        }
        let mut out = Vec::with_capacity(widths.iter().sum());
        for (row, &width) in widths.iter().enumerate() {
            if flows[row].len() != width {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "row/col flow width mismatch at row {row}: got {}, expected {width}",
                        flows[row].len()
                    ),
                ));
            }
            out.extend_from_slice(&flows[row]);
        }
        Ok(out)
    }

    let raw_args = _args
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect::<Vec<_>>();

    if raw_args.len() <= 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, usage()));
    }

    if raw_args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{}", usage());
        return Ok(());
    }
    if raw_args
        .iter()
        .any(|arg| arg == "--copyright" || arg == "--info")
    {
        println!("snaphu-rs native Rust path (work in progress)");
        return Ok(());
    }

    let mut infiles = InputFiles::default();
    let mut outfiles = OutputFiles::default();
    let mut linelen = 0usize;
    let mut params = crate::config::RunConfig::default();

    process_args(
        &raw_args,
        &mut infiles,
        &mut outfiles,
        &mut linelen,
        &mut params,
    )
    .map_err(|err| match err {
        ProcessArgsError::NoArguments | ProcessArgsError::NotEnoughPositionalArgs => {
            io::Error::new(io::ErrorKind::InvalidInput, usage())
        }
        other => io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("argument error: {other:?}"),
        ),
    })?;

    if params.ntilerow != 1 || params.ntilecol != 1 || params.onetilereopt || params.assemble_only {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native Rust CLI currently supports single-tile runs only",
        ));
    }
    if params.eval {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native Rust CLI quantify-only mode is not implemented yet",
        ));
    }

    let infile_path = Path::new(&infiles.infile);
    let infile_format = InputFileFormat::ComplexData;
    let unwrapped_line_format = InputFileFormat::AltLineData;
    let nlines_full = get_n_lines(
        infile_path,
        linelen,
        params.unwrapped,
        infile_format,
        unwrapped_line_format,
    )?;

    let window = if params.piecenrow > 0 && params.piecencol > 0 {
        let first_row = params.piecefirstrow.checked_sub(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "piece first row is 1-based")
        })?;
        let first_col = params.piecefirstcol.checked_sub(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "piece first col is 1-based")
        })?;
        TileWindow::new(first_row, first_col, params.piecenrow, params.piecencol)
    } else {
        TileWindow::new(0, 0, nlines_full, linelen)
    };

    check_params(&infiles, &outfiles, window.ncol, window.nrow, &params).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid parameters: {err:?}"),
        )
    })?;

    let mut input = read_input_file(
        &InputReadSpec {
            infile: infile_path.to_path_buf(),
            unwrapped: params.unwrapped,
            infile_format,
            unwrapped_infile_format: RasterFileFormat::AltLineData,
            flip_phase_sign: false,
        },
        linelen,
        nlines_full,
        window,
    )?;

    if !infiles.magfile.is_empty() {
        read_magnitude(
            &mut input.mag,
            Some(Path::new(&infiles.magfile)),
            MagnitudeFileFormat::FloatData,
            linelen,
            nlines_full,
            window,
        )?;
    }

    if !infiles.bytemaskfile.is_empty() {
        read_byte_mask(
            &mut input.mag,
            Some(Path::new(&infiles.bytemaskfile)),
            linelen,
            nlines_full,
            window,
            EdgeMaskParams {
                top: 0,
                bottom: 0,
                left: 0,
                right: 0,
            },
        )?;
    }

    let power_grid = if !infiles.ampfile.is_empty() {
        let intensity = read_intensity(
            &IntensityFiles {
                ampfile: PathBuf::from(&infiles.ampfile),
                ampfile2: if infiles.ampfile2.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&infiles.ampfile2))
                },
                ampfile_format: RasterFileFormat::FloatData,
            },
            linelen,
            nlines_full,
            window,
            params.amplitude,
        )?;
        Some(to_grid_f32(&intensity.pwr))
    } else {
        None
    };

    let corr_grid = if !infiles.corrfile.is_empty() {
        let corr = read_correlation(
            &CorrelationFile {
                corrfile: PathBuf::from(&infiles.corrfile),
                corrfile_format: RasterFileFormat::FloatData,
            },
            linelen,
            nlines_full,
            window,
        )?;
        Some(to_grid_f32(&corr))
    } else {
        None
    };

    let mag_grid = to_grid_f32(&input.mag);
    let wrapped_grid = to_grid_f32(&input.wrapped_phase);
    let initial_flows = input
        .flows
        .as_deref()
        .map(|flows| flat_to_row_col_flows(flows, window.nrow, window.ncol))
        .transpose()?;

    let out = unwrap_tile(
        UnwrapTileParams {
            mag: &mag_grid,
            wrapped_phase: &wrapped_grid,
            power: power_grid.as_deref(),
            correlation: corr_grid.as_deref(),
            initial_flows,
            cost_threshold: 0,
            min_region_size: 1,
            max_components: window.nrow.saturating_mul(window.ncol).max(1),
        },
        &params,
    )
    .map_err(|err| io::Error::other(format!("native unwrap failed: {err:?}")))?;

    let flat_flows = row_col_to_flat_flows(&out.flows, window.nrow, window.ncol)?;
    let unwrapped_phase = integrate_phase(
        &input.wrapped_phase.data,
        &flat_flows,
        window.nrow,
        window.ncol,
    );
    let unwrapped = Raster::new(window.ncol, window.nrow, unwrapped_phase);

    let _written = write_output_file(
        &input.mag,
        &unwrapped,
        Path::new(&outfiles.outfile),
        OutputFileFormat::AltLineData,
    )?;

    if params.verbose {
        eprintln!(
            "native Rust CLI wrote {}x{} output to {}",
            window.nrow, window.ncol, outfiles.outfile
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "legacy-cli"))]
    use super::run_cli;
    #[cfg(not(feature = "legacy-cli"))]
    use std::fs;
    #[cfg(not(feature = "legacy-cli"))]
    fn tmp_path(name: &str) -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!("snaphu_rs_{name}_{}_{}", std::process::id(), stamp))
    }

    #[cfg(not(feature = "legacy-cli"))]
    #[test]
    fn run_cli_native_help_returns_ok() {
        assert!(run_cli(["snaphu", "--help"]).is_ok());
    }

    #[cfg(not(feature = "legacy-cli"))]
    #[test]
    fn run_cli_native_single_tile_smoke() {
        let input = tmp_path("in");
        let output = tmp_path("out");

        // 2x2 complex raster: each sample is (re, im) f32 in native endian.
        let samples = [
            1.0f32, 0.0, 0.8, 0.2, //
            1.1, 0.1, 0.9, -0.2,
        ];
        let mut raw = Vec::with_capacity(samples.len() * std::mem::size_of::<f32>());
        for s in samples {
            raw.extend_from_slice(&s.to_ne_bytes());
        }
        fs::write(&input, raw).expect("failed to write temporary complex input");

        run_cli([
            "snaphu",
            input.to_str().expect("input path must be UTF-8"),
            "2",
            "-o",
            output.to_str().expect("output path must be UTF-8"),
        ])
        .expect("native run_cli should succeed");

        let out_meta = fs::metadata(&output).expect("output file must exist");
        assert!(out_meta.len() > 0);

        let _ = fs::remove_file(input);
        let _ = fs::remove_file(output);
    }
}
