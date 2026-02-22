#![allow(dead_code)]

//! Lookup table builders for rho/dz statistics.

pub struct LookupTable {
    pub values: Vec<f32>,
}

impl LookupTable {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }
}
