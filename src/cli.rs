#![allow(dead_code)]

//! Command-line interface scaffolding.
//!
//! The real CLI implementation will sit here instead of calling directly into
//! the legacy C entrypoint via `run_cli`.

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
