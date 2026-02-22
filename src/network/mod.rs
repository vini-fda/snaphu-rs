#![allow(dead_code)]

//! Network graph construction and solver wrappers.

pub mod bucket;

use crate::constants::{GROUNDROW, MASKED};

pub struct TileGraph;

pub trait MinCostFlowSolver {
    fn solve(&mut self, graph: &TileGraph) -> Result<(), String>;
}

/// Error cases for translated network topology helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkError {
    /// The caller requested boundary-node arc limits but did not supply
    /// boundary neighbor counts.
    MissingBoundaryNeighborCount,
}

/// Initial and ending values used by C-style neighbor scans.
///
/// C's `GetArcNumLims()` returned `arcnum` and wrote `upperarcnum` via an
/// output pointer. The Rust equivalent returns both values together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcNumLimits {
    pub arcnum_start: i64,
    pub upper_arcnum: i64,
}

/// Get the initial and ending values for `arcnum` to scan neighbors of a node.
///
/// This is the idiomatic Rust equivalent of the C `GetArcNumLims()` function.
/// Values are preserved exactly:
/// - grid nodes (`from_row >= 0`): `arcnum_start = -5`, `upper_arcnum = -1`
/// - ground node (`from_row == GROUNDROW`): `arcnum_start = -1`,
///   `upper_arcnum = nground_arcs - 1`
/// - boundary nodes (other negative rows): `arcnum_start = -1`,
///   `upper_arcnum = boundary_neighbor_count - 1`
pub fn get_arc_num_lims(
    from_row: i64,
    nground_arcs: i64,
    boundary_neighbor_count: Option<i64>,
) -> Result<ArcNumLimits, NetworkError> {
    if from_row < 0 {
        if from_row == GROUNDROW {
            return Ok(ArcNumLimits {
                arcnum_start: -1,
                upper_arcnum: nground_arcs - 1,
            });
        }

        let boundary_neighbor_count =
            boundary_neighbor_count.ok_or(NetworkError::MissingBoundaryNeighborCount)?;
        return Ok(ArcNumLimits {
            arcnum_start: -1,
            upper_arcnum: boundary_neighbor_count - 1,
        });
    }

    Ok(ArcNumLimits {
        arcnum_start: -5,
        upper_arcnum: -1,
    })
}

/// Supplementary per-node metadata for secondary (tile-overlap) networks.
///
/// Mirrors the `row` and `col` fields of the C `nodesuppT` struct, which
/// record each secondary node's position in the primary network grid.
/// Pointer-heavy fields (`neighbornodes`, `outarcs`) are not replicated here;
/// they will be expressed as adjacency structures when the tile-assembly code
/// is translated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeSupp {
    pub row: i32,
    pub col: i32,
}

impl NodeSupp {
    pub fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }
}

/// Find the index of a secondary node whose primary-network coordinates
/// match `(primary_row, primary_col)`.
///
/// This is the idiomatic Rust equivalent of the C `FindScndryNode()` function,
/// which performed an unbounded linear scan and dereferenced the result as a
/// pointer. The Rust version returns `Option<usize>` so the caller can handle
/// the not-found case safely.
pub fn find_secondary_node_index(
    node_supp: &[NodeSupp],
    primary_row: i32,
    primary_col: i32,
) -> Option<usize> {
    node_supp
        .iter()
        .position(|ns| ns.row == primary_row && ns.col == primary_col)
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

/// Resolve the two pixel coordinates on either side of an arc.
///
/// In the SNAPHU grid encoding, arcs with `arcrow < nrow - 1` are **row arcs**
/// (vertical, separating pixel `(arcrow, arccol)` from `(arcrow+1, arccol)`),
/// while arcs with `arcrow >= nrow - 1` are **column arcs** (horizontal,
/// separating pixel `(arcrow - (nrow-1), arccol)` from the same row at
/// `arccol+1`).
///
/// Returns `((row1, col1), (row2, col2))`.
#[inline]
fn arc_pixel_pair(arcrow: usize, arccol: usize, nrow: usize) -> ((usize, usize), (usize, usize)) {
    if arcrow < nrow - 1 {
        ((arcrow, arccol), (arcrow + 1, arccol))
    } else {
        let row = arcrow - (nrow - 1);
        ((row, arccol), (row, arccol + 1))
    }
}

/// Returns `true` if the arc connects two pixels where at least one has
/// nonzero magnitude — i.e. the arc belongs to a region.
///
/// Equivalent to the C `IsRegionArc()` function.
/// If `mag` is `None`, all arcs are considered region arcs (single-region mode).
pub fn is_region_arc(
    mag: Option<&[f32]>,
    arcrow: usize,
    arccol: usize,
    nrow: usize,
    ncol: usize,
) -> bool {
    let mag = match mag {
        Some(m) => m,
        None => return true,
    };
    let ((r1, c1), (r2, c2)) = arc_pixel_pair(arcrow, arccol, nrow);
    let idx = |r: usize, c: usize| r * ncol + c;
    mag[idx(r1, c1)] > 0.0 || mag[idx(r2, c2)] > 0.0
}

/// Returns `true` if the arc lies on a region edge — exactly one of the two
/// adjacent pixels has zero magnitude.
///
/// Equivalent to the C `IsRegionEdgeArc()` function.
/// If `mag` is `None`, no arcs are edge arcs (single-region mode).
pub fn is_region_edge_arc(
    mag: Option<&[f32]>,
    arcrow: usize,
    arccol: usize,
    nrow: usize,
    ncol: usize,
) -> bool {
    let mag = match mag {
        Some(m) => m,
        None => return false,
    };
    let ((r1, c1), (r2, c2)) = arc_pixel_pair(arcrow, arccol, nrow);
    let idx = |r: usize, c: usize| r * ncol + c;
    let nzeromag = (mag[idx(r1, c1)] == 0.0) as u32 + (mag[idx(r2, c2)] == 0.0) as u32;
    nzeromag == 1
}

/// Returns `true` if the arc connects two pixels that both have nonzero
/// magnitude — i.e. the arc is fully interior to a region.
///
/// Equivalent to the C `IsRegionInteriorArc()` function.
/// If `mag` is `None`, all arcs are considered interior (single-region mode).
pub fn is_region_interior_arc(
    mag: Option<&[f32]>,
    arcrow: usize,
    arccol: usize,
    nrow: usize,
    ncol: usize,
) -> bool {
    let mag = match mag {
        Some(m) => m,
        None => return true,
    };
    let ((r1, c1), (r2, c2)) = arc_pixel_pair(arcrow, arccol, nrow);
    let idx = |r: usize, c: usize| r * ncol + c;
    mag[idx(r1, c1)] > 0.0 && mag[idx(r2, c2)] > 0.0
}

/// Returns `true` if the node at `(row, col)` touches at least one zero-
/// magnitude pixel **and** at least one nonzero-magnitude pixel — i.e. the
/// node sits on a region boundary.
///
/// Equivalent to the C `IsRegionEdgeNode()` function.
/// Returns `false` for the ground node (`row == GROUNDROW`) or when `mag`
/// is `None`.
pub fn is_region_edge_node(
    mag: Option<&[f32]>,
    row: i64,
    col: usize,
    _nrow: usize,
    ncol: usize,
) -> bool {
    let mag = match mag {
        Some(m) => m,
        None => return false,
    };
    if row == GROUNDROW {
        return false;
    }
    let r = row as usize;
    let idx = |rr: usize, cc: usize| rr * ncol + cc;
    let pixels = [
        mag[idx(r, col)],
        mag[idx(r + 1, col)],
        mag[idx(r, col + 1)],
        mag[idx(r + 1, col + 1)],
    ];
    let has_zero = pixels.iter().any(|&v| v == 0.0);
    let has_nonzero = pixels.iter().any(|&v| v != 0.0);
    has_zero && has_nonzero
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- GetArcNumLims ---

    #[test]
    fn get_arc_num_lims_grid_node() {
        let limits = get_arc_num_lims(3, 4, None).unwrap();
        assert_eq!(
            limits,
            ArcNumLimits {
                arcnum_start: -5,
                upper_arcnum: -1
            }
        );
    }

    #[test]
    fn get_arc_num_lims_ground_node() {
        let limits = get_arc_num_lims(GROUNDROW, 6, None).unwrap();
        assert_eq!(
            limits,
            ArcNumLimits {
                arcnum_start: -1,
                upper_arcnum: 5
            }
        );
    }

    #[test]
    fn get_arc_num_lims_ground_node_with_zero_ground_arcs() {
        let limits = get_arc_num_lims(GROUNDROW, 0, None).unwrap();
        assert_eq!(
            limits,
            ArcNumLimits {
                arcnum_start: -1,
                upper_arcnum: -1
            }
        );
    }

    #[test]
    fn get_arc_num_lims_boundary_node() {
        // Any negative row that is not GROUNDROW is treated as boundary.
        let limits = get_arc_num_lims(-4, 99, Some(8)).unwrap();
        assert_eq!(
            limits,
            ArcNumLimits {
                arcnum_start: -1,
                upper_arcnum: 7
            }
        );
    }

    #[test]
    fn get_arc_num_lims_boundary_node_missing_count_errors() {
        let err = get_arc_num_lims(-4, 5, None).unwrap_err();
        assert_eq!(err, NetworkError::MissingBoundaryNeighborCount);
    }

    #[test]
    fn get_arc_num_lims_boundary_node_with_zero_neighbors() {
        let limits = get_arc_num_lims(-4, 12, Some(0)).unwrap();
        assert_eq!(
            limits,
            ArcNumLimits {
                arcnum_start: -1,
                upper_arcnum: -1
            }
        );
    }

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

    // --- IsRegionArc ---

    #[test]
    fn region_arc_none_mag_returns_true() {
        assert!(is_region_arc(None, 0, 0, 3, 4));
    }

    #[test]
    fn region_arc_both_zero_returns_false() {
        // 3×4 all zeros, row arc (arcrow=0 < nrow-1=2)
        let mag = vec![0.0f32; 12];
        assert!(!is_region_arc(Some(&mag), 0, 0, 3, 4));
    }

    #[test]
    fn region_arc_one_nonzero_returns_true() {
        let mut mag = vec![0.0f32; 12]; // 3×4
        // Row arc at (0,1): pixels (0,1) and (1,1).
        mag[0 * 4 + 1] = 1.0;
        assert!(is_region_arc(Some(&mag), 0, 1, 3, 4));
    }

    #[test]
    fn region_arc_column_arc() {
        // Column arc: arcrow >= nrow-1. For nrow=3, arcrow=2 → row = 2-2 = 0
        // pixels (0, arccol) and (0, arccol+1).
        let mut mag = vec![0.0f32; 12]; // 3×4
        mag[0 * 4 + 2] = 5.0; // pixel (0,2)
        assert!(is_region_arc(Some(&mag), 2, 1, 3, 4)); // pixels (0,1)=0, (0,2)=5 → true
    }

    // --- IsRegionEdgeArc ---

    #[test]
    fn region_edge_arc_none_mag_returns_false() {
        assert!(!is_region_edge_arc(None, 0, 0, 3, 4));
    }

    #[test]
    fn region_edge_arc_exactly_one_zero() {
        let mut mag = vec![1.0f32; 12]; // 3×4, all nonzero
        // Row arc (0,0): pixels (0,0) and (1,0). Zero out one.
        mag[1 * 4 + 0] = 0.0;
        assert!(is_region_edge_arc(Some(&mag), 0, 0, 3, 4));
    }

    #[test]
    fn region_edge_arc_both_zero_returns_false() {
        let mag = vec![0.0f32; 12];
        assert!(!is_region_edge_arc(Some(&mag), 0, 0, 3, 4));
    }

    #[test]
    fn region_edge_arc_both_nonzero_returns_false() {
        let mag = vec![1.0f32; 12];
        assert!(!is_region_edge_arc(Some(&mag), 0, 0, 3, 4));
    }

    // --- IsRegionInteriorArc ---

    #[test]
    fn region_interior_arc_none_mag_returns_true() {
        assert!(is_region_interior_arc(None, 0, 0, 3, 4));
    }

    #[test]
    fn region_interior_arc_both_nonzero() {
        let mag = vec![1.0f32; 12];
        assert!(is_region_interior_arc(Some(&mag), 0, 0, 3, 4));
    }

    #[test]
    fn region_interior_arc_one_zero_returns_false() {
        let mut mag = vec![1.0f32; 12];
        mag[0] = 0.0;
        assert!(!is_region_interior_arc(Some(&mag), 0, 0, 3, 4));
    }

    // --- IsRegionEdgeNode ---

    #[test]
    fn region_edge_node_none_mag_returns_false() {
        assert!(!is_region_edge_node(None, 0, 0, 3, 4));
    }

    #[test]
    fn region_edge_node_ground_row_returns_false() {
        let mag = vec![1.0f32; 12];
        assert!(!is_region_edge_node(Some(&mag), GROUNDROW, 0, 3, 4));
    }

    #[test]
    fn region_edge_node_mixed_magnitudes() {
        // 3×4, node (0,0) surrounds pixels (0,0),(0,1),(1,0),(1,1).
        let mut mag = vec![1.0f32; 12];
        mag[0] = 0.0; // pixel (0,0) zeroed
        assert!(is_region_edge_node(Some(&mag), 0, 0, 3, 4));
    }

    #[test]
    fn region_edge_node_all_nonzero_returns_false() {
        let mag = vec![1.0f32; 12];
        assert!(!is_region_edge_node(Some(&mag), 0, 0, 3, 4));
    }

    #[test]
    fn region_edge_node_all_zero_returns_false() {
        let mag = vec![0.0f32; 12];
        assert!(!is_region_edge_node(Some(&mag), 0, 0, 3, 4));
    }

    // --- FindScndryNode ---

    #[test]
    fn find_secondary_node_hit() {
        let nodes = vec![
            NodeSupp::new(0, 0),
            NodeSupp::new(1, 2),
            NodeSupp::new(3, 4),
        ];
        assert_eq!(find_secondary_node_index(&nodes, 1, 2), Some(1));
    }

    #[test]
    fn find_secondary_node_miss() {
        let nodes = vec![NodeSupp::new(0, 0), NodeSupp::new(1, 2)];
        assert_eq!(find_secondary_node_index(&nodes, 5, 5), None);
    }

    #[test]
    fn find_secondary_node_first_element() {
        let nodes = vec![NodeSupp::new(7, 8), NodeSupp::new(1, 2)];
        assert_eq!(find_secondary_node_index(&nodes, 7, 8), Some(0));
    }

    #[test]
    fn find_secondary_node_last_element() {
        let nodes = vec![NodeSupp::new(0, 0), NodeSupp::new(9, 9)];
        assert_eq!(find_secondary_node_index(&nodes, 9, 9), Some(1));
    }

    #[test]
    fn find_secondary_node_empty_slice() {
        assert_eq!(find_secondary_node_index(&[], 0, 0), None);
    }
}
