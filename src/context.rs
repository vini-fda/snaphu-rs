#![allow(dead_code)]

//! Shared runtime state for the SNAPHU algorithm.

use crate::config::RunConfig;

/// Aggregates the configuration and long-lived buffers used through the run.
#[derive(Debug)]
pub struct SnaphuContext {
    pub config: RunConfig,
}

impl SnaphuContext {
    pub fn new(config: RunConfig) -> Self {
        Self { config }
    }
}
