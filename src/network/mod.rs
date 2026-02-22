#![allow(dead_code)]

//! Network graph construction and solver wrappers.

pub struct TileGraph;

pub trait FlowSolver {
    fn solve(&mut self, graph: &TileGraph) -> Result<(), String>;
}
