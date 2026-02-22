#![allow(dead_code)]

//! Network graph construction and solver wrappers.

pub mod bucket;

use crate::constants::{GROUNDROW, MASKED};
use crate::costs::types::IncrCost;

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

/// Per-node state used by translated tree-solver helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeNodeGroup {
    NotInBucket,
    InBucket,
    OnTree,
    Pruned,
    Masked,
}

/// Minimal node view needed by `add_new_node()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeNode {
    pub outcost: i64,
    pub pred: Option<usize>,
    pub group: TreeNodeGroup,
    bucket_index: Option<i64>,
}

impl TreeNode {
    pub fn new(outcost: i64) -> Self {
        Self {
            outcost,
            pred: None,
            group: TreeNodeGroup::NotInBucket,
            bucket_index: None,
        }
    }

    pub fn bucket_index(&self) -> Option<i64> {
        self.bucket_index
    }
}

/// Bucket window for the translated tree-solver queue.
///
/// The C implementation stores nodes into buckets using signed indices
/// `[minind, maxind]`, with underflow and overflow clamping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontierBuckets {
    slots: Vec<Vec<usize>>,
    pub curr: i64,
    pub minind: i64,
    pub maxind: i64,
}

impl FrontierBuckets {
    pub fn new(minind: i64, maxind: i64, curr: i64) -> Result<Self, AddNodeError> {
        if maxind < minind {
            return Err(AddNodeError::InvalidBucketBounds { minind, maxind });
        }
        let width = (maxind - minind + 1) as usize;
        Ok(Self {
            slots: vec![Vec::new(); width],
            curr,
            minind,
            maxind,
        })
    }

    fn bucket_to_offset(&self, bucket: i64) -> Result<usize, AddNodeError> {
        if bucket < self.minind || bucket > self.maxind {
            return Err(AddNodeError::BucketOutOfRange {
                bucket,
                minind: self.minind,
                maxind: self.maxind,
            });
        }
        Ok((bucket - self.minind) as usize)
    }

    fn bucket_mut(&mut self, bucket: i64) -> Result<&mut Vec<usize>, AddNodeError> {
        let offset = self.bucket_to_offset(bucket)?;
        Ok(&mut self.slots[offset])
    }

    fn remove(&mut self, bucket: i64, node_idx: usize) -> Result<(), AddNodeError> {
        let slot = self.bucket_mut(bucket)?;
        let Some(position) = slot.iter().position(|&idx| idx == node_idx) else {
            return Err(AddNodeError::NodeNotPresentInBucket { node_idx, bucket });
        };
        slot.swap_remove(position);
        Ok(())
    }

    fn insert(&mut self, bucket: i64, node_idx: usize) -> Result<(), AddNodeError> {
        let slot = self.bucket_mut(bucket)?;
        slot.push(node_idx);
        Ok(())
    }

    /// C-style underflow/overflow bucket clamping.
    #[inline]
    pub fn clamp_bucket_index(&self, cost: i64) -> i64 {
        if cost < self.minind {
            self.minind
        } else if cost < self.maxind {
            cost
        } else {
            self.maxind
        }
    }

    /// Return the closest node available in the bucket queue.
    ///
    /// This is the idiomatic Rust equivalent of C's `ClosestNode()`.
    /// It scans buckets from `curr` upward to `maxind`, pops one node from
    /// the first non-empty bucket, marks it as `OnTree`, and returns its
    /// index. Returns `None` if no node is left.
    pub fn closest_node(&mut self, nodes: &mut [TreeNode]) -> Option<usize> {
        while self.curr <= self.maxind {
            let offset = (self.curr - self.minind) as usize;
            let Some(node_idx) = self.slots[offset].pop() else {
                self.curr += 1;
                continue;
            };
            if let Some(node) = nodes.get_mut(node_idx) {
                node.group = TreeNodeGroup::OnTree;
                node.bucket_index = None;
                return Some(node_idx);
            }
            // Invalid node index should not happen with consistent callers.
            return None;
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddNodeError {
    InvalidBucketBounds {
        minind: i64,
        maxind: i64,
    },
    InvalidNodeIndex {
        index: usize,
        len: usize,
    },
    ArcOutOfBounds {
        arcrow: usize,
        arccol: usize,
    },
    BucketOutOfRange {
        bucket: i64,
        minind: i64,
        maxind: i64,
    },
    NodeNotPresentInBucket {
        node_idx: usize,
        bucket: i64,
    },
}

pub const ONTREE_GROUP: i32 = -1;
pub const INBUCKET_GROUP: i32 = -2;
pub const NOTINBUCKET_GROUP: i32 = -3;
pub const PRUNED_GROUP: i32 = -4;
pub const BOUNDARY_ROW: i64 = -4;
pub const BOUNDARY_PTR_GROUP: i32 = -6;

/// Node state used by region scans (`ScanRegion` / `CheckBoundary`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionTraversalNode {
    pub row: i64,
    pub col: usize,
    pub group: i32,
    pub level: i32,
}

/// Graph arc used by region scans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionTraversalArc {
    pub to: usize,
    pub arcrow: usize,
    pub arccol: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionScanError {
    StartNodeMasked,
    MissingMagnitude,
    InconsistentConnectedCount,
    InconsistentBoundaryArcCount,
    InvalidBoundaryNodeCount,
}

/// Arc metadata required by `check_leaf`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafArcStatus {
    pub neighbor_group: i32,
    pub poscost: i16,
    pub negcost: i16,
    pub flow: i16,
}

/// Grid node coordinate used by region-neighbor scans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridNodeCoord {
    pub row: usize,
    pub col: usize,
}

/// Region-neighbor relation and corresponding arc index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionNeighbor {
    pub node: GridNodeCoord,
    pub arcrow: usize,
    pub arccol: usize,
}

/// Adds a node to the bucket queue if needed, updating predecessor/outcost.
///
/// This is the idiomatic Rust equivalent of the C `AddNewNode()` function.
/// The update rule is preserved exactly:
/// - compute `newoutcost = from.outcost + GetCost(...)`
/// - update when `newoutcost < to.outcost` OR `to.pred == from`
/// - if `to` is already in a bucket, remove it first
/// - reinsert into an in-range, underflow, or overflow bucket
/// - update `curr` following the C underflow/in-range rules
pub fn add_new_node(
    from_idx: usize,
    to_idx: usize,
    arcdir: i64,
    nodes: &mut [TreeNode],
    buckets: &mut FrontierBuckets,
    incrcosts: &[Vec<IncrCost>],
    arcrow: usize,
    arccol: usize,
) -> Result<(), AddNodeError> {
    let len = nodes.len();
    let from = *nodes.get(from_idx).ok_or(AddNodeError::InvalidNodeIndex {
        index: from_idx,
        len,
    })?;
    let to = *nodes
        .get(to_idx)
        .ok_or(AddNodeError::InvalidNodeIndex { index: to_idx, len })?;
    let arc_cost = incrcosts
        .get(arcrow)
        .and_then(|row| row.get(arccol))
        .ok_or(AddNodeError::ArcOutOfBounds { arcrow, arccol })?;
    let newoutcost = from.outcost + i64::from(arc_cost.get_cost(arcdir));

    if !(newoutcost < to.outcost || to.pred == Some(from_idx)) {
        return Ok(());
    }

    if to.group == TreeNodeGroup::InBucket {
        // Match C semantics by deriving the prior bucket from the previous
        // outcost with underflow/overflow clamping.
        let old_bucket = buckets.clamp_bucket_index(to.outcost);
        buckets.remove(old_bucket, to_idx)?;
        nodes[to_idx].bucket_index = None;
    }

    nodes[to_idx].outcost = newoutcost;
    nodes[to_idx].pred = Some(from_idx);
    let new_bucket = buckets.clamp_bucket_index(newoutcost);
    buckets.insert(new_bucket, to_idx)?;
    nodes[to_idx].bucket_index = Some(new_bucket);
    nodes[to_idx].group = TreeNodeGroup::InBucket;

    // Keep the original strict comparisons from C.
    if newoutcost < buckets.maxind {
        if newoutcost > buckets.minind {
            if newoutcost < buckets.curr {
                buckets.curr = newoutcost;
            }
        } else {
            buckets.curr = buckets.minind;
        }
    }

    Ok(())
}

/// Return the neighboring grid node for the requested arc number.
///
/// This is the idiomatic Rust equivalent of C `RegionsNeighborNode()`.
/// `arcnum` is incremented like the C pointer argument.
pub fn regions_neighbor_node(
    node: GridNodeCoord,
    arcnum: &mut i64,
    nrow: usize,
    ncol: usize,
) -> Option<RegionNeighbor> {
    loop {
        let current = *arcnum;
        *arcnum += 1;
        match current {
            0 => {
                if node.col != ncol - 1 {
                    return Some(RegionNeighbor {
                        node: GridNodeCoord {
                            row: node.row,
                            col: node.col + 1,
                        },
                        arcrow: nrow - 1 + node.row,
                        arccol: node.col,
                    });
                }
            }
            1 => {
                if node.row != nrow - 1 {
                    return Some(RegionNeighbor {
                        node: GridNodeCoord {
                            row: node.row + 1,
                            col: node.col,
                        },
                        arcrow: node.row,
                        arccol: node.col,
                    });
                }
            }
            2 => {
                if node.col != 0 {
                    return Some(RegionNeighbor {
                        node: GridNodeCoord {
                            row: node.row,
                            col: node.col - 1,
                        },
                        arcrow: nrow - 1 + node.row,
                        arccol: node.col - 1,
                    });
                }
            }
            3 => {
                if node.row != 0 {
                    return Some(RegionNeighbor {
                        node: GridNodeCoord {
                            row: node.row - 1,
                            col: node.col,
                        },
                        arcrow: node.row - 1,
                        arccol: node.col,
                    });
                }
            }
            _ => return None,
        }
    }
}

/// Check whether a tree node should be pruned.
///
/// This is the idiomatic Rust equivalent of C `CheckLeaf()`.
pub fn check_leaf(
    node_level: i32,
    next_level: i32,
    arcs: &[LeafArcStatus],
    prune_cost_thresh: i16,
) -> bool {
    // Not a leaf if next threaded node is a child.
    if next_level > node_level {
        return false;
    }

    for arc in arcs {
        if arc.neighbor_group == 0
            || arc.neighbor_group == INBUCKET_GROUP
            || arc.poscost < prune_cost_thresh
            || arc.negcost < prune_cost_thresh
            || arc.flow != 0
        {
            return false;
        }
    }
    true
}

/// Find all connected nodes in a region and update node groups.
///
/// This is the idiomatic Rust equivalent of C `ScanRegion()`. The caller
/// supplies explicit graph adjacency (`adjacency`), while region membership
/// still follows SNAPHU's arc test (`is_region_arc`).
pub fn scan_region(
    start_idx: usize,
    nodes: &mut [RegionTraversalNode],
    adjacency: &[Vec<RegionTraversalArc>],
    mag: Option<&[f32]>,
    nground_arcs: i64,
    nrow: usize,
    ncol: usize,
    groupsetting: i32,
) -> Result<usize, RegionScanError> {
    let _ = nground_arcs; // Kept for parity with the C API surface.
    use std::collections::VecDeque;

    let mut queue = VecDeque::new();
    let mut visited = Vec::new();
    nodes[start_idx].group = INBUCKET_GROUP;
    queue.push_back(start_idx);

    while let Some(node_idx) = queue.pop_front() {
        for arc in &adjacency[node_idx] {
            let neighbor_idx = arc.to;
            if nodes[neighbor_idx].group == BOUNDARY_PTR_GROUP {
                nodes[neighbor_idx].group = 0;
            }
            if is_region_arc(mag, arc.arcrow, arc.arccol, nrow, ncol)
                && nodes[neighbor_idx].group != ONTREE_GROUP
                && nodes[neighbor_idx].group != INBUCKET_GROUP
            {
                nodes[neighbor_idx].group = INBUCKET_GROUP;
                queue.push_back(neighbor_idx);
            }
        }

        nodes[node_idx].group = ONTREE_GROUP;
        if groupsetting == ONTREE_GROUP {
            nodes[node_idx].level = 0;
        }
        visited.push(node_idx);
    }

    if groupsetting != ONTREE_GROUP {
        for &node_idx in &visited {
            for arc in &adjacency[node_idx] {
                let neighbor_idx = arc.to;
                if nodes[neighbor_idx].group != ONTREE_GROUP {
                    if groupsetting == MASKED {
                        nodes[neighbor_idx].group = MASKED;
                    } else if groupsetting == 0 {
                        let mag = mag.ok_or(RegionScanError::MissingMagnitude)?;
                        nodes[neighbor_idx].group = if nodes[neighbor_idx].row == GROUNDROW {
                            ground_mask_status(mag, nrow, ncol)
                        } else {
                            grid_node_mask_status(
                                nodes[neighbor_idx].row as usize,
                                nodes[neighbor_idx].col,
                                mag,
                                ncol,
                            )
                        };
                    }
                }
            }
        }
        for &node_idx in &visited {
            nodes[node_idx].group = 0;
        }
    }

    Ok(visited.len())
}

/// Validate boundary connectivity and reset traversal groups.
///
/// This is the idiomatic Rust equivalent of C `CheckBoundary()`.
pub fn check_boundary(
    start_idx: usize,
    nodes: &mut [RegionTraversalNode],
    adjacency: &[Vec<RegionTraversalArc>],
    expected_boundary_neighbor_count: usize,
) -> Result<usize, RegionScanError> {
    use std::collections::VecDeque;

    if nodes[start_idx].group == MASKED {
        return Err(RegionScanError::StartNodeMasked);
    }

    let mut queue = VecDeque::new();
    let mut connected = Vec::new();
    nodes[start_idx].group = INBUCKET_GROUP;
    queue.push_back(start_idx);

    while let Some(node_idx) = queue.pop_front() {
        for arc in &adjacency[node_idx] {
            let nidx = arc.to;
            if nodes[nidx].group != MASKED
                && nodes[nidx].group != ONTREE_GROUP
                && nodes[nidx].group != INBUCKET_GROUP
            {
                nodes[nidx].group = INBUCKET_GROUP;
                queue.push_back(nidx);
            }
        }
        nodes[node_idx].group = ONTREE_GROUP;
        connected.push(node_idx);
    }

    let nconnected = connected.len();
    let mut nontree = 0usize;
    let mut nboundary_arc = 0usize;
    let mut nboundary_node = 0usize;
    for &node_idx in &connected {
        for arc in &adjacency[node_idx] {
            if nodes[arc.to].row == BOUNDARY_ROW {
                nboundary_arc += 1;
            }
        }
        if nodes[node_idx].row == BOUNDARY_ROW {
            nboundary_node += 1;
        }
        nontree += 1;
        if nodes[node_idx].group == ONTREE_GROUP {
            nodes[node_idx].group = 0;
        }
    }

    if nontree != nconnected {
        return Err(RegionScanError::InconsistentConnectedCount);
    }
    if nboundary_arc != expected_boundary_neighbor_count {
        return Err(RegionScanError::InconsistentBoundaryArcCount);
    }
    if nboundary_node != 1 {
        return Err(RegionScanError::InvalidBoundaryNodeCount);
    }
    Ok(nconnected)
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

    // --- AddNewNode ---

    fn make_incrcost_grid(pos: i16, neg: i16) -> Vec<Vec<IncrCost>> {
        vec![vec![IncrCost::new(pos, neg)]]
    }

    #[test]
    fn add_new_node_inserts_and_updates_curr_for_in_range_cost() {
        let mut nodes = vec![TreeNode::new(10), TreeNode::new(100)];
        let mut buckets = FrontierBuckets::new(0, 50, 20).unwrap();

        add_new_node(
            0,
            1,
            1,
            &mut nodes,
            &mut buckets,
            &make_incrcost_grid(5, -5),
            0,
            0,
        )
        .unwrap();

        assert_eq!(nodes[1].outcost, 15);
        assert_eq!(nodes[1].pred, Some(0));
        assert_eq!(nodes[1].group, TreeNodeGroup::InBucket);
        assert_eq!(nodes[1].bucket_index(), Some(15));
        assert_eq!(buckets.curr, 15);
    }

    #[test]
    fn add_new_node_noop_when_not_improved_and_pred_differs() {
        let mut nodes = vec![TreeNode::new(10), TreeNode::new(12)];
        nodes[1].pred = Some(99);
        let mut buckets = FrontierBuckets::new(0, 50, 20).unwrap();

        add_new_node(
            0,
            1,
            1,
            &mut nodes,
            &mut buckets,
            &make_incrcost_grid(5, -5),
            0,
            0,
        )
        .unwrap();

        assert_eq!(nodes[1].outcost, 12);
        assert_eq!(nodes[1].pred, Some(99));
        assert_eq!(nodes[1].group, TreeNodeGroup::NotInBucket);
        assert_eq!(nodes[1].bucket_index(), None);
    }

    #[test]
    fn add_new_node_forces_update_when_pred_matches_from() {
        let mut nodes = vec![TreeNode::new(10), TreeNode::new(12)];
        nodes[1].pred = Some(0);
        nodes[1].group = TreeNodeGroup::InBucket;
        nodes[1].bucket_index = Some(12);

        let mut buckets = FrontierBuckets::new(0, 50, 20).unwrap();
        buckets.insert(12, 1).unwrap();

        add_new_node(
            0,
            1,
            1,
            &mut nodes,
            &mut buckets,
            &make_incrcost_grid(5, -5),
            0,
            0,
        )
        .unwrap();

        assert_eq!(nodes[1].outcost, 15);
        assert_eq!(nodes[1].pred, Some(0));
        assert_eq!(nodes[1].bucket_index(), Some(15));
    }

    #[test]
    fn add_new_node_underflow_clamps_to_min_bucket_and_moves_curr() {
        let mut nodes = vec![TreeNode::new(5), TreeNode::new(99)];
        let mut buckets = FrontierBuckets::new(10, 100, 80).unwrap();

        add_new_node(
            0,
            1,
            -1,
            &mut nodes,
            &mut buckets,
            &make_incrcost_grid(1, -20),
            0,
            0,
        )
        .unwrap();

        assert_eq!(nodes[1].outcost, -15);
        assert_eq!(nodes[1].bucket_index(), Some(10));
        assert_eq!(buckets.curr, 10);
    }

    #[test]
    fn add_new_node_overflow_clamps_to_max_bucket_without_curr_change() {
        let mut nodes = vec![TreeNode::new(90), TreeNode::new(200)];
        let mut buckets = FrontierBuckets::new(0, 100, 30).unwrap();

        add_new_node(
            0,
            1,
            1,
            &mut nodes,
            &mut buckets,
            &make_incrcost_grid(30, -30),
            0,
            0,
        )
        .unwrap();

        assert_eq!(nodes[1].outcost, 120);
        assert_eq!(nodes[1].bucket_index(), Some(100));
        assert_eq!(buckets.curr, 30);
    }

    #[test]
    fn add_new_node_reinsert_removes_from_clamped_old_bucket() {
        let mut nodes = vec![TreeNode::new(30), TreeNode::new(-3)];
        nodes[1].group = TreeNodeGroup::InBucket;
        nodes[1].bucket_index = Some(0);
        let mut buckets = FrontierBuckets::new(0, 100, 50).unwrap();
        buckets.insert(0, 1).unwrap();

        add_new_node(
            0,
            1,
            -1,
            &mut nodes,
            &mut buckets,
            &make_incrcost_grid(10, -40),
            0,
            0,
        )
        .unwrap();

        assert_eq!(nodes[1].outcost, -10);
        assert_eq!(nodes[1].bucket_index(), Some(0));
        let min_slot = buckets.bucket_mut(0).unwrap();
        assert_eq!(min_slot.len(), 1);
        assert_eq!(min_slot[0], 1);
    }

    #[test]
    fn closest_node_returns_first_available_bucket_node() {
        let mut nodes = vec![TreeNode::new(0), TreeNode::new(0), TreeNode::new(0)];
        let mut buckets = FrontierBuckets::new(0, 5, 0).unwrap();
        buckets.insert(2, 1).unwrap();
        buckets.insert(4, 2).unwrap();

        let idx = buckets.closest_node(&mut nodes).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(nodes[idx].group, TreeNodeGroup::OnTree);
        assert_eq!(buckets.curr, 2);
    }

    #[test]
    fn closest_node_returns_none_when_no_nodes_left() {
        let mut nodes = vec![TreeNode::new(0)];
        let mut buckets = FrontierBuckets::new(0, 2, 0).unwrap();
        assert_eq!(buckets.closest_node(&mut nodes), None);
        assert_eq!(buckets.curr, 3);
    }

    #[test]
    fn regions_neighbor_node_scans_right_down_left_up() {
        let node = GridNodeCoord { row: 2, col: 3 };
        let mut arcnum = 0;

        let right = regions_neighbor_node(node, &mut arcnum, 5, 6).unwrap();
        assert_eq!(right.node, GridNodeCoord { row: 2, col: 4 });
        assert_eq!(right.arcrow, 6);
        assert_eq!(right.arccol, 3);

        let down = regions_neighbor_node(node, &mut arcnum, 5, 6).unwrap();
        assert_eq!(down.node, GridNodeCoord { row: 3, col: 3 });
        assert_eq!(down.arcrow, 2);
        assert_eq!(down.arccol, 3);

        let left = regions_neighbor_node(node, &mut arcnum, 5, 6).unwrap();
        assert_eq!(left.node, GridNodeCoord { row: 2, col: 2 });
        assert_eq!(left.arcrow, 6);
        assert_eq!(left.arccol, 2);

        let up = regions_neighbor_node(node, &mut arcnum, 5, 6).unwrap();
        assert_eq!(up.node, GridNodeCoord { row: 1, col: 3 });
        assert_eq!(up.arcrow, 1);
        assert_eq!(up.arccol, 3);

        assert_eq!(regions_neighbor_node(node, &mut arcnum, 5, 6), None);
    }

    #[test]
    fn regions_neighbor_node_respects_boundaries() {
        let mut arcnum = 0;
        let top_left = GridNodeCoord { row: 0, col: 0 };
        let n = regions_neighbor_node(top_left, &mut arcnum, 3, 3).unwrap();
        assert_eq!(n.node, GridNodeCoord { row: 0, col: 1 });
        let n = regions_neighbor_node(top_left, &mut arcnum, 3, 3).unwrap();
        assert_eq!(n.node, GridNodeCoord { row: 1, col: 0 });
        assert_eq!(regions_neighbor_node(top_left, &mut arcnum, 3, 3), None);
    }

    #[test]
    fn check_leaf_requires_leaf_and_high_cost_zero_flow_neighbors() {
        let arcs = vec![LeafArcStatus {
            neighbor_group: ONTREE_GROUP,
            poscost: 50,
            negcost: 60,
            flow: 0,
        }];
        assert!(check_leaf(3, 3, &arcs, 40));
        assert!(!check_leaf(3, 4, &arcs, 40)); // has child in thread
        assert!(!check_leaf(
            3,
            3,
            &[LeafArcStatus {
                poscost: 10,
                ..arcs[0]
            }],
            40
        ));
        assert!(!check_leaf(
            3,
            3,
            &[LeafArcStatus { flow: 1, ..arcs[0] }],
            40
        ));
    }

    #[test]
    fn scan_region_marks_connected_region_and_resets_group_on_cleanup_mode() {
        // 2x2 nodes, all connected through valid region arcs.
        let mut nodes = vec![
            RegionTraversalNode {
                row: 0,
                col: 0,
                group: 0,
                level: 9,
            },
            RegionTraversalNode {
                row: 0,
                col: 1,
                group: 0,
                level: 9,
            },
            RegionTraversalNode {
                row: 1,
                col: 0,
                group: 0,
                level: 9,
            },
            RegionTraversalNode {
                row: 1,
                col: 1,
                group: 0,
                level: 9,
            },
        ];
        let adjacency = vec![
            vec![
                RegionTraversalArc {
                    to: 1,
                    arcrow: 2,
                    arccol: 0,
                },
                RegionTraversalArc {
                    to: 2,
                    arcrow: 0,
                    arccol: 0,
                },
            ],
            vec![
                RegionTraversalArc {
                    to: 0,
                    arcrow: 2,
                    arccol: 0,
                },
                RegionTraversalArc {
                    to: 3,
                    arcrow: 1,
                    arccol: 0,
                },
            ],
            vec![
                RegionTraversalArc {
                    to: 0,
                    arcrow: 0,
                    arccol: 0,
                },
                RegionTraversalArc {
                    to: 3,
                    arcrow: 2,
                    arccol: 1,
                },
            ],
            vec![
                RegionTraversalArc {
                    to: 1,
                    arcrow: 1,
                    arccol: 0,
                },
                RegionTraversalArc {
                    to: 2,
                    arcrow: 2,
                    arccol: 1,
                },
            ],
        ];
        let mag = vec![1.0f32; 9]; // 3x3 pixels around 2x2 nodes

        let n = scan_region(0, &mut nodes, &adjacency, Some(&mag), 0, 3, 3, 0).unwrap();
        assert_eq!(n, 4);
        assert!(nodes.iter().all(|n| n.group == 0));
    }

    #[test]
    fn check_boundary_counts_boundary_arcs_and_nodes() {
        // node 0 is boundary row node; node 1 is regular node.
        let mut nodes = vec![
            RegionTraversalNode {
                row: BOUNDARY_ROW,
                col: 0,
                group: 0,
                level: 0,
            },
            RegionTraversalNode {
                row: 0,
                col: 0,
                group: 0,
                level: 0,
            },
        ];
        let adjacency = vec![
            vec![RegionTraversalArc {
                to: 1,
                arcrow: 0,
                arccol: 0,
            }],
            vec![RegionTraversalArc {
                to: 0,
                arcrow: 0,
                arccol: 0,
            }],
        ];

        let n = check_boundary(1, &mut nodes, &adjacency, 1).unwrap();
        assert_eq!(n, 2);
        assert_eq!(nodes[0].group, 0);
        assert_eq!(nodes[1].group, 0);
    }
}
