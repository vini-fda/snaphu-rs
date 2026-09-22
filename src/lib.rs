//! Public API surface for the snaphu-rs crate.

pub mod api;
pub mod cli;
pub mod config;
pub mod constants;
pub mod context;
pub mod costs;
pub mod data;
pub mod io;
pub mod network;
pub mod phase_compare;
pub mod unwrapping;

pub use api::{
    SnaphuError, UnwrapInputs, UnwrapOutputs, UnwrapReport, arc_count, col_arc_count,
    row_arc_count, run_snaphu, run_snaphu_inplace,
};
pub use config::{CostMode, FileFormat, InitMethod, RunConfig, TransmitMode};

/// Run the SNAPHU CLI.
pub fn run_cli<I, S>(args: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use crate::api::{UnwrapInputs, run_snaphu};
    use crate::cli::{ProcessArgsError, process_args};
    use crate::config::{FileFormat, InputFiles, OutputFiles, check_params};
    use crate::costs::CostArrayData;
    use crate::data::raster::Raster;
    use crate::io::phase_format::read_phase_header;
    use crate::io::reader::{
        CorrelationFile, EdgeMaskParams, InputFileFormat, InputReadSpec, IntensityFiles,
        MagnitudeFileFormat, RasterFileFormat, TileWindow, get_n_lines, read_byte_mask,
        read_correlation, read_input_file, read_intensity, read_magnitude,
        read_unwrapped_estimate_file, read_weights_file, set_up_do_tile_mask,
    };
    use crate::io::writer::{
        NativeWritable, OutputFileFormat, write_2d_array, write_2d_row_col_array, write_output_file,
    };
    use std::io;
    use std::path::{Path, PathBuf};

    fn to_input_file_format(f: FileFormat) -> InputFileFormat {
        match f {
            FileFormat::ComplexData => InputFileFormat::ComplexData,
            FileFormat::FloatData => InputFileFormat::FloatData,
            FileFormat::AltSampleData => InputFileFormat::AltSampleData,
            FileFormat::AltLineData => InputFileFormat::AltLineData,
            FileFormat::FloatDataPhase => InputFileFormat::FloatDataPhase,
        }
    }

    fn to_raster_file_format(f: FileFormat) -> RasterFileFormat {
        match f {
            FileFormat::FloatData | FileFormat::ComplexData => RasterFileFormat::FloatData,
            FileFormat::AltSampleData => RasterFileFormat::AltSampleData,
            FileFormat::AltLineData => RasterFileFormat::AltLineData,
            FileFormat::FloatDataPhase => RasterFileFormat::FloatDataPhase,
        }
    }

    fn to_output_file_format(f: FileFormat) -> OutputFileFormat {
        match f {
            FileFormat::FloatData | FileFormat::ComplexData => OutputFileFormat::FloatData,
            FileFormat::AltSampleData => OutputFileFormat::AltSampleData,
            FileFormat::AltLineData => OutputFileFormat::AltLineData,
            FileFormat::FloatDataPhase => OutputFileFormat::FloatDataPhase,
        }
    }

    const FULL_HELP: &str = "\
snaphu v2.0.7
usage:  snaphu [options] infile linelength [options]
        (linelength may be omitted when the input is in the snaphu-rs
         FLOAT_DATA_PHASE_FORMAT, which stores its own dimensions)
options:
  -t              use topography mode costs (default)
  -d              use deformation mode costs
  -s              use smooth-solution mode costs
  -C <confstr>    parse argument string as config line as from conf file
  -f <filename>   read configuration parameters from file
  -o <filename>   write output to file
  -a <filename>   read amplitude data from file
  -A <filename>   read power data from file
  -m <filename>   read interferogram magnitude data from file
  -M <filename>   read byte mask data from file
  -c <filename>   read correlation data from file
  -e <filename>   read coarse unwrapped-phase estimate from file
  -w <filename>   read scalar weights from file
  -b <decimal>    perpendicular baseline (meters, topo mode only)
  -p <decimal>    Lp-norm parameter p
  -i              do initialization and exit
  -n              do not use statistical costs (with -p or -i)
  -u              infile is already unwrapped; initialization not needed
  -q              quantify cost of unwrapped input file then exit
  -g <filename>   grow connected components mask and write to file
  -G <filename>   grow connected components mask for unwrapped input
  -S              single-tile reoptimization after multi-tile init
  -k              keep temporary tile outputs
  -l <filename>   log runtime parameters to file
  -v              give verbose output
  --mst           use MST algorithm for initialization (default)
  --mcf           use MCF algorithm for initialization
  --aa <filename1> <filename2>    read amplitude from next two files
  --AA <filename1> <filename2>    read power from next two files
  --costinfile <filename>         read statistical costs from file
  --costoutfile <filename>        write statistical costs to file
  --tile <nrow> <ncol> <rowovrlp> <colovrlp>  unwrap as nrow x ncol tiles
  --nproc <integer>               number of processors used in tile mode
                                   (secondary config keys: SCNDRYARCFLOWMAX,
                                    TILEEDGEWEIGHT, MAXCYCLEFRACTION)
  --tiledir <dirname>             use specified directory for tiles
  --assemble                      assemble unwrapped tiles in tiledir
  --piece <firstrow> <firstcol> <nrow> <ncol>  unwrap subset of image
  --debug, --dumpall              dump all intermediate data arrays
  --copyright, --info             print copyright and bug report info
  -h, --help                      print this help text";

    fn row_col_widths(nrow: usize, ncol: usize) -> Vec<usize> {
        (0..(2 * nrow - 1))
            .map(|row| if row < nrow - 1 { ncol } else { ncol - 1 })
            .collect()
    }

    fn row_col_to_rasters<T: Clone>(
        values: &[Vec<T>],
        nrow: usize,
        ncol: usize,
    ) -> io::Result<(Raster<T>, Raster<T>)> {
        let widths = row_col_widths(nrow, ncol);
        if values.len() != widths.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "row/col row count mismatch",
            ));
        }

        let mut row_data = Vec::with_capacity((nrow - 1) * ncol);
        let mut col_data = Vec::with_capacity(nrow * (ncol - 1));
        for (row, &width) in widths.iter().enumerate() {
            if values[row].len() != width {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "row/col width mismatch at row {row}: got {}, expected {width}",
                        values[row].len()
                    ),
                ));
            }
            if row < nrow - 1 {
                row_data.extend_from_slice(&values[row]);
            } else {
                col_data.extend_from_slice(&values[row]);
            }
        }
        Ok((
            Raster::new(ncol, nrow - 1, row_data),
            Raster::new(ncol - 1, nrow, col_data),
        ))
    }

    /// Split a packed arc array into the row-arc and column-arc rasters that
    /// the RowCol on-disk layout stores.
    fn flat_arcs_to_rasters(
        values: &[i16],
        nrow: usize,
        ncol: usize,
    ) -> io::Result<(Raster<i16>, Raster<i16>)> {
        let row_arcs = (nrow - 1) * ncol;
        let col_arcs = nrow * (ncol - 1);
        if values.len() != row_arcs + col_arcs {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "arc array length {} does not match expected {}",
                    values.len(),
                    row_arcs + col_arcs
                ),
            ));
        }
        Ok((
            Raster::new(ncol, nrow - 1, values[..row_arcs].to_vec()),
            Raster::new(ncol - 1, nrow, values[row_arcs..].to_vec()),
        ))
    }

    /// Write the combined RowCol dump plus the separate row/column dumps for
    /// one jagged arc array, skipping whichever filenames are empty.
    fn write_arc_dumps<T: Clone + NativeWritable>(
        values: &[Vec<T>],
        nrow: usize,
        ncol: usize,
        both_file: &str,
        row_file: &str,
        col_file: &str,
        keys: (&str, &str, &str),
    ) -> io::Result<()> {
        if both_file.is_empty() && row_file.is_empty() && col_file.is_empty() {
            return Ok(());
        }
        let (row_arcs, col_arcs) = row_col_to_rasters(values, nrow, ncol)?;
        if !both_file.is_empty() {
            let real = write_2d_row_col_array(&row_arcs, &col_arcs, Path::new(both_file))?;
            log::debug!(
                "native Rust wrote {} {} (requested {both_file})",
                keys.0,
                real.display()
            );
        }
        if !row_file.is_empty() {
            let real = write_2d_array(
                row_arcs.as_slice(),
                row_arcs.height,
                row_arcs.width,
                Path::new(row_file),
            )?;
            log::debug!(
                "native Rust wrote {} {} (requested {row_file})",
                keys.1,
                real.display()
            );
        }
        if !col_file.is_empty() {
            let real = write_2d_array(
                col_arcs.as_slice(),
                col_arcs.height,
                col_arcs.width,
                Path::new(col_file),
            )?;
            log::debug!(
                "native Rust wrote {} {} (requested {col_file})",
                keys.2,
                real.display()
            );
        }
        Ok(())
    }

    let raw_args = args
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect::<Vec<_>>();

    if raw_args.len() <= 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, FULL_HELP));
    }

    let mut infiles = InputFiles::default();
    let mut outfiles = OutputFiles::default();
    let mut linelen = 0usize;
    let mut params = crate::config::RunConfig::default();

    match process_args(
        &raw_args,
        &mut infiles,
        &mut outfiles,
        &mut linelen,
        &mut params,
    ) {
        Err(ProcessArgsError::HelpRequested) => {
            println!("{FULL_HELP}");
            return Ok(());
        }
        Err(ProcessArgsError::CopyrightRequested) => {
            println!("snaphu-rs native Rust path (work in progress)");
            return Ok(());
        }
        Err(ProcessArgsError::NoArguments | ProcessArgsError::NotEnoughPositionalArgs) => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, FULL_HELP));
        }
        Err(other) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("argument error: {other}"),
            ));
        }
        Ok(()) => {}
    }

    if params.onetilereopt {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native Rust CLI does not support single-tile reoptimization (-S) yet",
        ));
    }
    if params.assemble_only {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native Rust CLI does not support --assemble mode yet",
        ));
    }
    if params.eval {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native Rust CLI quantify-only mode is not implemented yet",
        ));
    }
    if !outfiles.logfile.is_empty() {
        log::warn!(
            "LOGFILE is ignored in native Rust path; use RUST_LOG/env_logger for runtime logging"
        );
    }

    let multi_tile = params.ntilerow != 1 || params.ntilecol != 1;

    let infile_path = Path::new(&infiles.infile);
    let infile_format = to_input_file_format(params.infile_format);
    let unwrapped_line_format = to_input_file_format(params.unwrapped_infile_format);
    if linelen == 0 {
        // Only reachable for the self-describing `.phase` input format, whose
        // header supplies the width the linelength positional would have.
        linelen = read_phase_header(infile_path)?.ncols;
    }
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
            unwrapped_infile_format: to_raster_file_format(params.unwrapped_infile_format),
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
            match params.magfile_format {
                FileFormat::ComplexData => MagnitudeFileFormat::ComplexData,
                FileFormat::FloatData => MagnitudeFileFormat::FloatData,
                FileFormat::AltSampleData => MagnitudeFileFormat::AltSampleData,
                FileFormat::AltLineData => MagnitudeFileFormat::AltLineData,
                FileFormat::FloatDataPhase => MagnitudeFileFormat::FloatDataPhase,
            },
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

    let power_raster = if !infiles.ampfile.is_empty() {
        let intensity = read_intensity(
            &IntensityFiles {
                ampfile: PathBuf::from(&infiles.ampfile),
                ampfile2: if infiles.ampfile2.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&infiles.ampfile2))
                },
                ampfile_format: to_raster_file_format(params.ampfile_format),
            },
            linelen,
            nlines_full,
            window,
            params.amplitude,
        )?;
        Some(intensity.pwr)
    } else {
        None
    };

    let corr_raster = if !infiles.corrfile.is_empty() {
        let corr = read_correlation(
            &CorrelationFile {
                corrfile: PathBuf::from(&infiles.corrfile),
                corrfile_format: to_raster_file_format(params.corrfile_format),
            },
            linelen,
            nlines_full,
            window,
        )?;
        Some(corr)
    } else {
        None
    };

    let estimate_raster = if !infiles.estfile.is_empty() {
        Some(read_unwrapped_estimate_file(
            Path::new(&infiles.estfile),
            to_raster_file_format(params.unwrapped_infile_format),
            linelen,
            nlines_full,
            window,
            false,
        )?)
    } else {
        None
    };

    let weight_tile = if !infiles.weightfile.is_empty() {
        Some(read_weights_file(
            Some(Path::new(&infiles.weightfile)),
            linelen,
            nlines_full,
            window,
        )?)
    } else {
        None
    };

    // Everything above this point is file I/O; the unwrapping itself is one
    // call into the library API.
    let tile_mask = if infiles.dotilemaskfile.is_empty() {
        None
    } else {
        Some(set_up_do_tile_mask(
            Some(Path::new(&infiles.dotilemaskfile)),
            params.ntilerow,
            params.ntilecol,
        )?)
    };

    let inputs = UnwrapInputs {
        wrapped_phase: &input.wrapped_phase,
        magnitude: Some(&input.mag),
        power: power_raster.as_ref(),
        correlation: corr_raster.as_ref(),
        unwrapped_estimate: estimate_raster.as_ref(),
        arc_weights: weight_tile.as_ref(),
        initial_flows: input.flows.as_deref(),
        // The byte mask was already folded into the magnitude on read.
        mask: None,
        tile_mask: tile_mask.as_ref().map(|m| m.data.as_slice()),
        // read_input_file already wrapped the phase into [0, 2pi).
        wrap_input: false,
    };

    let out = run_snaphu(&inputs, &params)
        .map_err(|err| io::Error::other(format!("native unwrap failed: {err}")))?;
    let unwrapped = out.unwrapped_phase;

    if let Some(flows) = &out.flows {
        if !outfiles.flowfile.is_empty() {
            let (row_arcs, col_arcs) = flat_arcs_to_rasters(flows, window.nrow, window.ncol)?;
            let real = write_2d_row_col_array(&row_arcs, &col_arcs, Path::new(&outfiles.flowfile))?;
            log::debug!(
                "native Rust wrote FLOWFILE {} (requested {})",
                real.display(),
                outfiles.flowfile
            );
        }
    }

    if let Some(diagnostics) = &out.diagnostics {
        if let Some(topo) = &diagnostics.topo_diagnostics
            && !outfiles.eifile.is_empty()
        {
            let real = write_2d_array(
                topo.normalized_intensity.as_slice(),
                topo.normalized_intensity.height,
                topo.normalized_intensity.width,
                Path::new(&outfiles.eifile),
            )?;
            log::debug!(
                "native Rust wrote EIFILE {} (requested {})",
                real.display(),
                outfiles.eifile
            );
        }

        let cost_keys = ("COSTOUTFILE", "ROWCOSTFILE", "COLCOSTFILE");
        match &diagnostics.costs {
            CostArrayData::Topo(costs) | CostArrayData::Defo(costs) => write_arc_dumps(
                costs,
                window.nrow,
                window.ncol,
                &outfiles.costoutfile,
                &outfiles.rowcostfile,
                &outfiles.colcostfile,
                cost_keys,
            )?,
            CostArrayData::Smooth(costs) => write_arc_dumps(
                costs,
                window.nrow,
                window.ncol,
                &outfiles.costoutfile,
                &outfiles.rowcostfile,
                &outfiles.colcostfile,
                cost_keys,
            )?,
            CostArrayData::Scalar(_) => {}
        }
        write_arc_dumps(
            &diagnostics.mst_costs,
            window.nrow,
            window.ncol,
            &outfiles.mstcostsfile,
            &outfiles.mstrowcostfile,
            &outfiles.mstcolcostfile,
            ("MSTCOSTSFILE", "MSTROWCOSTFILE", "MSTCOLCOSTFILE"),
        )?;
    }

    if !outfiles.initfile.is_empty() {
        let real = write_2d_array(
            &unwrapped.data,
            window.nrow,
            window.ncol,
            Path::new(&outfiles.initfile),
        )?;
        log::debug!(
            "native Rust wrote INITFILE {} (requested {})",
            real.display(),
            outfiles.initfile
        );
    }

    let _written = write_output_file(
        &out.magnitude,
        &unwrapped,
        Path::new(&outfiles.outfile),
        to_output_file_format(params.outfile_format),
    )?;

    if params.verbose {
        log::info!(
            "native Rust CLI wrote {}x{}{} output to {}",
            window.nrow,
            window.ncol,
            if multi_tile { " multi-tile" } else { "" },
            outfiles.outfile
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_cli;
    use std::fs;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!("snaphu_rs_{name}_{}_{}", std::process::id(), stamp))
    }

    #[test]
    fn run_cli_native_help_returns_ok() {
        assert!(run_cli(["snaphu", "--help"]).is_ok());
    }

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
