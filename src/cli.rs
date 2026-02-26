//! Command-line interface to snaphu-rs.

use crate::config::{
    CostMode, InitMethod, InputFiles, OutputFiles, RunConfig, apply_config_entries,
    read_config_file,
};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Temporary representation for parsed CLI arguments.
#[derive(Debug, Default, Clone)]
pub struct CliArgs {
    pub raw: Vec<String>,
}

impl CliArgs {
    pub fn new(raw: Vec<String>) -> Self {
        Self { raw }
    }
}
/// Placeholder parser that simply stores the provided arguments.
pub fn parse<I, S>(args: I) -> CliArgs
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    CliArgs::new(args.into_iter().map(|s| s.as_ref().to_string()).collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessArgsError {
    NoArguments,
    UnknownOption(String),
    MissingValue(String),
    InvalidValue {
        option: String,
        value: String,
    },
    ConfigReadFailed {
        option: String,
        path: String,
        error: String,
    },
    NotEnoughPositionalArgs,
    MultipleInputFiles {
        first: String,
        second: String,
    },
    HelpRequested,
    CopyrightRequested,
}

impl std::fmt::Display for ProcessArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessArgsError::NoArguments => write!(f, "no arguments provided"),
            ProcessArgsError::UnknownOption(opt) => write!(f, "unknown option: {opt}"),
            ProcessArgsError::MissingValue(opt) => write!(f, "missing value for option: {opt}"),
            ProcessArgsError::InvalidValue { option, value } => {
                write!(f, "invalid value for {option}: {value}")
            }
            ProcessArgsError::ConfigReadFailed {
                option,
                path,
                error,
            } => write!(
                f,
                "failed to read config file for {option} ('{path}'): {error}"
            ),
            ProcessArgsError::NotEnoughPositionalArgs => {
                write!(f, "missing required positional arguments")
            }
            ProcessArgsError::MultipleInputFiles { first, second } => {
                write!(f, "multiple input files provided: '{first}' and '{second}'")
            }
            ProcessArgsError::HelpRequested => write!(f, "help requested"),
            ProcessArgsError::CopyrightRequested => write!(f, "copyright requested"),
        }
    }
}

fn parse_usize_arg(option: &str, value: &str) -> Result<usize, ProcessArgsError> {
    value
        .parse::<usize>()
        .map_err(|_| ProcessArgsError::InvalidValue {
            option: option.to_string(),
            value: value.to_string(),
        })
}

fn parse_f64_arg(option: &str, value: &str) -> Result<f64, ProcessArgsError> {
    value
        .parse::<f64>()
        .map_err(|_| ProcessArgsError::InvalidValue {
            option: option.to_string(),
            value: value.to_string(),
        })
}

fn next_arg(args: &[String], i: &mut usize, option: &str) -> Result<String, ProcessArgsError> {
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| ProcessArgsError::MissingValue(option.to_string()))
}

/// Parse CLI arguments into typed config/file structs.
///
/// This is the idiomatic Rust equivalent of C `ProcessArgs()`.
pub fn process_args(
    raw_args: &[String],
    infiles: &mut InputFiles,
    outfiles: &mut OutputFiles,
    linelen: &mut usize,
    params: &mut RunConfig,
) -> Result<(), ProcessArgsError> {
    if raw_args.len() < 2 {
        return Err(ProcessArgsError::NoArguments);
    }

    let mut i = 1usize;
    while i < raw_args.len() {
        let arg = &raw_args[i];
        if arg.starts_with("--") {
            match arg.as_str() {
                "--help" => return Err(ProcessArgsError::HelpRequested),
                "--costinfile" => infiles.costinfile = next_arg(raw_args, &mut i, arg)?,
                "--costoutfile" => outfiles.costoutfile = next_arg(raw_args, &mut i, arg)?,
                "--debug" | "--dumpall" => params.dump_all = true,
                "--mst" => params.init_method = InitMethod::Mst,
                "--mcf" => params.init_method = InitMethod::Mcf,
                "--aa" => {
                    infiles.ampfile = next_arg(raw_args, &mut i, arg)?;
                    infiles.ampfile2 = next_arg(raw_args, &mut i, arg)?;
                    params.amplitude = true;
                }
                "--AA" => {
                    infiles.ampfile = next_arg(raw_args, &mut i, arg)?;
                    infiles.ampfile2 = next_arg(raw_args, &mut i, arg)?;
                    params.amplitude = false;
                }
                "--tile" => {
                    params.ntilerow = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                    params.ntilecol = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                    params.rowovrlp = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                    params.colovrlp = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                }
                "--piece" => {
                    params.piecefirstrow = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                    params.piecefirstcol = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                    params.piecenrow = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                    params.piecencol = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?;
                }
                "--nproc" => {
                    params.nthreads = parse_usize_arg(arg, &next_arg(raw_args, &mut i, arg)?)?
                }
                "--tiledir" => params.tiledir = next_arg(raw_args, &mut i, arg)?,
                "--assemble" => params.assemble_only = true,
                "--copyright" | "--info" => return Err(ProcessArgsError::CopyrightRequested),
                _ => return Err(ProcessArgsError::UnknownOption(arg.clone())),
            }
            i += 1;
            continue;
        }

        if arg.starts_with('-') && arg.len() > 1 {
            let chars: Vec<char> = arg[1..].chars().collect();
            let mut j = 0usize;
            while j < chars.len() {
                let opt = chars[j];
                let option_name = format!("-{opt}");
                let is_last = j + 1 == chars.len();
                match opt {
                    'h' => return Err(ProcessArgsError::HelpRequested),
                    'u' => params.unwrapped = true,
                    't' => params.cost_mode = CostMode::Topo,
                    'd' => params.cost_mode = CostMode::Defo,
                    's' => {
                        params.cost_mode = CostMode::Smooth;
                    }
                    'q' => {
                        params.eval = true;
                        params.unwrapped = true;
                    }
                    'o' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        outfiles.outfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'c' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        infiles.corrfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'm' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        infiles.magfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'M' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        infiles.bytemaskfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'a' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        infiles.ampfile = next_arg(raw_args, &mut i, &option_name)?;
                        params.amplitude = true;
                        break;
                    }
                    'A' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        infiles.ampfile = next_arg(raw_args, &mut i, &option_name)?;
                        params.amplitude = false;
                        break;
                    }
                    'e' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        infiles.estfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'w' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        infiles.weightfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'g' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        outfiles.conncompfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'G' => {
                        params.regrow_conn_comps = true;
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        outfiles.conncompfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'b' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        params.bperp = parse_f64_arg(
                            &option_name,
                            &next_arg(raw_args, &mut i, &option_name)?,
                        )?;
                        break;
                    }
                    'p' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        params.p = parse_f64_arg(
                            &option_name,
                            &next_arg(raw_args, &mut i, &option_name)?,
                        )?;
                        break;
                    }
                    'i' => params.init_only = true,
                    'S' => params.onetilereopt = true,
                    'k' => {
                        params.rm_tmp_tile = false;
                        params.rm_tile_init = false;
                    }
                    'n' => params.cost_mode = CostMode::NoStatCosts,
                    'v' => params.verbose = true,
                    'l' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        outfiles.logfile = next_arg(raw_args, &mut i, &option_name)?;
                        break;
                    }
                    'f' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        let conf_path = next_arg(raw_args, &mut i, &option_name)?;
                        let entries =
                            read_config_file(std::path::Path::new(&conf_path)).map_err(|err| {
                                ProcessArgsError::ConfigReadFailed {
                                    option: option_name,
                                    path: conf_path.clone(),
                                    error: err.to_string(),
                                }
                            })?;
                        apply_config_entries(&entries, infiles, outfiles, params);
                        break;
                    }
                    'C' => {
                        if !is_last {
                            return Err(ProcessArgsError::MissingValue(option_name));
                        }
                        // -C parses a single config line inline
                        let conf_str = next_arg(raw_args, &mut i, &option_name)?;
                        if let Ok(Some(entry)) = crate::config::parse_config_line(&conf_str) {
                            apply_config_entries(&[entry], infiles, outfiles, params);
                        }
                        break;
                    }
                    _ => return Err(ProcessArgsError::UnknownOption(option_name)),
                }
                j += 1;
            }
            i += 1;
            continue;
        }

        if infiles.infile.is_empty() {
            infiles.infile = arg.clone();
        } else if *linelen == 0 {
            *linelen = parse_usize_arg("linelen", arg)?;
            if *linelen == 0 {
                return Err(ProcessArgsError::InvalidValue {
                    option: "linelen".to_string(),
                    value: arg.clone(),
                });
            }
        } else {
            return Err(ProcessArgsError::MultipleInputFiles {
                first: infiles.infile.clone(),
                second: arg.clone(),
            });
        }
        i += 1;
    }

    if infiles.infile.is_empty() || *linelen == 0 {
        return Err(ProcessArgsError::NotEnoughPositionalArgs);
    }
    Ok(())
}

/// Signals trapped by C `CatchSignals()`.
pub const CAUGHT_SIGNALS: [libc::c_int; 11] = [
    libc::SIGHUP,
    libc::SIGINT,
    libc::SIGQUIT,
    libc::SIGILL,
    libc::SIGABRT,
    libc::SIGFPE,
    libc::SIGSEGV,
    libc::SIGPIPE,
    libc::SIGALRM,
    libc::SIGTERM,
    libc::SIGBUS,
];

const NULL_FILE: &str = "/dev/null";
const LOG_FILE_ROOT: &str = "tmptilelog_";

/// Logical destination for SNAPHU stream pointers (`sp0..sp3`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamTarget {
    Stdout,
    Stderr,
    Stdin,
    NullFile(PathBuf),
    LogFile(PathBuf),
}

impl StreamTarget {
    fn is_stdout_or_stderr(&self) -> bool {
        matches!(self, StreamTarget::Stdout | StreamTarget::Stderr)
    }
}

/// Typed replacement for global C stream pointers (`sp0..sp3`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamPointers {
    pub sp0_error: StreamTarget,
    pub sp1_output: StreamTarget,
    pub sp2_verbose: StreamTarget,
    pub sp3_counter: StreamTarget,
}

/// Metadata produced when resetting child streams for tile unwrapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildLogReset {
    pub logfile: PathBuf,
}

/// Wall/CPU timer snapshot captured at program start.
///
/// This is the typed Rust equivalent of C `StartTimers()` output arguments.
#[derive(Debug, Clone, Copy)]
pub struct TimerSnapshot {
    pub wall_start: SystemTime,
    pub cpu_start_seconds: Option<f64>,
}

/// Elapsed wall and CPU timings derived from a [`TimerSnapshot`].
///
/// This mirrors the values C `DisplayElapsedTime()` computes before printing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElapsedTime {
    pub cpu_seconds: Option<f64>,
    pub wall_seconds: Option<f64>,
}

/// Installs one signal handler for the set of common abort signals.
///
/// This is the idiomatic Rust equivalent of C `CatchSignals()`.
pub fn catch_signals(sig_handler: libc::sighandler_t) -> io::Result<()> {
    catch_signals_with(|signum| {
        // SAFETY: Calling libc `signal()` is required to mirror the C behavior.
        // We pass through the caller-provided handler and return OS errors.
        let previous = unsafe { libc::signal(signum, sig_handler) };
        if previous == libc::SIG_ERR {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    })
}

/// Initialize default stream pointer targets.
///
/// This is the idiomatic Rust equivalent of C `SetStreamPointers()`.
pub fn set_stream_pointers() -> io::Result<StreamPointers> {
    Ok(StreamPointers {
        sp0_error: StreamTarget::Stderr,
        sp1_output: StreamTarget::Stdout,
        sp2_verbose: StreamTarget::NullFile(PathBuf::from(NULL_FILE)),
        sp3_counter: StreamTarget::NullFile(PathBuf::from(NULL_FILE)),
    })
}

/// Enable verbose stream output to stdout when requested.
///
/// This is the idiomatic Rust equivalent of C `SetVerboseOut()`.
pub fn set_verbose_out(streams: &mut StreamPointers, verbose: bool) {
    if verbose {
        streams.sp2_verbose = StreamTarget::Stdout;
        streams.sp3_counter = StreamTarget::Stdout;
    }
}

/// Reset stream pointers for a tile-processing child process.
///
/// This is the idiomatic Rust equivalent of C `ChildResetStreamPointers()`.
pub fn child_reset_stream_pointers(
    pid: u32,
    tilerow: i64,
    tilecol: i64,
    tiledir: &Path,
    streams: &mut StreamPointers,
) -> io::Result<ChildLogReset> {
    std::fs::create_dir_all(tiledir)?;
    let logfile = tiledir.join(format!("{LOG_FILE_ROOT}{tilerow}_{tilecol}"));
    let mut logfp = std::fs::File::create(&logfile)?;
    writeln!(
        logfp,
        "snaphu (pid {}): unwrapping tile at row {}, column {}\n",
        pid, tilerow, tilecol
    )?;
    if let Ok(cwd) = std::env::current_dir() {
        writeln!(logfp, "Current working directory is {}", cwd.display())?;
    }
    logfp.flush()?;

    if streams.sp2_verbose.is_stdout_or_stderr() {
        streams.sp2_verbose = StreamTarget::LogFile(logfile.clone());
    }
    if streams.sp1_output.is_stdout_or_stderr() {
        streams.sp1_output = StreamTarget::LogFile(logfile.clone());
    }
    if streams.sp0_error.is_stdout_or_stderr() {
        streams.sp0_error = StreamTarget::LogFile(logfile.clone());
    }
    // C always reopens `sp3` to NULLFILE after closing any prior non-stdio.
    streams.sp3_counter = StreamTarget::NullFile(PathBuf::from(NULL_FILE));

    Ok(ChildLogReset { logfile })
}

fn catch_signals_with<F>(mut install: F) -> io::Result<()>
where
    F: FnMut(libc::c_int) -> io::Result<()>,
{
    for signum in CAUGHT_SIGNALS {
        install(signum)?;
    }
    Ok(())
}

/// Capture wall-clock and combined (self + children) CPU start times.
///
/// This is the idiomatic Rust equivalent of C `StartTimers()`.
pub fn start_timers() -> TimerSnapshot {
    TimerSnapshot {
        wall_start: SystemTime::now(),
        cpu_start_seconds: current_cpu_seconds(),
    }
}

/// Compute elapsed wall-clock and CPU times since [`start_timers`].
///
/// This is the idiomatic Rust equivalent of C `DisplayElapsedTime()`.
pub fn display_elapsed_time(start: TimerSnapshot) -> ElapsedTime {
    let cpu_seconds = match (start.cpu_start_seconds, current_cpu_seconds()) {
        (Some(start_cpu), Some(stop_cpu)) if stop_cpu > 0.0 && start_cpu >= 0.0 => {
            Some((stop_cpu - start_cpu).max(0.0))
        }
        _ => None,
    };
    let wall_seconds = SystemTime::now()
        .duration_since(start.wall_start)
        .ok()
        .map(|d| d.as_secs_f64());

    ElapsedTime {
        cpu_seconds,
        wall_seconds,
    }
}

fn current_cpu_seconds() -> Option<f64> {
    // SAFETY: `getrusage` initializes `usage` on success; all accesses are to
    // valid fields, and constants are the standard libc selectors.
    unsafe {
        let mut usage: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut usage) != 0 {
            return None;
        }
        let mut cpu = timeval_to_seconds(usage.ru_utime) + timeval_to_seconds(usage.ru_stime);
        if libc::getrusage(libc::RUSAGE_CHILDREN, &mut usage) == 0 {
            cpu += timeval_to_seconds(usage.ru_utime) + timeval_to_seconds(usage.ru_stime);
        }
        Some(cpu)
    }
}

#[inline]
fn timeval_to_seconds(tv: libc::timeval) -> f64 {
    tv.tv_sec as f64 + (tv.tv_usec as f64 / 1_000_000.0)
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
    fn parse_keeps_raw_arguments() {
        let args = parse(["snaphu", "-v", "input.bin"]);
        assert_eq!(args.raw, vec!["snaphu", "-v", "input.bin"]);
    }

    #[test]
    fn process_args_parses_basic_short_options_and_positionals() {
        let mut infiles = InputFiles::default();
        let mut outfiles = OutputFiles::default();
        let mut params = RunConfig::default();
        let mut linelen = 0usize;
        let args = vec![
            "snaphu".to_string(),
            "-v".to_string(),
            "-d".to_string(),
            "-o".to_string(),
            "out.bin".to_string(),
            "wrapped.bin".to_string(),
            "512".to_string(),
        ];
        process_args(
            &args,
            &mut infiles,
            &mut outfiles,
            &mut linelen,
            &mut params,
        )
        .unwrap();
        assert!(params.verbose);
        assert!(matches!(params.cost_mode, CostMode::Defo));
        assert_eq!(outfiles.outfile, "out.bin");
        assert_eq!(infiles.infile, "wrapped.bin");
        assert_eq!(linelen, 512);
    }

    #[test]
    fn process_args_parses_long_tile_args() {
        let mut infiles = InputFiles::default();
        let mut outfiles = OutputFiles::default();
        let mut params = RunConfig::default();
        let mut linelen = 0usize;
        let args = vec![
            "snaphu".to_string(),
            "--tile".to_string(),
            "2".to_string(),
            "3".to_string(),
            "10".to_string(),
            "11".to_string(),
            "in.bin".to_string(),
            "400".to_string(),
        ];
        process_args(
            &args,
            &mut infiles,
            &mut outfiles,
            &mut linelen,
            &mut params,
        )
        .unwrap();
        assert_eq!(params.ntilerow, 2);
        assert_eq!(params.ntilecol, 3);
        assert_eq!(params.rowovrlp, 10);
        assert_eq!(params.colovrlp, 11);
    }

    #[test]
    fn process_args_reports_config_read_error_with_cause() {
        let mut infiles = InputFiles::default();
        let mut outfiles = OutputFiles::default();
        let mut params = RunConfig::default();
        let mut linelen = 0usize;
        let args = vec![
            "snaphu".to_string(),
            "-f".to_string(),
            "definitely_missing_config_file_12345.conf".to_string(),
            "in.bin".to_string(),
            "10".to_string(),
        ];

        let err = process_args(
            &args,
            &mut infiles,
            &mut outfiles,
            &mut linelen,
            &mut params,
        )
        .unwrap_err();

        match err {
            ProcessArgsError::ConfigReadFailed {
                option,
                path,
                error,
                ..
            } => {
                assert_eq!(option, "-f");
                assert_eq!(path, "definitely_missing_config_file_12345.conf");
                assert!(!error.is_empty());
            }
            other => panic!("expected ConfigReadFailed, got {other:?}"),
        }
    }

    #[test]
    fn catch_signals_registers_all_expected_signals() {
        let mut seen = Vec::new();
        catch_signals_with(|sig| {
            seen.push(sig);
            Ok(())
        })
        .unwrap();
        assert_eq!(seen, CAUGHT_SIGNALS);
    }

    #[test]
    fn start_and_display_timers_return_non_negative_elapsed_time() {
        let start = start_timers();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let elapsed = display_elapsed_time(start);
        assert!(elapsed.wall_seconds.unwrap_or_default() >= 0.0);
        if let Some(cpu) = elapsed.cpu_seconds {
            assert!(cpu >= 0.0);
        }
    }

    #[test]
    fn set_stream_pointers_assigns_expected_defaults() {
        let streams = set_stream_pointers().unwrap();
        assert_eq!(streams.sp0_error, StreamTarget::Stderr);
        assert_eq!(streams.sp1_output, StreamTarget::Stdout);
        assert_eq!(
            streams.sp2_verbose,
            StreamTarget::NullFile(PathBuf::from("/dev/null"))
        );
        assert_eq!(
            streams.sp3_counter,
            StreamTarget::NullFile(PathBuf::from("/dev/null"))
        );
    }

    #[test]
    fn set_verbose_out_routes_verbose_and_counter_to_stdout() {
        let mut streams = set_stream_pointers().unwrap();
        set_verbose_out(&mut streams, true);
        assert_eq!(streams.sp2_verbose, StreamTarget::Stdout);
        assert_eq!(streams.sp3_counter, StreamTarget::Stdout);
    }

    #[test]
    fn child_reset_stream_pointers_writes_log_and_redirects_targets() {
        let dir = std::env::temp_dir().join(format!("snaphu_child_streams_{}", unique_suffix()));
        fs::create_dir_all(&dir).unwrap();
        let mut streams = set_stream_pointers().unwrap();
        streams.sp2_verbose = StreamTarget::Stdout;
        streams.sp1_output = StreamTarget::Stderr;
        streams.sp0_error = StreamTarget::Stdout;
        streams.sp3_counter = StreamTarget::LogFile(dir.join("old_counter.log"));

        let reset = child_reset_stream_pointers(4242, 3, 7, &dir, &mut streams).unwrap();
        assert!(reset.logfile.exists());
        assert_eq!(
            streams.sp2_verbose,
            StreamTarget::LogFile(reset.logfile.clone())
        );
        assert_eq!(
            streams.sp1_output,
            StreamTarget::LogFile(reset.logfile.clone())
        );
        assert_eq!(
            streams.sp0_error,
            StreamTarget::LogFile(reset.logfile.clone())
        );
        assert_eq!(
            streams.sp3_counter,
            StreamTarget::NullFile(PathBuf::from("/dev/null"))
        );

        let content = fs::read_to_string(&reset.logfile).unwrap();
        assert!(content.contains("unwrapping tile at row 3, column 7"));

        fs::remove_file(&reset.logfile).unwrap();
        fs::remove_dir_all(&dir).unwrap();
    }
}
