#![allow(dead_code)]

//! Configuration types for running SNAPHU.

pub mod defaults;

/// Placeholder for the eventual full configuration structure.
#[derive(Debug, Clone, Default)]
pub struct RunConfig {
    pub description: String,
}
