#![allow(dead_code)]

//! Cost calculation traits and helpers.

pub mod lookup;

/// Trait implemented by each cost component once translated.
pub trait CostBuilder {
    fn name(&self) -> &str;
}
