#![allow(dead_code)]

//! Network graph construction and solver wrappers.

pub mod bucket;

use crate::constants::MASKED;

pub struct TileGraph;

pub trait MinCostFlowSolver {
    fn solve(&mut self, graph: &TileGraph) -> Result<(), String>;
}

/// Returns [`MASKED`] if all four pixels surrounding a grid node have zero
/// magnitude, and `0` otherwise.
///
/// This is the idiomatic Rust equivalent of the C `GridNodeMaskStatus()`
/// function. The grid node at `(row, col)` is surrounded by pixels at
/// `(row, col)`, `(row, col+1)`, `(row+1, col)`, and `(row+1, col+1)` in
/// the magnitude raster.
///
/// `mag` is a row-major flat slice with `ncol` columns.
///
/// # Panics
///
/// Panics if `row+1` or `col+1` exceeds the raster dimensions.
pub fn grid_node_mask_status(row: usize, col: usize, mag: &[f32], ncol: usize) -> i32 {
    let idx = |r: usize, c: usize| r * ncol + c;
    if mag[idx(row, col)] != 0.0
        || mag[idx(row, col + 1)] != 0.0
        || mag[idx(row + 1, col)] != 0.0
        || mag[idx(row + 1, col + 1)] != 0.0
    {
        0
    } else {
        MASKED
    }
}

/// Returns [`MASKED`] if all pixels along the grid boundary have zero
/// magnitude, and `0` otherwise.
///
/// This is the idiomatic Rust equivalent of the C `GroundMaskStatus()`
/// function. It checks the first and last column of every row, then the
/// first and last row of every column.
///
/// `mag` is a row-major flat slice of dimensions `nrow × ncol`.
pub fn ground_mask_status(mag: &[f32], nrow: usize, ncol: usize) -> i32 {
    let idx = |r: usize, c: usize| r * ncol + c;

    // Check left and right edges.
    for row in 0..nrow {
        if mag[idx(row, 0)] != 0.0 || mag[idx(row, ncol - 1)] != 0.0 {
            return 0;
        }
    }
    // Check top and bottom edges.
    for col in 0..ncol {
        if mag[idx(0, col)] != 0.0 || mag[idx(nrow - 1, col)] != 0.0 {
            return 0;
        }
    }
    MASKED
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_node_all_zero_returns_masked() {
        // 3×3 raster, all zeros.
        let mag = vec![0.0f32; 9];
        assert_eq!(grid_node_mask_status(0, 0, &mag, 3), MASKED);
        assert_eq!(grid_node_mask_status(1, 1, &mag, 3), MASKED);
    }

    #[test]
    fn grid_node_any_nonzero_returns_zero() {
        // 3×3 raster with one nonzero pixel at (1,1).
        let mut mag = vec![0.0f32; 9];
        mag[1 * 3 + 1] = 1.0;
        // Node (0,0) touches pixel (1,1) → not masked.
        assert_eq!(grid_node_mask_status(0, 0, &mag, 3), 0);
        // Node (1,0) does NOT touch pixel (1,1) in its 2×2 — touches
        // (1,0),(1,1),(2,0),(2,1). pixel (1,1) is nonzero → not masked.
        assert_eq!(grid_node_mask_status(1, 0, &mag, 3), 0);
    }

    #[test]
    fn grid_node_only_far_pixel_nonzero() {
        // 3×3 raster with nonzero only at (0,2).
        let mut mag = vec![0.0f32; 9];
        mag[0 * 3 + 2] = 5.0;
        // Node (0,0) surrounds (0,0),(0,1),(1,0),(1,1) — all zero → MASKED.
        assert_eq!(grid_node_mask_status(0, 0, &mag, 3), MASKED);
        // Node (0,1) surrounds (0,1),(0,2),(1,1),(1,2) — (0,2) nonzero → 0.
        assert_eq!(grid_node_mask_status(0, 1, &mag, 3), 0);
    }

    #[test]
    fn ground_all_zero_returns_masked() {
        let mag = vec![0.0f32; 12]; // 3×4
        assert_eq!(ground_mask_status(&mag, 3, 4), MASKED);
    }

    #[test]
    fn ground_edge_nonzero_returns_zero() {
        let mut mag = vec![0.0f32; 12]; // 3×4
        // Set a pixel on the left edge.
        mag[1 * 4 + 0] = 1.0;
        assert_eq!(ground_mask_status(&mag, 3, 4), 0);
    }

    #[test]
    fn ground_interior_nonzero_still_masked() {
        let mut mag = vec![0.0f32; 12]; // 3×4
        // Set an interior pixel — edges are still all zero.
        mag[1 * 4 + 1] = 99.0;
        assert_eq!(ground_mask_status(&mag, 3, 4), MASKED);
    }

    #[test]
    fn ground_bottom_right_corner_nonzero() {
        let mut mag = vec![0.0f32; 12]; // 3×4
        mag[2 * 4 + 3] = 0.5;
        assert_eq!(ground_mask_status(&mag, 3, 4), 0);
    }
}
