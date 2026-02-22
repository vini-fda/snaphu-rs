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
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "legacy CLI disabled; enable the `legacy-cli` feature to run the original C implementation",
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder_translation_state() {
        const {
            assert!(cfg!(feature = "legacy-cli"));
        }
    }
}
