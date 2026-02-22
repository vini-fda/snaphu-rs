#![allow(dead_code)]

//! Command-line interface scaffolding.
//!
//! The real CLI implementation will sit here instead of calling directly into
//! the legacy C entrypoint via `run_cli`.

use std::io;
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

    #[test]
    fn parse_keeps_raw_arguments() {
        let args = parse(["snaphu", "-v", "input.bin"]);
        assert_eq!(args.raw, vec!["snaphu", "-v", "input.bin"]);
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
}
