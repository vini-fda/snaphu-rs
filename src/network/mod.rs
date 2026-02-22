#![allow(dead_code)]

//! Network graph construction and solver wrappers.

pub mod bucket;

pub struct TileGraph;

pub trait FlowSolver {
    fn solve(&mut self, graph: &TileGraph) -> Result<(), String>;
}
