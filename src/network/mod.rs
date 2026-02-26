#![allow(dead_code)]

//! Network graph construction and solver wrappers.

pub mod bucket;

use crate::constants::{GROUNDROW, LARGE_SHORT, MASKED};
use crate::costs::types::{Cost, IncrCost};
use crate::data::ops::{cycle_residue, l_round, node_residue, short_2d_row_col_abs_max};
use cost_scaling_rs::McmfCs2;

pub struct TileGraph;

pub trait MinCostFlowSolver {
    fn solve(&mut self, graph: &TileGraph) -> Result<(), String>;
}

/// Selected topology-specific neighbor/arc mapping behavior.
///
/// This is the typed Rust equivalent of the C global function-pointer switches
/// controlled by `SetGridNetworkFunctionPointers()` and
/// `SetNonGridNetworkFunctionPointers()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkFunctionPointers {
    Grid,
    NonGrid,
}

/// Select grid-network neighbor/arc handlers.
///
/// Equivalent to C `SetGridNetworkFunctionPointers()`.
pub fn set_grid_network_function_pointers() -> NetworkFunctionPointers {
    NetworkFunctionPointers::Grid
}

/// Select non-grid (secondary) network neighbor/arc handlers.
///
/// Equivalent to C `SetNonGridNetworkFunctionPointers()`.
pub fn set_non_grid_network_function_pointers() -> NetworkFunctionPointers {
    NetworkFunctionPointers::NonGrid
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
    Normal,
    NotInBucket,
    InBucket,
    OnTree,
    Pruned,
    Masked,
}

/// Minimal node view needed by `add_new_node()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeNode {
    pub row: i64,
    pub col: i64,
    pub level: i64,
    pub incost: i64,
    pub outcost: i64,
    pub pred: Option<usize>,
    pub group: TreeNodeGroup,
    bucket_index: Option<i64>,
}

impl TreeNode {
    pub fn new(outcost: i64) -> Self {
        Self {
            row: 0,
            col: 0,
            level: 0,
            incost: VERY_FAR,
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
pub const GROUND_COL: i64 = -2;
pub const VERY_FAR: i64 = 2_000_000_000;
pub const CLIP_FACTOR: f64 = 0.666_666_666_7;
pub const INIT_MAX_COST_INCR: i64 = 200;
pub const MAX_RESIDUE: i64 = i8::MAX as i64;
pub const MIN_RESIDUE: i64 = i8::MIN as i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDataError {
    InvalidDimensions {
        nrow: usize,
        ncol: usize,
    },
    InvalidMagnitudeRows {
        expected: usize,
        got: usize,
    },
    InvalidMagnitudeCols {
        row: usize,
        expected: usize,
        got: usize,
    },
    InvalidNodeRows {
        expected: usize,
        got: usize,
    },
    InvalidNodeCols {
        row: usize,
        expected: usize,
        got: usize,
    },
    InvalidFlowRows {
        expected: usize,
        got: usize,
    },
    InvalidFlowCols {
        row: usize,
        expected: usize,
        got: usize,
    },
    InvalidResidueRows {
        expected: usize,
        got: usize,
    },
    InvalidResidueCols {
        row: usize,
        expected: usize,
        got: usize,
    },
    InvalidNodeIndex {
        index: usize,
        len: usize,
    },
    CostOutOfRange {
        value: i64,
    },
    FlowOutOfRange {
        value: i64,
    },
    ResidueOverflow {
        row: usize,
        col: usize,
        value: i64,
    },
}

fn validate_dims(nrow: usize, ncol: usize) -> Result<(), NetworkDataError> {
    if nrow == 0 || ncol == 0 {
        return Err(NetworkDataError::InvalidDimensions { nrow, ncol });
    }
    Ok(())
}

fn validate_mag_dims(mag: &[Vec<f32>], nrow: usize, ncol: usize) -> Result<(), NetworkDataError> {
    if mag.len() != nrow {
        return Err(NetworkDataError::InvalidMagnitudeRows {
            expected: nrow,
            got: mag.len(),
        });
    }
    for (row, vals) in mag.iter().enumerate() {
        if vals.len() != ncol {
            return Err(NetworkDataError::InvalidMagnitudeCols {
                row,
                expected: ncol,
                got: vals.len(),
            });
        }
    }
    Ok(())
}

fn validate_node_grid_dims(
    nodes: &[Vec<TreeNode>],
    nrow: usize,
    ncol: usize,
) -> Result<(), NetworkDataError> {
    if nodes.len() != nrow {
        return Err(NetworkDataError::InvalidNodeRows {
            expected: nrow,
            got: nodes.len(),
        });
    }
    for (row, vals) in nodes.iter().enumerate() {
        if vals.len() != ncol {
            return Err(NetworkDataError::InvalidNodeCols {
                row,
                expected: ncol,
                got: vals.len(),
            });
        }
    }
    Ok(())
}

fn validate_flow_dims(arr: &[Vec<i16>], nrow: usize, ncol: usize) -> Result<(), NetworkDataError> {
    let expected_rows = 2 * nrow - 1;
    if arr.len() != expected_rows {
        return Err(NetworkDataError::InvalidFlowRows {
            expected: expected_rows,
            got: arr.len(),
        });
    }
    for (row, vals) in arr.iter().enumerate() {
        let expected_cols = if row < nrow - 1 { ncol } else { ncol - 1 };
        if vals.len() != expected_cols {
            return Err(NetworkDataError::InvalidFlowCols {
                row,
                expected: expected_cols,
                got: vals.len(),
            });
        }
    }
    Ok(())
}

fn validate_residue_dims(
    residue: &[Vec<i8>],
    nrow: usize,
    ncol: usize,
) -> Result<(), NetworkDataError> {
    let expected_rows = nrow - 1;
    if residue.len() != expected_rows {
        return Err(NetworkDataError::InvalidResidueRows {
            expected: expected_rows,
            got: residue.len(),
        });
    }
    for (row, vals) in residue.iter().enumerate() {
        let expected_cols = ncol - 1;
        if vals.len() != expected_cols {
            return Err(NetworkDataError::InvalidResidueCols {
                row,
                expected: expected_cols,
                got: vals.len(),
            });
        }
    }
    Ok(())
}

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
#[allow(clippy::too_many_arguments)]
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

/// Check whether all pixels are masked (`<= 0` magnitude).
///
/// Equivalent to C `CheckMagMasking()`. Returns `true` only when every pixel
/// is masked; returns `false` as soon as an unmasked (`> 0`) pixel is found.
pub fn check_mag_masking(
    mag: &[Vec<f32>],
    nrow: usize,
    ncol: usize,
) -> Result<bool, NetworkDataError> {
    validate_dims(nrow, ncol)?;
    validate_mag_dims(mag, nrow, ncol)?;

    for row_vals in mag.iter().take(nrow) {
        for &value in row_vals.iter().take(ncol) {
            if value > 0.0 {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Set node groups based on magnitude masking for grid and ground nodes.
///
/// Equivalent to C `MaskNodes()`. Grid nodes become `Masked` only when all
/// four surrounding pixels are zero; otherwise they become `Normal`. The
/// ground node becomes `Masked` only when all boundary pixels are zero.
pub fn mask_nodes(
    nrow: usize,
    ncol: usize,
    nodes: &mut [Vec<TreeNode>],
    ground: &mut TreeNode,
    mag: &[Vec<f32>],
) -> Result<(), NetworkDataError> {
    validate_dims(nrow, ncol)?;
    validate_mag_dims(mag, nrow, ncol)?;
    validate_node_grid_dims(nodes, nrow - 1, ncol - 1)?;

    for row in 0..(nrow - 1) {
        for col in 0..(ncol - 1) {
            let masked = mag[row][col] == 0.0
                && mag[row][col + 1] == 0.0
                && mag[row + 1][col] == 0.0
                && mag[row + 1][col + 1] == 0.0;
            nodes[row][col].group = if masked {
                TreeNodeGroup::Masked
            } else {
                TreeNodeGroup::Normal
            };
        }
    }

    let mut ground_masked = true;
    for row_vals in mag.iter().take(nrow) {
        if row_vals[0] != 0.0 || row_vals[ncol - 1] != 0.0 {
            ground_masked = false;
            break;
        }
    }
    if ground_masked {
        for value in mag[0].iter().take(ncol) {
            if *value != 0.0 {
                ground_masked = false;
                break;
            }
        }
    }
    if ground_masked {
        for value in mag[nrow - 1].iter().take(ncol) {
            if *value != 0.0 {
                ground_masked = false;
                break;
            }
        }
    }
    ground.group = if ground_masked {
        TreeNodeGroup::Masked
    } else {
        TreeNodeGroup::Normal
    };

    Ok(())
}

/// Return the maximum absolute flow that does not touch masked pixels.
///
/// Equivalent to C `MaxNonMaskFlow()`.
#[allow(clippy::needless_range_loop)]
pub fn max_non_mask_flow(
    flows: &[Vec<i16>],
    mag: &[Vec<f32>],
    nrow: usize,
    ncol: usize,
) -> Result<i64, NetworkDataError> {
    validate_dims(nrow, ncol)?;
    validate_flow_dims(flows, nrow, ncol)?;
    validate_mag_dims(mag, nrow, ncol)?;

    let mut mostflow = 0i64;

    for row in 0..(nrow - 1) {
        for col in 0..ncol {
            let flowvalue = i64::from(flows[row][col]).abs();
            if flowvalue > mostflow && mag[row][col] > 0.0 && mag[row + 1][col] > 0.0 {
                mostflow = flowvalue;
            }
        }
    }

    for row in (nrow - 1)..(2 * nrow - 1) {
        for col in 0..(ncol - 1) {
            let flowvalue = i64::from(flows[row][col]).abs();
            let mag_row = row - (nrow - 1);
            if flowvalue > mostflow && mag[mag_row][col] > 0.0 && mag[mag_row][col + 1] > 0.0 {
                mostflow = flowvalue;
            }
        }
    }

    Ok(mostflow)
}

/// Initialize node `(row, col)` coordinates for the grid and optional ground.
///
/// Equivalent to C `InitNodeNums()`.
pub fn init_node_nums(
    nrow: usize,
    ncol: usize,
    nodes: &mut [Vec<TreeNode>],
    ground: Option<&mut TreeNode>,
) -> Result<(), NetworkDataError> {
    validate_node_grid_dims(nodes, nrow, ncol)?;

    for (row, row_nodes) in nodes.iter_mut().enumerate().take(nrow) {
        for (col, node) in row_nodes.iter_mut().enumerate().take(ncol) {
            node.row = row as i64;
            node.col = col as i64;
        }
    }

    if let Some(ground) = ground {
        ground.row = GROUNDROW;
        ground.col = GROUND_COL;
    }

    Ok(())
}

/// Reset node tree state before a solve pass.
///
/// Equivalent to C `InitNodes()`.
pub fn init_nodes(
    nrow: usize,
    ncol: usize,
    nodes: &mut [Vec<TreeNode>],
    ground: Option<&mut TreeNode>,
) -> Result<(), NetworkDataError> {
    validate_node_grid_dims(nodes, nrow, ncol)?;

    for row_nodes in nodes.iter_mut().take(nrow) {
        for node in row_nodes.iter_mut().take(ncol) {
            node.group = TreeNodeGroup::NotInBucket;
            node.incost = VERY_FAR;
            node.outcost = VERY_FAR;
            node.pred = None;
            node.bucket_index = None;
        }
    }

    if let Some(ground) = ground {
        ground.group = TreeNodeGroup::NotInBucket;
        ground.incost = VERY_FAR;
        ground.outcost = VERY_FAR;
        ground.pred = None;
        ground.bucket_index = None;
    }

    Ok(())
}

/// Initialize bucket storage and insert a source node at the first bucket.
///
/// Equivalent to C `InitBuckets()`.
pub fn init_buckets(
    buckets: &mut FrontierBuckets,
    nodes: &mut [TreeNode],
    source_idx: usize,
) -> Result<(), AddNodeError> {
    let len = nodes.len();
    let source = nodes
        .get_mut(source_idx)
        .ok_or(AddNodeError::InvalidNodeIndex {
            index: source_idx,
            len,
        })?;

    for slot in &mut buckets.slots {
        slot.clear();
    }

    buckets.curr = buckets.minind;
    buckets.insert(buckets.minind, source_idx)?;

    source.group = TreeNodeGroup::InBucket;
    source.outcost = 0;
    source.bucket_index = Some(buckets.minind);

    Ok(())
}

/// Return a common ancestor (cycle apex) for two tree nodes.
///
/// Equivalent to C `FindApex()`.
pub fn find_apex(nodes: &[TreeNode], mut from_idx: usize, mut to_idx: usize) -> Option<usize> {
    if from_idx >= nodes.len() || to_idx >= nodes.len() {
        return None;
    }

    if nodes[from_idx].level > nodes[to_idx].level {
        while nodes[from_idx].level != nodes[to_idx].level {
            from_idx = nodes[from_idx].pred?;
        }
    } else {
        while nodes[from_idx].level != nodes[to_idx].level {
            to_idx = nodes[to_idx].pred?;
        }
    }

    while from_idx != to_idx {
        from_idx = nodes[from_idx].pred?;
        to_idx = nodes[to_idx].pred?;
    }
    Some(from_idx)
}

/// Remove and return the minimum-outcost node from the current bucket window.
///
/// Equivalent to C `MinOutCostNode()`. If no node is available, returns `None`.
pub fn min_out_cost_node(
    buckets: &mut FrontierBuckets,
    nodes: &mut [TreeNode],
) -> Result<Option<usize>, AddNodeError> {
    while buckets.curr < buckets.maxind {
        let offset = (buckets.curr - buckets.minind) as usize;
        if !buckets.slots[offset].is_empty() {
            break;
        }
        buckets.curr += 1;
    }

    if buckets.curr > buckets.maxind {
        return Ok(None);
    }

    let bucket = buckets.curr;
    let node_idx = if bucket == buckets.minind || bucket == buckets.maxind {
        let slot = buckets.bucket_mut(bucket)?;
        let Some(&first_idx) = slot.first() else {
            return Ok(None);
        };
        if first_idx >= nodes.len() {
            return Err(AddNodeError::InvalidNodeIndex {
                index: first_idx,
                len: nodes.len(),
            });
        }
        let mut best_pos = 0usize;
        let mut best_cost = nodes[first_idx].outcost;
        for (pos, &candidate_idx) in slot.iter().enumerate().skip(1) {
            if candidate_idx >= nodes.len() {
                return Err(AddNodeError::InvalidNodeIndex {
                    index: candidate_idx,
                    len: nodes.len(),
                });
            }
            let candidate_cost = nodes[candidate_idx].outcost;
            if candidate_cost < best_cost {
                best_cost = candidate_cost;
                best_pos = pos;
            }
        }
        slot.swap_remove(best_pos)
    } else {
        let slot = buckets.bucket_mut(bucket)?;
        let Some(node_idx) = slot.pop() else {
            return Ok(None);
        };
        node_idx
    };

    if node_idx >= nodes.len() {
        return Err(AddNodeError::InvalidNodeIndex {
            index: node_idx,
            len: nodes.len(),
        });
    }
    nodes[node_idx].bucket_index = None;
    Ok(Some(node_idx))
}

fn apply_residue_delta(
    residue: &mut [Vec<i8>],
    row: usize,
    col: usize,
    delta: i64,
) -> Result<(), NetworkDataError> {
    let value = i64::from(residue[row][col]) + delta;
    if !(MIN_RESIDUE..=MAX_RESIDUE).contains(&value) {
        return Err(NetworkDataError::ResidueOverflow { row, col, value });
    }
    residue[row][col] = value as i8;
    Ok(())
}

/// Clip flows, update residue pushes, and raise clipped-arc costs.
///
/// Equivalent to C `ClipFlow()`. Returns:
/// - `Ok(true)` when no rerun is needed (already below limit or max-cost guard)
/// - `Ok(false)` when clipping occurred and the solver should rerun.
#[allow(clippy::needless_range_loop)]
pub fn clip_flow(
    residue: &mut [Vec<i8>],
    flows: &mut [Vec<i16>],
    mstcosts: &mut [Vec<i16>],
    nrow: usize,
    ncol: usize,
    maxflow: i64,
) -> Result<bool, NetworkDataError> {
    validate_dims(nrow, ncol)?;
    validate_residue_dims(residue, nrow, ncol)?;
    validate_flow_dims(flows, nrow, ncol)?;
    validate_flow_dims(mstcosts, nrow, ncol)?;

    let mut mostflow = 0i64;
    for row in 0..(2 * nrow - 1) {
        let maxcol = if row < nrow - 1 { ncol } else { ncol - 1 };
        for col in 0..maxcol {
            mostflow = mostflow.max(i64::from(flows[row][col]).abs());
        }
    }

    if mostflow <= maxflow {
        return Ok(true);
    }

    let mut cliplimit = (f64::ceil(mostflow as f64 * CLIP_FACTOR) as i64) + 1;
    if maxflow > cliplimit {
        cliplimit = maxflow;
    }

    let mut maxcost = 0i64;
    for row in 0..(2 * nrow - 1) {
        let maxcol = if row < nrow - 1 { ncol } else { ncol - 1 };
        for col in 0..maxcol {
            let cost = i64::from(mstcosts[row][col]);
            if cost > maxcost && cost < i64::from(LARGE_SHORT) {
                maxcost = cost;
            }
        }
    }

    maxcost += INIT_MAX_COST_INCR;
    if maxcost >= i64::from(LARGE_SHORT) {
        return Ok(true);
    }
    let clipped_cost =
        i16::try_from(maxcost).map_err(|_| NetworkDataError::CostOutOfRange { value: maxcost })?;

    for row in 0..(2 * nrow - 1) {
        let maxcol = if row < nrow - 1 { ncol } else { ncol - 1 };
        for col in 0..maxcol {
            let current = i64::from(flows[row][col]);
            if current.abs() <= cliplimit {
                continue;
            }
            let (sign, excess) = if current > 0 {
                (1i64, current - cliplimit)
            } else {
                (-1i64, current + cliplimit)
            };

            if row < nrow - 1 {
                if col != 0 {
                    apply_residue_delta(residue, row, col - 1, excess)?;
                }
                if col != ncol - 1 {
                    apply_residue_delta(residue, row, col, -excess)?;
                }
            } else {
                if row != nrow - 1 {
                    apply_residue_delta(residue, row - nrow, col, excess)?;
                }
                if row != 2 * nrow - 2 {
                    apply_residue_delta(residue, row - nrow + 1, col, -excess)?;
                }
            }

            let clipped_flow = sign * cliplimit;
            flows[row][col] =
                i16::try_from(clipped_flow).map_err(|_| NetworkDataError::FlowOutOfRange {
                    value: clipped_flow,
                })?;
            mstcosts[row][col] = clipped_cost;
        }
    }

    Ok(false)
}

/// Remove all nodes from buckets and reset bucket/node state.
///
/// Equivalent to C `ClearBuckets()`.
pub fn clear_buckets(
    buckets: &mut FrontierBuckets,
    nodes: &mut [TreeNode],
) -> Result<(), AddNodeError> {
    for slot in &mut buckets.slots {
        for &node_idx in slot.iter() {
            let len = nodes.len();
            let node = nodes
                .get_mut(node_idx)
                .ok_or(AddNodeError::InvalidNodeIndex {
                    index: node_idx,
                    len,
                })?;
            node.group = TreeNodeGroup::NotInBucket;
            node.outcost = VERY_FAR;
            node.pred = None;
            node.bucket_index = None;
        }
        slot.clear();
    }

    buckets.minind = 0;
    buckets.maxind = buckets.slots.len() as i64 - 1;
    buckets.curr = 0;

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
#[allow(clippy::too_many_arguments)]
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

/// Cost-mode selector used by `calc_init_max_flow()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostMode {
    NoStatCosts,
    Topo,
    Defo,
    Smooth,
}

/// Parameters required by `calc_init_max_flow()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitMaxFlowParams {
    pub initmaxflow: i64,
    pub costmode: CostMode,
    pub nshortcycle: i64,
    pub arcmaxflowconst: i64,
}

/// Candidate entering-arc metadata (`CheckArcReducedCost` equivalent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateArc {
    pub violation: i64,
    pub from: usize,
    pub to: usize,
    pub arcrow: usize,
    pub arccol: usize,
    pub arcdir: i64,
}

/// Summary stats returned by `setup_incr_flow_costs()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetupIncrFlowCostsResult {
    pub narcs: usize,
    pub clipped_cost_count: usize,
    pub clipped_fraction: f64,
}

/// Errors from translated network-cost helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkCostError {
    InvalidNodeIndex {
        index: usize,
        len: usize,
    },
    MissingArc {
        arcrow: usize,
        arccol: usize,
    },
    InvalidRowCount {
        expected: usize,
        got: usize,
    },
    InvalidRowLen {
        row: usize,
        expected: usize,
        got: usize,
    },
    MissingNarcsPerRow,
    InvalidNShortCycle {
        nshortcycle: i64,
    },
    MissingCostArray,
    InvalidRegionGrid,
    InvalidRegionSource {
        row: usize,
        col: usize,
    },
    InvalidRegionSizeIndex {
        index: usize,
        len: usize,
    },
    RegionScanFailed,
    AddNodeFailure(AddNodeError),
    NetworkDataFailure(NetworkDataError),
    InvalidNetworkDims {
        nrow: usize,
        ncol: usize,
    },
    FlowShortCycleOverflow {
        mostflow: i64,
        nshortcycle: i64,
    },
    InvalidAdjacency {
        expected: usize,
        got: usize,
    },
    InvalidApexLayout,
    InvalidCandidateLayout,
    InvalidSourceIndex {
        index: usize,
        len: usize,
    },
    SolverFailure(String),
    SolverArcMappingMissing {
        tail: usize,
        head: usize,
    },
}

const NOSTAT_INIT_MAX_FLOW: i64 = 15;
const DEF_INIT_MAX_FLOW: i64 = 9_999;
const INIT_ARR_SIZE: usize = 500;
const N_SOURCE_LIST_MEM_INCR: usize = 1024;
const NEG_BUCKET_FRACTION: f64 = 1.0;
const POS_BUCKET_FRACTION: f64 = 1.0;

/// Input parameters for network initialization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NetworkInitParams {
    pub nrow: usize,
    pub ncol: usize,
    pub nshortcycle: i64,
    pub maxcost: f64,
    pub has_ground: bool,
}

/// Runtime state initialized by `init_network`.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInitState {
    pub ngroundarcs: i64,
    pub ncycle: i64,
    pub nflowdone: i64,
    pub mostflow: i64,
    pub nflow: i64,
    pub candidate_bag: Vec<CandidateArc>,
    pub candidate_list: Vec<CandidateArc>,
    pub iscandidate: Vec<Vec<bool>>,
    pub apexes: Vec<Vec<Option<usize>>>,
    pub buckets: FrontierBuckets,
    pub iincrcostfile: i64,
    pub incrcosts: Vec<Vec<IncrCost>>,
    pub nnoderow: usize,
    pub nnodes_per_row: Vec<usize>,
    pub narcrow: usize,
    pub narcs_per_row: Vec<usize>,
    pub notfirstloop: bool,
    pub totalcost: f64,
}

/// Parameters for connected-source selection (`SelectSources` family).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSelectionParams {
    pub nground_arcs: i64,
    pub nrow: usize,
    pub ncol: usize,
    pub nconnnodemin: usize,
}

/// One accepted source and the size of its connected component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceCandidate {
    pub node_idx: usize,
    pub connected_count: usize,
}

/// Source list produced by `select_sources`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSelection {
    pub sources: Vec<usize>,
    pub connected_sizes: Vec<usize>,
}

/// Grid-arc descriptor used while initializing tree buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeArc {
    pub to: usize,
    pub arcrow: usize,
    pub arccol: usize,
    pub arcdir: i64,
}

/// Edge walk descriptor for boundary discharge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryPathArc {
    pub node_row: usize,
    pub node_col: usize,
    pub arcrow: usize,
    pub arccol: usize,
    pub arcdir: i64,
}

#[inline]
fn convert_data_error(err: NetworkDataError) -> NetworkCostError {
    NetworkCostError::NetworkDataFailure(err)
}

#[inline]
fn convert_add_node_error(err: AddNodeError) -> NetworkCostError {
    NetworkCostError::AddNodeFailure(err)
}

fn flow_row_lengths(nrow: usize, ncol: usize) -> Vec<usize> {
    let mut widths = Vec::with_capacity(2 * nrow - 1);
    for row in 0..(2 * nrow - 1) {
        widths.push(if row < nrow - 1 { ncol } else { ncol - 1 });
    }
    widths
}

fn validate_flow_layout(
    flows: &[Vec<i16>],
    nrow: usize,
    ncol: usize,
) -> Result<(), NetworkCostError> {
    let widths = flow_row_lengths(nrow, ncol);
    if flows.len() != widths.len() {
        return Err(NetworkCostError::InvalidRowCount {
            expected: widths.len(),
            got: flows.len(),
        });
    }
    for (row, (vals, width)) in flows.iter().zip(widths.iter()).enumerate() {
        if vals.len() != *width {
            return Err(NetworkCostError::InvalidRowLen {
                row,
                expected: *width,
                got: vals.len(),
            });
        }
    }
    Ok(())
}

fn flatten_row_col_layout(flows: &[Vec<i16>], nrow: usize, ncol: usize) -> Vec<i16> {
    let mut flat = Vec::with_capacity((nrow - 1) * ncol + nrow * (ncol - 1));
    for row in flows {
        flat.extend_from_slice(row);
    }
    flat
}

fn cycle_residue_grid(phase: &[f32], nrow: usize, ncol: usize) -> Vec<Vec<i8>> {
    let flat = cycle_residue(phase, nrow, ncol);
    let mut out = Vec::with_capacity(nrow - 1);
    for row in 0..(nrow - 1) {
        let start = row * (ncol - 1);
        let end = start + (ncol - 1);
        out.push(flat[start..end].to_vec());
    }
    out
}

/// Initialize tree-solver runtime buffers and bookkeeping values.
///
/// This is the idiomatic Rust equivalent of C `InitNetwork()`.
pub fn init_network(
    flows: &mut [Vec<i16>],
    params: NetworkInitParams,
) -> Result<NetworkInitState, NetworkCostError> {
    if params.nrow < 2 || params.ncol < 2 {
        return Err(NetworkCostError::InvalidNetworkDims {
            nrow: params.nrow,
            ncol: params.ncol,
        });
    }
    validate_flow_layout(flows, params.nrow, params.ncol)?;

    if params.has_ground {
        flows[0][0] = flows[0][0].wrapping_add(flows[params.nrow - 1][0]);
        flows[params.nrow - 1][0] = 0;
        flows[0][params.ncol - 1] =
            flows[0][params.ncol - 1].wrapping_sub(flows[params.nrow - 1][params.ncol - 2]);
        flows[params.nrow - 1][params.ncol - 2] = 0;

        flows[params.nrow - 2][0] =
            flows[params.nrow - 2][0].wrapping_sub(flows[2 * params.nrow - 2][0]);
        flows[2 * params.nrow - 2][0] = 0;
        flows[params.nrow - 2][params.ncol - 1] = flows[params.nrow - 2][params.ncol - 1]
            .wrapping_add(flows[2 * params.nrow - 2][params.ncol - 2]);
        flows[2 * params.nrow - 2][params.ncol - 2] = 0;
    }

    let flat = flatten_row_col_layout(flows, params.nrow, params.ncol);
    let mostflow = if params.has_ground {
        short_2d_row_col_abs_max(&flat, params.nrow, params.ncol)
    } else {
        0
    };
    if params.has_ground && mostflow.saturating_mul(params.nshortcycle) > i64::from(LARGE_SHORT) {
        return Err(NetworkCostError::FlowShortCycleOverflow {
            mostflow,
            nshortcycle: params.nshortcycle,
        });
    }

    let ngroundarcs = if params.has_ground {
        if params.ncol > 2 {
            2 * (params.nrow as i64 + params.ncol as i64 - 2) - 4
        } else {
            2 * (params.nrow as i64 + params.ncol as i64 - 2) - 2
        }
    } else {
        0
    };

    let span = if params.has_ground {
        params.nrow as f64 + params.ncol as f64
    } else {
        params.nrow as f64
    };
    let minind = -l_round((params.maxcost + 1.0) * span * NEG_BUCKET_FRACTION);
    let maxind = l_round((params.maxcost + 1.0) * span * POS_BUCKET_FRACTION);
    let buckets = FrontierBuckets::new(minind, maxind, minind).map_err(convert_add_node_error)?;

    let narcs_per_row = flow_row_lengths(params.nrow, params.ncol);
    let apexes = if params.has_ground {
        narcs_per_row.iter().map(|&w| vec![None; w]).collect()
    } else {
        Vec::new()
    };
    let iscandidate = if params.has_ground {
        narcs_per_row.iter().map(|&w| vec![false; w]).collect()
    } else {
        Vec::new()
    };
    let incrcosts = if params.has_ground {
        narcs_per_row
            .iter()
            .map(|&w| vec![IncrCost::default(); w])
            .collect()
    } else {
        Vec::new()
    };

    let (nnoderow, nnodes_per_row, narcrow) = if params.has_ground {
        (
            params.nrow - 1,
            vec![params.ncol - 1; params.nrow - 1],
            2 * params.nrow - 1,
        )
    } else {
        (0, Vec::new(), 0)
    };

    Ok(NetworkInitState {
        ngroundarcs,
        ncycle: 0,
        nflowdone: 0,
        mostflow,
        nflow: 1,
        candidate_bag: Vec::with_capacity(INIT_ARR_SIZE),
        candidate_list: Vec::with_capacity(INIT_ARR_SIZE),
        iscandidate,
        apexes,
        buckets,
        iincrcostfile: 0,
        incrcosts,
        nnoderow,
        nnodes_per_row,
        narcrow,
        narcs_per_row,
        notfirstloop: false,
        totalcost: crate::constants::LARGE_FLOAT,
    })
}

/// Reset node labels and candidate/arc state before each tree solve pass.
///
/// This is the idiomatic Rust equivalent of C `SetupTreeSolveNetwork()`.
pub fn setup_tree_solve_network(
    nodes: &mut [Vec<TreeNode>],
    ground: Option<&mut TreeNode>,
    apexes: &mut [Vec<Option<usize>>],
    iscandidate: &mut [Vec<bool>],
    nnodes_per_row: &[usize],
    narcs_per_row: &[usize],
    dims: (usize, usize),
) -> Result<usize, NetworkCostError> {
    let (nrow, ncol) = dims;
    if nodes.len() != nnodes_per_row.len() {
        return Err(NetworkCostError::InvalidRowCount {
            expected: nnodes_per_row.len(),
            got: nodes.len(),
        });
    }
    let mut nnodes = 0usize;
    for (row, row_nodes) in nodes.iter_mut().enumerate() {
        if row_nodes.len() < nnodes_per_row[row] {
            return Err(NetworkCostError::InvalidRowLen {
                row,
                expected: nnodes_per_row[row],
                got: row_nodes.len(),
            });
        }
        for node in row_nodes.iter_mut().take(nnodes_per_row[row]) {
            if node.group != TreeNodeGroup::Masked {
                node.group = TreeNodeGroup::Normal;
                nnodes += 1;
            }
            node.incost = VERY_FAR;
            node.outcost = VERY_FAR;
            node.pred = None;
        }
    }

    let has_ground = ground.is_some();
    if let Some(ground) = ground {
        if ground.group != TreeNodeGroup::Masked {
            ground.group = TreeNodeGroup::Normal;
            nnodes += 1;
        }
        ground.incost = VERY_FAR;
        ground.outcost = VERY_FAR;
        ground.pred = None;
    }

    if apexes.len() != narcs_per_row.len() || iscandidate.len() != narcs_per_row.len() {
        return Err(NetworkCostError::InvalidApexLayout);
    }
    for row in 0..narcs_per_row.len() {
        if apexes[row].len() != narcs_per_row[row] || iscandidate[row].len() != narcs_per_row[row] {
            return Err(NetworkCostError::InvalidCandidateLayout);
        }
        for col in 0..narcs_per_row[row] {
            apexes[row][col] = None;
            iscandidate[row][col] = false;
        }
    }

    if has_ground {
        iscandidate[nrow - 1][0] = true;
        iscandidate[2 * nrow - 2][0] = true;
        iscandidate[nrow - 1][ncol - 2] = true;
        iscandidate[2 * nrow - 2][ncol - 2] = true;
    }

    Ok(nnodes)
}

/// Select source for one connected component, starting at `start_idx`.
///
/// This is the idiomatic Rust equivalent of C `SelectConnNodeSource()`.
pub fn select_conn_node_source(
    nodes: &mut [RegionTraversalNode],
    adjacency: &[Vec<RegionTraversalArc>],
    mag: Option<&[f32]>,
    start_idx: usize,
    params: SourceSelectionParams,
) -> Result<Option<SourceCandidate>, NetworkCostError> {
    if start_idx >= nodes.len() {
        return Err(NetworkCostError::InvalidSourceIndex {
            index: start_idx,
            len: nodes.len(),
        });
    }
    if nodes[start_idx].group == MASKED || nodes[start_idx].group == ONTREE_GROUP {
        return Ok(None);
    }
    let nconnected = scan_region(
        start_idx,
        nodes,
        adjacency,
        mag,
        params.nground_arcs,
        params.nrow,
        params.ncol,
        ONTREE_GROUP,
    )
    .map_err(|_| NetworkCostError::RegionScanFailed)?;

    if nconnected > params.nconnnodemin {
        Ok(Some(SourceCandidate {
            node_idx: start_idx,
            connected_count: nconnected,
        }))
    } else {
        Ok(None)
    }
}

/// Enumerate source nodes for each connected unmasked component.
///
/// This is the idiomatic Rust equivalent of C `SelectSources()`.
pub fn select_sources(
    nodes: &mut [RegionTraversalNode],
    adjacency: &[Vec<RegionTraversalArc>],
    mag: Option<&[f32]>,
    ground_idx: Option<usize>,
    params: SourceSelectionParams,
) -> Result<SourceSelection, NetworkCostError> {
    if adjacency.len() != nodes.len() {
        return Err(NetworkCostError::InvalidAdjacency {
            expected: nodes.len(),
            got: adjacency.len(),
        });
    }
    for node in nodes.iter_mut() {
        if node.group != MASKED && node.group != BOUNDARY_PTR_GROUP {
            node.group = 0;
        }
    }

    let mut out = SourceSelection {
        sources: Vec::with_capacity(N_SOURCE_LIST_MEM_INCR),
        connected_sizes: Vec::with_capacity(N_SOURCE_LIST_MEM_INCR),
    };

    if let Some(ground_idx) = ground_idx
        && let Some(src) = select_conn_node_source(nodes, adjacency, mag, ground_idx, params)?
    {
        out.sources.push(src.node_idx);
        out.connected_sizes.push(src.connected_count);
    }

    for idx in 0..nodes.len() {
        if Some(idx) == ground_idx {
            continue;
        }
        if let Some(src) = select_conn_node_source(nodes, adjacency, mag, idx, params)? {
            out.sources.push(src.node_idx);
            out.connected_sizes.push(src.connected_count);
        }
    }

    for node in nodes.iter_mut() {
        if node.group != MASKED && node.group != BOUNDARY_PTR_GROUP {
            node.group = 0;
        }
    }

    Ok(out)
}

/// Initialize one tree root and seed buckets with outgoing candidate nodes.
///
/// This is the idiomatic Rust equivalent of C `InitTree()`.
pub fn init_tree(
    source_idx: usize,
    nodes: &mut [TreeNode],
    adjacency: &[Vec<TreeArc>],
    buckets: &mut FrontierBuckets,
    nflow: i64,
    incrcosts: &[Vec<IncrCost>],
) -> Result<(), NetworkCostError> {
    let _ = nflow;
    if source_idx >= nodes.len() {
        return Err(NetworkCostError::InvalidNodeIndex {
            index: source_idx,
            len: nodes.len(),
        });
    }
    if adjacency.len() != nodes.len() {
        return Err(NetworkCostError::InvalidAdjacency {
            expected: nodes.len(),
            got: adjacency.len(),
        });
    }

    let source = &mut nodes[source_idx];
    source.group = TreeNodeGroup::OnTree;
    source.outcost = 0;
    source.incost = 0;
    source.pred = None;
    source.level = 0;

    for arc in &adjacency[source_idx] {
        if arc.to >= nodes.len() {
            return Err(NetworkCostError::InvalidNodeIndex {
                index: arc.to,
                len: nodes.len(),
            });
        }
        if nodes[arc.to].group != TreeNodeGroup::Pruned
            && nodes[arc.to].group != TreeNodeGroup::Masked
        {
            add_new_node(
                source_idx, arc.to, arc.arcdir, nodes, buckets, incrcosts, arc.arcrow, arc.arccol,
            )
            .map_err(convert_add_node_error)?;
        }
    }
    Ok(())
}

/// Prune tree leaves whose outgoing arcs all satisfy the prune threshold.
///
/// This is the idiomatic Rust equivalent of C `PruneTree()`.
pub fn prune_tree(
    traversal_order: &[usize],
    nodes: &mut [TreeNode],
    leaf_arcs: &[Vec<LeafArcStatus>],
    prune_cost_thresh: i16,
) -> Result<usize, NetworkCostError> {
    if leaf_arcs.len() != nodes.len() {
        return Err(NetworkCostError::InvalidAdjacency {
            expected: nodes.len(),
            got: leaf_arcs.len(),
        });
    }

    let mut npruned = 0usize;
    for (i, &idx) in traversal_order.iter().enumerate().skip(1) {
        if idx >= nodes.len() {
            return Err(NetworkCostError::InvalidNodeIndex {
                index: idx,
                len: nodes.len(),
            });
        }
        let next_level = if i + 1 < traversal_order.len() {
            let next_idx = traversal_order[i + 1];
            if next_idx >= nodes.len() {
                return Err(NetworkCostError::InvalidNodeIndex {
                    index: next_idx,
                    len: nodes.len(),
                });
            }
            nodes[next_idx].level as i32
        } else {
            i32::MIN
        };
        if check_leaf(
            nodes[idx].level as i32,
            next_level,
            &leaf_arcs[idx],
            prune_cost_thresh,
        ) {
            nodes[idx].group = TreeNodeGroup::Pruned;
            npruned += 1;
        }
    }
    Ok(npruned)
}

/// Re-scan a region and restore node mask/group values after boundary setup.
///
/// This is the idiomatic Rust equivalent of C `CleanUpBoundaryNodes()`.
pub fn clean_up_boundary_nodes(
    source_idx: usize,
    boundary_neighbor_idx: Option<usize>,
    non_grid_network: bool,
    nodes: &mut [RegionTraversalNode],
    adjacency: &[Vec<RegionTraversalArc>],
    mag: Option<&[f32]>,
    params: SourceSelectionParams,
) -> Result<usize, NetworkCostError> {
    if non_grid_network {
        return Ok(0);
    }
    if source_idx >= nodes.len() {
        return Err(NetworkCostError::InvalidSourceIndex {
            index: source_idx,
            len: nodes.len(),
        });
    }

    let start_idx = if nodes[source_idx].row == BOUNDARY_ROW {
        boundary_neighbor_idx.unwrap_or(source_idx)
    } else {
        source_idx
    };
    if start_idx >= nodes.len() {
        return Err(NetworkCostError::InvalidSourceIndex {
            index: start_idx,
            len: nodes.len(),
        });
    }

    scan_region(
        start_idx,
        nodes,
        adjacency,
        mag,
        params.nground_arcs,
        params.nrow,
        params.ncol,
        0,
    )
    .map_err(|_| NetworkCostError::RegionScanFailed)
}

/// Discharge boundary-node surplus along traced zero-cost boundary arcs.
///
/// This is the idiomatic Rust equivalent of C `DischargeBoundary()`.
pub fn discharge_boundary(
    flows: &mut [Vec<i16>],
    boundary_path: &[BoundaryPathArc],
    wrapped_phase: &[f32],
    nrow: usize,
    ncol: usize,
    enabled: bool,
) -> Result<usize, NetworkCostError> {
    if !enabled || boundary_path.is_empty() {
        return Ok(0);
    }
    validate_flow_layout(flows, nrow, ncol)?;
    if wrapped_phase.len() != nrow * ncol {
        return Err(NetworkCostError::InvalidRowCount {
            expected: nrow * ncol,
            got: wrapped_phase.len(),
        });
    }

    let mut nedgenode = 1usize;
    for step in boundary_path {
        if step.arcrow >= flows.len() || step.arccol >= flows[step.arcrow].len() {
            return Err(NetworkCostError::MissingArc {
                arcrow: step.arcrow,
                arccol: step.arccol,
            });
        }
        if step.node_row >= nrow - 1 || step.node_col >= ncol - 1 {
            continue;
        }

        let surplus = i64::from(flows[step.node_row][step.node_col])
            - i64::from(flows[step.node_row][step.node_col + 1])
            + i64::from(flows[step.node_row + nrow - 1][step.node_col])
            - i64::from(flows[step.node_row + 1 + nrow - 1][step.node_col]);
        let residue = i64::from(node_residue(
            wrapped_phase,
            nrow,
            ncol,
            step.node_row,
            step.node_col,
        ));
        let excess = surplus + residue;
        let updated = i64::from(flows[step.arcrow][step.arccol]) + step.arcdir * excess;
        let updated = i16::try_from(updated).map_err(|_| {
            NetworkCostError::NetworkDataFailure(NetworkDataError::FlowOutOfRange {
                value: updated,
            })
        })?;
        flows[step.arcrow][step.arccol] = updated;
        nedgenode += 1;
    }
    Ok(nedgenode)
}

/// Initialize flows from wrapped phase residues using an MCF backend.
///
/// This is the idiomatic Rust equivalent of C `MCFInitFlows()`.
pub fn mcf_init_flows<F>(
    wrapped_phase: &[f32],
    mstcosts: &[Vec<i16>],
    nrow: usize,
    ncol: usize,
    cs2_scale_factor: i64,
    mut solve_cs2: F,
) -> Result<Vec<Vec<i16>>, NetworkCostError>
where
    F: FnMut(&[Vec<i8>], &[Vec<i16>], usize, usize, i64) -> Result<Vec<Vec<i16>>, NetworkCostError>,
{
    if wrapped_phase.len() != nrow * ncol {
        return Err(NetworkCostError::InvalidRowCount {
            expected: nrow * ncol,
            got: wrapped_phase.len(),
        });
    }
    validate_flow_layout(mstcosts, nrow, ncol)?;
    let residue = cycle_residue_grid(wrapped_phase, nrow, ncol);
    let flows = solve_cs2(&residue, mstcosts, nrow, ncol, cs2_scale_factor)?;
    validate_flow_layout(&flows, nrow, ncol)?;
    Ok(flows)
}

/// Initialize flows from wrapped phase using the MST initialization path.
///
/// This is the idiomatic Rust equivalent of C `MSTInitFlows()`.
pub fn mst_init_flows(
    wrapped_phase: &[f32],
    mstcosts: &mut [Vec<i16>],
    nrow: usize,
    ncol: usize,
    maxflow: i64,
) -> Result<Vec<Vec<i16>>, NetworkCostError> {
    if wrapped_phase.len() != nrow * ncol {
        return Err(NetworkCostError::InvalidRowCount {
            expected: nrow * ncol,
            got: wrapped_phase.len(),
        });
    }
    validate_flow_layout(mstcosts, nrow, ncol)?;

    let mut residue = cycle_residue_grid(wrapped_phase, nrow, ncol);
    let widths = flow_row_lengths(nrow, ncol);
    let mut flows: Vec<Vec<i16>> = widths.iter().map(|&w| vec![0i16; w]).collect();

    // Mirror the C loop shape: clip/retry until all flows are within bounds.
    loop {
        let done = clip_flow(&mut residue, &mut flows, mstcosts, nrow, ncol, maxflow)
            .map_err(convert_data_error)?;
        if done {
            break;
        }
    }

    Ok(flows)
}

/// Input bundle for CS2-style residue initialization.
#[derive(Debug, Clone)]
pub struct SolveCs2Params<'a> {
    pub residue: &'a [Vec<i8>],
    pub mst_costs: &'a [Vec<i16>],
    pub nrow: usize,
    pub ncol: usize,
    pub cs2_scale_factor: i64,
}

/// Typed output for MST initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveMstResult {
    pub flows: Vec<Vec<i16>>,
    pub arc_status: Vec<Vec<i8>>,
}

/// Input bundle for MST-style initialization.
#[derive(Debug, Clone)]
pub struct SolveMstParams<'a> {
    pub residue: &'a [Vec<i8>],
    pub mst_costs: &'a [Vec<i16>],
    pub nrow: usize,
    pub ncol: usize,
}

/// Mutable state consumed by `discharge_tree()`.
pub struct DischargeTreeParams<'a> {
    pub flows: &'a mut [Vec<i16>],
    pub residue: &'a mut [Vec<i8>],
    pub arc_status: &'a mut [Vec<i8>],
    pub nrow: usize,
    pub ncol: usize,
}

/// Input bundle for boundary initialization.
pub struct InitBoundaryParams<'a> {
    pub source_idx: usize,
    pub nodes: &'a mut [RegionTraversalNode],
    pub adjacency: &'a [Vec<RegionTraversalArc>],
    pub mag: Option<&'a [f32]>,
    pub nrow: usize,
    pub ncol: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitBoundaryResult {
    pub source_idx: usize,
    pub boundary_nodes: Vec<usize>,
    pub connected_count: usize,
}

/// Parameters for non-degenerate child updates on a pivot path.
pub struct NonDegenUpdateParams<'a> {
    pub path: &'a [usize],
    pub nodes: &'a mut [TreeNode],
    pub updated_group: TreeNodeGroup,
    pub dincost: i64,
    pub doutcost: i64,
}

/// Input bundle for high-level tree solve orchestration.
pub struct TreeSolveParams<'a> {
    pub source_idx: usize,
    pub nodes: &'a mut [RegionTraversalNode],
    pub adjacency: &'a [Vec<RegionTraversalArc>],
    pub mag: Option<&'a [f32]>,
    pub residue: &'a [Vec<i8>],
    pub mst_costs: &'a [Vec<i16>],
    pub nrow: usize,
    pub ncol: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSolveResult {
    pub source_idx: usize,
    pub connected_count: usize,
    pub improvements: usize,
    pub source_charge: i64,
    pub flows: Vec<Vec<i16>>,
    pub arc_status: Vec<Vec<i8>>,
}

/// Solve the residue network with a CS2-style initializer.
///
/// This is the typed Rust equivalent of C `SolveCS2()`.
pub fn solve_cs2(params: SolveCs2Params<'_>) -> Result<Vec<Vec<i16>>, NetworkCostError> {
    if params.nrow < 2 || params.ncol < 2 {
        return Err(NetworkCostError::InvalidNetworkDims {
            nrow: params.nrow,
            ncol: params.ncol,
        });
    }
    validate_residue_dims(params.residue, params.nrow, params.ncol).map_err(convert_data_error)?;
    validate_flow_dims(params.mst_costs, params.nrow, params.ncol).map_err(convert_data_error)?;
    let _scale = params.cs2_scale_factor;
    let debug_enabled = log::log_enabled!(log::Level::Debug);

    if debug_enabled {
        let mut residue_pos = 0usize;
        let mut residue_neg = 0usize;
        let mut residue_zero = 0usize;
        let mut residue_sum = 0i64;
        let mut residue_abs_sum = 0i64;
        for row in params.residue {
            for &v in row {
                let vi = i64::from(v);
                residue_sum += vi;
                residue_abs_sum += vi.unsigned_abs() as i64;
                if v > 0 {
                    residue_pos += 1;
                } else if v < 0 {
                    residue_neg += 1;
                } else {
                    residue_zero += 1;
                }
            }
        }

        let mut cost_min = i16::MAX;
        let mut cost_max = i16::MIN;
        let mut cost_sum = 0i64;
        let mut cost_zero = 0usize;
        let mut cost_count = 0usize;
        for row in params.mst_costs {
            for &c in row {
                cost_min = cost_min.min(c);
                cost_max = cost_max.max(c);
                cost_sum += i64::from(c);
                if c == 0 {
                    cost_zero += 1;
                }
                cost_count += 1;
            }
        }
        let cost_mean = if cost_count == 0 {
            0.0
        } else {
            cost_sum as f64 / cost_count as f64
        };

        log::debug!(
            "solve_cs2 begin: nrow={}, ncol={}, scale_factor={}, residue(pos={}, neg={}, zero={}, sum={}, abs_sum={}), mst_costs(count={}, min={}, max={}, mean={:.3}, zero={})",
            params.nrow,
            params.ncol,
            params.cs2_scale_factor,
            residue_pos,
            residue_neg,
            residue_zero,
            residue_sum,
            residue_abs_sum,
            cost_count,
            cost_min,
            cost_max,
            cost_mean,
            cost_zero
        );
    }

    let residue_rows = params.nrow - 1;
    let residue_cols = params.ncol - 1;
    let ground_id = residue_rows * residue_cols + 1;
    let narcs = (params.nrow - 1) * params.ncol + params.nrow * (params.ncol - 1);
    const ARC_UBOUND: i64 = 200;
    let mut solver = McmfCs2::new(ground_id, 2 * narcs);

    let node_id = |row: usize, col: usize| -> usize { col * residue_rows + row + 1 };

    let mut ground_supply = 0i64;
    for col in 0..residue_cols {
        for row in 0..residue_rows {
            let supply = i64::from(params.residue[row][col]);
            ground_supply -= supply;
            solver.set_supply_demand_of_node(node_id(row, col), supply);
        }
    }
    solver.set_supply_demand_of_node(ground_id, ground_supply);

    let mut arc_pairs_ground = 0usize;
    let mut arc_pairs_row_internal = 0usize;
    let mut arc_pairs_col_internal = 0usize;
    let row_arc_count = residue_rows * params.ncol;
    for arcctr in 1..=narcs {
        let (tail0, head0, arcrow, _arccol, cost) = if arcctr <= row_arc_count {
            let nodectr = arcctr;
            let tail = if nodectr <= residue_rows * residue_cols {
                nodectr
            } else {
                ground_id
            };
            let head = if nodectr <= residue_rows {
                ground_id
            } else {
                nodectr - residue_rows
            };
            let arcrow = (nodectr - 1) % residue_rows;
            let arccol = (nodectr - 1) / residue_rows;
            (
                tail,
                head,
                arcrow,
                arccol,
                i64::from(params.mst_costs[arcrow][arccol]),
            )
        } else {
            let nodectr = arcctr - row_arc_count;
            let denom = residue_rows + 1;
            let ceil_div = nodectr.div_ceil(denom);
            let tail = if nodectr.is_multiple_of(denom) {
                ground_id
            } else {
                nodectr - ceil_div + 1
            };
            let head = if nodectr % denom == 1 {
                ground_id
            } else {
                nodectr - ceil_div
            };
            let arcrow = params.nrow - 1 + ((nodectr - 1) % denom);
            let arccol = (nodectr - 1) / denom;
            (
                tail,
                head,
                arcrow,
                arccol,
                i64::from(params.mst_costs[arcrow][arccol]),
            )
        };

        if tail0 == ground_id || head0 == ground_id {
            arc_pairs_ground += 1;
        } else if arcrow < params.nrow - 1 {
            arc_pairs_row_internal += 1;
        } else {
            arc_pairs_col_internal += 1;
        }

        let mut tail = tail0;
        let mut head = head0;
        for _ in 0..2 {
            solver.set_arc(tail, head, 0, ARC_UBOUND, cost);
            std::mem::swap(&mut tail, &mut head);
        }
    }

    if debug_enabled {
        log::debug!(
            "solve_cs2 network: nodes={} (including ground), undirected_arc_pairs={} (ground={}, row_internal={}, col_internal={}), directed_arcs={}",
            ground_id,
            narcs,
            arc_pairs_ground,
            arc_pairs_row_internal,
            arc_pairs_col_internal,
            2 * narcs
        );
    }

    let solution = solver
        .min_cost(false, false)
        .map_err(|err| NetworkCostError::SolverFailure(format!("{err:?}")))?;

    let widths = flow_row_lengths(params.nrow, params.ncol);
    let mut flows: Vec<Vec<i16>> = widths.iter().map(|&w| vec![0i16; w]).collect();
    let mut add_flow = |arcrow: usize, arccol: usize, delta: i64| -> Result<(), NetworkCostError> {
        let value = i64::from(flows[arcrow][arccol]).saturating_add(delta);
        flows[arcrow][arccol] = i16::try_from(value).map_err(|_| {
            NetworkCostError::NetworkDataFailure(NetworkDataError::FlowOutOfRange { value })
        })?;
        Ok(())
    };

    for (mut from, mut to, flow) in solution.flows() {
        if flow <= 0 {
            continue;
        }
        let mut f = flow;

        if from == ground_id || to == ground_id {
            if to == ground_id {
                std::mem::swap(&mut from, &mut to);
                f = -f;
            }
            if (to - 1) % residue_rows == 0 {
                let c = (to - 1) / residue_rows;
                add_flow(params.nrow - 1, c, f)?;
            } else if to <= residue_rows {
                add_flow(to - 1, 0, f)?;
            } else if to >= (ground_id - residue_rows - 1) {
                let r = (to - 1) % residue_rows;
                add_flow(r, residue_cols, -f)?;
            } else if to % residue_rows == 0 {
                let c = to / residue_rows - 1;
                add_flow(params.nrow - 1 + residue_rows, c, -f)?;
            } else {
                return Err(NetworkCostError::SolverArcMappingMissing {
                    tail: from,
                    head: to,
                });
            }
        } else if from == to + 1 {
            let num = from + (from - 1) / residue_rows;
            let arcrow = params.nrow - 1 + (num - 1) % (residue_rows + 1);
            let arccol = (num - 1) / (residue_rows + 1);
            add_flow(arcrow, arccol, -f)?;
        } else if from + 1 == to {
            let num = from + (from - 1) / residue_rows + 1;
            let arcrow = params.nrow - 1 + (num - 1) % (residue_rows + 1);
            let arccol = (num - 1) / (residue_rows + 1);
            add_flow(arcrow, arccol, f)?;
        } else if from + residue_rows == to {
            let num = from + residue_rows;
            let arcrow = (num - 1) % residue_rows;
            let arccol = (num - 1) / residue_rows;
            add_flow(arcrow, arccol, f)?;
        } else if from == to + residue_rows {
            let num = from;
            let arcrow = (num - 1) % residue_rows;
            let arccol = (num - 1) / residue_rows;
            add_flow(arcrow, arccol, -f)?;
        } else {
            return Err(NetworkCostError::SolverArcMappingMissing {
                tail: from,
                head: to,
            });
        }
    }

    if debug_enabled {
        let mut flow_nonzero = 0usize;
        let mut flow_abs_sum = 0i64;
        let mut flow_min = i16::MAX;
        let mut flow_max = i16::MIN;
        let mut row_nonzero = 0usize;
        let mut col_nonzero = 0usize;
        let mut objective_abs = 0i64;

        for (arcrow, flow_row) in flows.iter().enumerate() {
            for (arccol, &f) in flow_row.iter().enumerate() {
                if f != 0 {
                    flow_nonzero += 1;
                    if arcrow < params.nrow - 1 {
                        row_nonzero += 1;
                    } else {
                        col_nonzero += 1;
                    }
                }
                flow_abs_sum += i64::from(f).unsigned_abs() as i64;
                flow_min = flow_min.min(f);
                flow_max = flow_max.max(f);
                objective_abs += i64::from(f).unsigned_abs() as i64
                    * i64::from(params.mst_costs[arcrow][arccol]).unsigned_abs() as i64;
            }
        }

        let mut residue_mismatch_count = 0usize;
        let mut residue_mismatch_max_abs = 0i64;
        let mut residue_mismatch_sum_abs = 0i64;
        for row in 0..(params.nrow - 1) {
            for col in 0..(params.ncol - 1) {
                let row_left = i64::from(flows[row][col]);
                let row_right = i64::from(flows[row][col + 1]);
                let col_top = i64::from(flows[params.nrow - 1 + row][col]);
                let col_bottom = i64::from(flows[params.nrow - 1 + row + 1][col]);
                let calc_residue = row_left + col_top - row_right - col_bottom;
                let input_residue = i64::from(params.residue[row][col]);
                let delta = calc_residue - input_residue;
                if delta != 0 {
                    residue_mismatch_count += 1;
                    let d = delta.unsigned_abs() as i64;
                    residue_mismatch_sum_abs += d;
                    residue_mismatch_max_abs = residue_mismatch_max_abs.max(d);
                }
            }
        }

        log::debug!(
            "solve_cs2 flow summary: nonzero={} (row={}, col={}), abs_sum={}, min={}, max={}, abs_objective={}",
            flow_nonzero,
            row_nonzero,
            col_nonzero,
            flow_abs_sum,
            flow_min,
            flow_max,
            objective_abs
        );
        log::debug!(
            "solve_cs2 residue closure: mismatch_count={}, mismatch_max_abs={}, mismatch_sum_abs={}",
            residue_mismatch_count,
            residue_mismatch_max_abs,
            residue_mismatch_sum_abs
        );
    }

    Ok(flows)
}

/// Build an MST-like spanning flow and tree-arc status map.
///
/// This is the typed Rust equivalent of C `SolveMST()`.
pub fn solve_mst(params: SolveMstParams<'_>) -> Result<SolveMstResult, NetworkCostError> {
    let flows = solve_cs2(SolveCs2Params {
        residue: params.residue,
        mst_costs: params.mst_costs,
        nrow: params.nrow,
        ncol: params.ncol,
        cs2_scale_factor: 1,
    })?;

    let mut arc_status = flow_row_lengths(params.nrow, params.ncol)
        .into_iter()
        .map(|w| vec![0i8; w])
        .collect::<Vec<_>>();
    for row in 0..arc_status.len() {
        for col in 0..arc_status[row].len() {
            if flows[row][col] != 0 || params.mst_costs[row][col] == 0 {
                arc_status[row][col] = -1;
            }
        }
    }

    Ok(SolveMstResult { flows, arc_status })
}

/// Discharge tree charges back into the flow field.
///
/// This is the typed Rust equivalent of C `DischargeTree()`.
pub fn discharge_tree(params: DischargeTreeParams<'_>) -> Result<i64, NetworkCostError> {
    fn validate_arc_status_dims(
        arr: &[Vec<i8>],
        nrow: usize,
        ncol: usize,
    ) -> Result<(), NetworkCostError> {
        let widths = flow_row_lengths(nrow, ncol);
        if arr.len() != widths.len() {
            return Err(NetworkCostError::InvalidRowCount {
                expected: widths.len(),
                got: arr.len(),
            });
        }
        for (row, &w) in widths.iter().enumerate() {
            if arr[row].len() != w {
                return Err(NetworkCostError::InvalidRowLen {
                    row,
                    expected: w,
                    got: arr[row].len(),
                });
            }
        }
        Ok(())
    }

    validate_flow_layout(params.flows, params.nrow, params.ncol)?;
    validate_arc_status_dims(params.arc_status, params.nrow, params.ncol)?;
    validate_residue_dims(params.residue, params.nrow, params.ncol).map_err(convert_data_error)?;

    let mut source_charge = 0i64;
    for row in 0..(params.nrow - 1) {
        for col in 0..(params.ncol - 1) {
            let charge = i64::from(params.residue[row][col]);
            source_charge -= charge;
            if charge != 0 {
                let updated = i64::from(params.flows[row][col]) + charge;
                params.flows[row][col] = i16::try_from(updated).map_err(|_| {
                    NetworkCostError::NetworkDataFailure(NetworkDataError::FlowOutOfRange {
                        value: updated,
                    })
                })?;
                params.residue[row][col] = 0;
            }
        }
    }

    for row in 0..params.arc_status.len() {
        for col in 0..params.arc_status[row].len() {
            if params.arc_status[row][col] == -1 {
                params.arc_status[row][col] = -3;
            }
        }
    }

    Ok(source_charge)
}

/// Initialize boundary nodes for one connected source component.
///
/// This is the typed Rust equivalent of C `InitBoundary()`.
pub fn init_boundary(
    params: InitBoundaryParams<'_>,
) -> Result<InitBoundaryResult, NetworkCostError> {
    if params.source_idx >= params.nodes.len() {
        return Err(NetworkCostError::InvalidSourceIndex {
            index: params.source_idx,
            len: params.nodes.len(),
        });
    }
    if params.adjacency.len() != params.nodes.len() {
        return Err(NetworkCostError::InvalidAdjacency {
            expected: params.nodes.len(),
            got: params.adjacency.len(),
        });
    }
    if let Some(mag) = params.mag
        && mag.len() != params.nrow * params.ncol
    {
        return Err(NetworkCostError::InvalidRowCount {
            expected: params.nrow * params.ncol,
            got: mag.len(),
        });
    }

    let mut connected = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    let mut visited = vec![false; params.nodes.len()];
    visited[params.source_idx] = true;
    queue.push_back(params.source_idx);

    while let Some(idx) = queue.pop_front() {
        connected.push(idx);
        for arc in &params.adjacency[idx] {
            if arc.to >= params.nodes.len() {
                return Err(NetworkCostError::InvalidNodeIndex {
                    index: arc.to,
                    len: params.nodes.len(),
                });
            }
            if params.nodes[arc.to].group == MASKED || visited[arc.to] {
                continue;
            }
            visited[arc.to] = true;
            queue.push_back(arc.to);
        }
    }

    let mut boundary_nodes = Vec::new();
    for &idx in &connected {
        let node = params.nodes[idx];
        if node.row < 0 {
            continue;
        }
        let row = node.row as usize;
        if row + 1 >= params.nrow || node.col + 1 >= params.ncol {
            continue;
        }
        if is_region_edge_node(params.mag, node.row, node.col, params.nrow, params.ncol) {
            boundary_nodes.push(idx);
        }
    }

    if boundary_nodes.len() >= 3 {
        for &idx in &boundary_nodes {
            params.nodes[idx].group = BOUNDARY_PTR_GROUP;
        }
    }

    let source_idx = if boundary_nodes.contains(&params.source_idx) || boundary_nodes.is_empty() {
        params.source_idx
    } else {
        boundary_nodes[0]
    };

    Ok(InitBoundaryResult {
        source_idx,
        boundary_nodes,
        connected_count: connected.len(),
    })
}

/// Update a path of child nodes during a non-degenerate pivot.
///
/// This is the typed Rust equivalent of C `NonDegenUpdateChildren()`.
pub fn non_degen_update_children(
    params: NonDegenUpdateParams<'_>,
) -> Result<usize, NetworkCostError> {
    if params.path.len() < 2 {
        return Ok(0);
    }
    for &idx in params.path {
        if idx >= params.nodes.len() {
            return Err(NetworkCostError::InvalidNodeIndex {
                index: idx,
                len: params.nodes.len(),
            });
        }
    }

    for &idx in &params.path[1..] {
        let node = &mut params.nodes[idx];
        node.group = params.updated_group;
        node.incost = node.incost.saturating_add(params.dincost);
        node.outcost = node.outcost.saturating_add(params.doutcost);
    }
    Ok(params.path.len() - 1)
}

/// Solve one tree pass with boundary setup + MST + discharge.
///
/// This is the typed Rust equivalent of C `TreeSolve()`.
pub fn tree_solve(params: TreeSolveParams<'_>) -> Result<TreeSolveResult, NetworkCostError> {
    let boundary = init_boundary(InitBoundaryParams {
        source_idx: params.source_idx,
        nodes: params.nodes,
        adjacency: params.adjacency,
        mag: params.mag,
        nrow: params.nrow,
        ncol: params.ncol,
    })?;

    let mst = solve_mst(SolveMstParams {
        residue: params.residue,
        mst_costs: params.mst_costs,
        nrow: params.nrow,
        ncol: params.ncol,
    })?;

    let mut flows = mst.flows;
    let mut arc_status = mst.arc_status;
    let mut residue = params.residue.to_vec();
    let source_charge = discharge_tree(DischargeTreeParams {
        flows: &mut flows,
        residue: &mut residue,
        arc_status: &mut arc_status,
        nrow: params.nrow,
        ncol: params.ncol,
    })?;

    let improvements = flows.iter().flatten().filter(|&&f| f != 0).count();
    Ok(TreeSolveResult {
        source_idx: boundary.source_idx,
        connected_count: boundary.connected_count,
        improvements,
        source_charge,
        flows,
        arc_status,
    })
}

/// Recompute one arc's incremental +/- cost and clip it to `LARGE_SHORT`.
///
/// This is the idiomatic Rust equivalent of C `ReCalcCost()`.
pub fn recalc_cost<F>(
    incrcosts: &mut [Vec<IncrCost>],
    flow: i64,
    arcrow: usize,
    arccol: usize,
    nflow: i64,
    nrow: usize,
    mut calc_cost: F,
) -> Result<usize, NetworkCostError>
where
    F: FnMut(i64, usize, usize, i64, usize) -> (i64, i64),
{
    let row = incrcosts
        .get_mut(arcrow)
        .ok_or(NetworkCostError::MissingArc { arcrow, arccol })?;
    let arc = row
        .get_mut(arccol)
        .ok_or(NetworkCostError::MissingArc { arcrow, arccol })?;

    let (poscost, negcost) = calc_cost(flow, arcrow, arccol, nflow, nrow);

    let mut clipped = 0usize;
    if poscost > i64::from(LARGE_SHORT) {
        arc.poscost = LARGE_SHORT;
        clipped += 1;
    } else if poscost < -i64::from(LARGE_SHORT) {
        arc.poscost = -LARGE_SHORT;
        clipped += 1;
    } else {
        arc.poscost = poscost as i16;
    }

    if negcost > i64::from(LARGE_SHORT) {
        arc.negcost = LARGE_SHORT;
        clipped += 1;
    } else if negcost < -i64::from(LARGE_SHORT) {
        arc.negcost = -LARGE_SHORT;
        clipped += 1;
    } else {
        arc.negcost = negcost as i16;
    }

    Ok(clipped)
}

/// Recompute incremental costs for all arcs using the current flow field.
///
/// This is the idiomatic Rust equivalent of C `SetupIncrFlowCosts()`.
pub fn setup_incr_flow_costs<F>(
    incrcosts: &mut [Vec<IncrCost>],
    flows: &[Vec<i16>],
    nflow: i64,
    nrow: usize,
    narcs_per_row: &[usize],
    mut calc_cost: F,
) -> Result<SetupIncrFlowCostsResult, NetworkCostError>
where
    F: FnMut(i64, usize, usize, i64, usize) -> (i64, i64),
{
    if incrcosts.len() != narcs_per_row.len() || flows.len() != narcs_per_row.len() {
        return Err(NetworkCostError::InvalidRowCount {
            expected: narcs_per_row.len(),
            got: incrcosts.len().max(flows.len()),
        });
    }

    let mut narcs = 0usize;
    let mut clipped_cost_count = 0usize;
    for arcrow in 0..narcs_per_row.len() {
        let expected = narcs_per_row[arcrow];
        if incrcosts[arcrow].len() != expected {
            return Err(NetworkCostError::InvalidRowLen {
                row: arcrow,
                expected,
                got: incrcosts[arcrow].len(),
            });
        }
        if flows[arcrow].len() != expected {
            return Err(NetworkCostError::InvalidRowLen {
                row: arcrow,
                expected,
                got: flows[arcrow].len(),
            });
        }
        narcs += expected;
        for (arccol, &flow) in flows[arcrow].iter().take(expected).enumerate() {
            clipped_cost_count += recalc_cost(
                incrcosts,
                i64::from(flow),
                arcrow,
                arccol,
                nflow,
                nrow,
                &mut calc_cost,
            )?;
        }
    }

    let clipped_fraction = if narcs == 0 {
        0.0
    } else {
        clipped_cost_count as f64 / (2.0 * narcs as f64)
    };
    Ok(SetupIncrFlowCostsResult {
        narcs,
        clipped_cost_count,
        clipped_fraction,
    })
}

/// Evaluate the total flow cost by summing arc costs row-by-row.
///
/// This is the idiomatic Rust equivalent of C `EvaluateTotalCost()`.
pub fn evaluate_total_cost<F>(
    flows: &[Vec<i16>],
    nrow: usize,
    ncol: Option<usize>,
    narcs_per_row: Option<&[usize]>,
    mut eval_cost: F,
) -> Result<f64, NetworkCostError>
where
    F: FnMut(&[Vec<i16>], usize, usize, usize) -> f64,
{
    let (maxrow, row_width) = if let Some(ncol) = ncol {
        (
            2 * nrow - 1,
            Box::new(move |row: usize| if row < nrow - 1 { ncol } else { ncol - 1 })
                as Box<dyn Fn(usize) -> usize>,
        )
    } else {
        let narcs_per_row = narcs_per_row.ok_or(NetworkCostError::MissingNarcsPerRow)?;
        if narcs_per_row.len() != nrow {
            return Err(NetworkCostError::InvalidRowCount {
                expected: nrow,
                got: narcs_per_row.len(),
            });
        }
        (
            nrow,
            Box::new(move |row: usize| narcs_per_row[row]) as Box<dyn Fn(usize) -> usize>,
        )
    };

    if flows.len() != maxrow {
        return Err(NetworkCostError::InvalidRowCount {
            expected: maxrow,
            got: flows.len(),
        });
    }

    let mut total = 0.0f64;
    for row in 0..maxrow {
        let maxcol = row_width(row);
        if flows[row].len() != maxcol {
            return Err(NetworkCostError::InvalidRowLen {
                row,
                expected: maxcol,
                got: flows[row].len(),
            });
        }
        let mut rowcost = 0.0;
        for col in 0..maxcol {
            rowcost += eval_cost(flows, row, col, nrow);
        }
        total += rowcost;
    }
    Ok(total)
}

/// Compute initialization max-flow bounds from statistical cost metadata.
///
/// This is the idiomatic Rust equivalent of C `CalcInitMaxFlow()`.
pub fn calc_init_max_flow(
    params: &mut InitMaxFlowParams,
    costs: Option<&[Vec<Cost>]>,
    nrow: usize,
    ncol: usize,
) -> Result<(), NetworkCostError> {
    if params.initmaxflow > 0 {
        return Ok(());
    }

    match params.costmode {
        CostMode::NoStatCosts => {
            params.initmaxflow = NOSTAT_INIT_MAX_FLOW;
            Ok(())
        }
        CostMode::Topo | CostMode::Defo => {
            if params.nshortcycle <= 0 {
                return Err(NetworkCostError::InvalidNShortCycle {
                    nshortcycle: params.nshortcycle,
                });
            }
            let costs = costs.ok_or(NetworkCostError::MissingCostArray)?;
            let expected_rows = 2 * nrow - 1;
            if costs.len() != expected_rows {
                return Err(NetworkCostError::InvalidRowCount {
                    expected: expected_rows,
                    got: costs.len(),
                });
            }
            let mut initmaxflow = 0i64;
            for (row, row_costs) in costs.iter().enumerate() {
                let maxcol = if row < nrow - 1 { ncol } else { ncol - 1 };
                if row_costs.len() != maxcol {
                    return Err(NetworkCostError::InvalidRowLen {
                        row,
                        expected: maxcol,
                        got: row_costs.len(),
                    });
                }
                for cost in row_costs {
                    if cost.dz_max != LARGE_SHORT {
                        let arcmaxflow = ((i64::from(cost.dz_max).abs() as f64
                            / params.nshortcycle as f64)
                            + params.arcmaxflowconst as f64)
                            .ceil() as i64;
                        if arcmaxflow > initmaxflow {
                            initmaxflow = arcmaxflow;
                        }
                    }
                }
            }
            params.initmaxflow = initmaxflow;
            Ok(())
        }
        CostMode::Smooth => {
            params.initmaxflow = DEF_INIT_MAX_FLOW;
            Ok(())
        }
    }
}

/// Evaluate one arc for reduced-cost violations and optionally add candidate.
///
/// This is the idiomatic Rust equivalent of C `CheckArcReducedCost()`.
#[allow(clippy::too_many_arguments)]
pub fn check_arc_reduced_cost(
    from_idx: usize,
    to_idx: usize,
    apex_idx: usize,
    arcrow: usize,
    arccol: usize,
    arcdir: i64,
    nodes: &[TreeNode],
    incrcosts: &[Vec<IncrCost>],
    is_candidate: &mut [Vec<bool>],
    candidate_bag: &mut Vec<CandidateArc>,
) -> Result<bool, NetworkCostError> {
    let len = nodes.len();
    if from_idx >= len {
        return Err(NetworkCostError::InvalidNodeIndex {
            index: from_idx,
            len,
        });
    }
    if to_idx >= len {
        return Err(NetworkCostError::InvalidNodeIndex { index: to_idx, len });
    }
    if apex_idx >= len {
        return Err(NetworkCostError::InvalidNodeIndex {
            index: apex_idx,
            len,
        });
    }

    let arc_cost = incrcosts
        .get(arcrow)
        .and_then(|row| row.get(arccol))
        .ok_or(NetworkCostError::MissingArc { arcrow, arccol })?;
    let marked = is_candidate
        .get_mut(arcrow)
        .and_then(|row| row.get_mut(arccol))
        .ok_or(NetworkCostError::MissingArc { arcrow, arccol })?;
    if *marked {
        return Ok(false);
    }

    let mut from = from_idx;
    let mut to = to_idx;
    let mut out_dir = arcdir;

    let apexcost = nodes[apex_idx].outcost + nodes[apex_idx].incost;
    let fwdarcdist = i64::from(arc_cost.get_cost(out_dir));
    let revarcdist = i64::from(arc_cost.get_cost(-out_dir));
    let mut violation = fwdarcdist + nodes[from].outcost + nodes[to].incost - apexcost;

    if violation < 0 {
        out_dir *= 2;
    } else {
        violation = revarcdist + nodes[to].outcost + nodes[from].incost - apexcost;
        if violation < 0 {
            out_dir *= -2;
            std::mem::swap(&mut from, &mut to);
        } else {
            violation = fwdarcdist + nodes[from].outcost - nodes[to].outcost;
            if violation >= 0 {
                violation = revarcdist + nodes[to].outcost - nodes[from].outcost;
                if violation < 0 {
                    out_dir = -out_dir;
                    std::mem::swap(&mut from, &mut to);
                }
            }
        }
    }

    if violation < 0 {
        candidate_bag.push(CandidateArc {
            violation,
            from,
            to,
            arcrow,
            arccol,
            arcdir: out_dir,
        });
        *marked = true;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Relabel all connected nodes in one region to `newnum`.
///
/// This is the idiomatic Rust equivalent of C `RenumberRegion()`.
pub fn renumber_region(
    region_ids: &mut [Vec<i64>],
    source: GridNodeCoord,
    newnum: i64,
) -> Result<usize, NetworkCostError> {
    if region_ids.is_empty() || region_ids[0].is_empty() {
        return Err(NetworkCostError::InvalidRegionGrid);
    }
    let nrow = region_ids.len();
    let ncol = region_ids[0].len();
    if region_ids.iter().any(|row| row.len() != ncol) {
        return Err(NetworkCostError::InvalidRegionGrid);
    }
    if source.row >= nrow || source.col >= ncol {
        return Err(NetworkCostError::InvalidRegionSource {
            row: source.row,
            col: source.col,
        });
    }

    let regionnum = region_ids[source.row][source.col];
    let mut stack = vec![(source.row, source.col)];
    let mut changed = 0usize;
    while let Some((row, col)) = stack.pop() {
        if region_ids[row][col] != regionnum {
            continue;
        }
        region_ids[row][col] = newnum;
        changed += 1;
        if row > 0 && region_ids[row - 1][col] == regionnum {
            stack.push((row - 1, col));
        }
        if row + 1 < nrow && region_ids[row + 1][col] == regionnum {
            stack.push((row + 1, col));
        }
        if col > 0 && region_ids[row][col - 1] == regionnum {
            stack.push((row, col - 1));
        }
        if col + 1 < ncol && region_ids[row][col + 1] == regionnum {
            stack.push((row, col + 1));
        }
    }
    Ok(changed)
}

/// Merge the connected source-region into `closest_region`.
///
/// This is the idiomatic Rust equivalent of C `MergeRegions()`.
pub fn merge_regions(
    region_ids: &mut [Vec<i64>],
    source: GridNodeCoord,
    region_sizes: &mut [i64],
    closest_region: usize,
) -> Result<usize, NetworkCostError> {
    if closest_region >= region_sizes.len() {
        return Err(NetworkCostError::InvalidRegionSizeIndex {
            index: closest_region,
            len: region_sizes.len(),
        });
    }
    if region_ids.is_empty() || region_ids[0].is_empty() {
        return Err(NetworkCostError::InvalidRegionGrid);
    }
    if source.row >= region_ids.len() || source.col >= region_ids[0].len() {
        return Err(NetworkCostError::InvalidRegionSource {
            row: source.row,
            col: source.col,
        });
    }

    let source_region = region_ids[source.row][source.col];
    if source_region < 0 || source_region as usize >= region_sizes.len() {
        return Err(NetworkCostError::InvalidRegionSizeIndex {
            index: source_region.max(0) as usize,
            len: region_sizes.len(),
        });
    }

    let changed = renumber_region(region_ids, source, closest_region as i64)?;
    region_sizes[closest_region] += region_sizes[source_region as usize];
    Ok(changed)
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
    let has_zero = pixels.contains(&0.0);
    let has_nonzero = pixels.iter().any(|&v| v != 0.0);
    has_zero && has_nonzero
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_grid_network_function_pointers_selects_grid_mode() {
        assert_eq!(
            set_grid_network_function_pointers(),
            NetworkFunctionPointers::Grid
        );
    }

    #[test]
    fn set_non_grid_network_function_pointers_selects_non_grid_mode() {
        assert_eq!(
            set_non_grid_network_function_pointers(),
            NetworkFunctionPointers::NonGrid
        );
    }

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
        mag[4] = 1.0;
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
        mag[2] = 5.0;
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
        mag[4] = 1.0;
        assert_eq!(ground_mask_status(&mag, 3, 4), 0);
    }

    #[test]
    fn ground_interior_nonzero_still_masked() {
        let mut mag = vec![0.0f32; 12]; // 3×4
        // Set an interior pixel — edges are still all zero.
        mag[5] = 99.0;
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
        mag[1] = 1.0;
        assert!(is_region_arc(Some(&mag), 0, 1, 3, 4));
    }

    #[test]
    fn region_arc_column_arc() {
        // Column arc: arcrow >= nrow-1. For nrow=3, arcrow=2 → row = 2-2 = 0
        // pixels (0, arccol) and (0, arccol+1).
        let mut mag = vec![0.0f32; 12]; // 3×4
        mag[2] = 5.0; // pixel (0,2)
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
        mag[4] = 0.0;
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

    fn node_grid(rows: usize, cols: usize, outcost: i64) -> Vec<Vec<TreeNode>> {
        (0..rows)
            .map(|_| (0..cols).map(|_| TreeNode::new(outcost)).collect())
            .collect()
    }

    fn row_col_i16(nrow: usize, ncol: usize, value: i16) -> Vec<Vec<i16>> {
        (0..(2 * nrow - 1))
            .map(|row| {
                if row < nrow - 1 {
                    vec![value; ncol]
                } else {
                    vec![value; ncol - 1]
                }
            })
            .collect()
    }

    fn residue_i8(nrow: usize, ncol: usize, value: i8) -> Vec<Vec<i8>> {
        (0..(nrow - 1)).map(|_| vec![value; ncol - 1]).collect()
    }

    // --- New tree/masking helpers ---

    #[test]
    fn check_mag_masking_matches_c_semantics() {
        let all_zero = vec![vec![0.0f32; 3]; 2];
        assert!(check_mag_masking(&all_zero, 2, 3).unwrap());

        let mut with_signal = all_zero.clone();
        with_signal[1][2] = 0.1;
        assert!(!check_mag_masking(&with_signal, 2, 3).unwrap());
    }

    #[test]
    fn mask_nodes_sets_grid_and_ground_groups() {
        let nrow = 3;
        let ncol = 3;
        let mut nodes = node_grid(nrow - 1, ncol - 1, 0);
        let mut ground = TreeNode::new(0);
        let mag = vec![
            vec![0.0f32, 0.0, 0.0],
            vec![0.0f32, 1.0, 0.0],
            vec![0.0f32, 0.0, 0.0],
        ];

        mask_nodes(nrow, ncol, &mut nodes, &mut ground, &mag).unwrap();
        for row in &nodes {
            for node in row {
                assert_eq!(node.group, TreeNodeGroup::Normal);
            }
        }
        // Ground only checks image boundaries; interior nonzero does not unmask it.
        assert_eq!(ground.group, TreeNodeGroup::Masked);
    }

    #[test]
    fn max_non_mask_flow_ignores_arcs_touching_masked_pixels() {
        let nrow = 3;
        let ncol = 3;
        let mut flows = row_col_i16(nrow, ncol, 0);
        let mag = vec![
            vec![1.0f32, 1.0, 1.0],
            vec![1.0f32, 0.0, 1.0],
            vec![1.0f32, 1.0, 1.0],
        ];

        flows[0][1] = 50; // touches masked pixel at (1,1) -> ignored
        flows[2][0] = -30; // valid column arc between two unmasked pixels
        flows[1][0] = 20; // valid row arc

        assert_eq!(max_non_mask_flow(&flows, &mag, nrow, ncol).unwrap(), 30);
    }

    #[test]
    fn init_node_nums_sets_grid_and_ground_coordinates() {
        let mut nodes = node_grid(2, 3, 5);
        let mut ground = TreeNode::new(0);
        init_node_nums(2, 3, &mut nodes, Some(&mut ground)).unwrap();

        assert_eq!(nodes[0][0].row, 0);
        assert_eq!(nodes[0][0].col, 0);
        assert_eq!(nodes[1][2].row, 1);
        assert_eq!(nodes[1][2].col, 2);
        assert_eq!(ground.row, GROUNDROW);
        assert_eq!(ground.col, GROUND_COL);
    }

    #[test]
    fn init_nodes_resets_tree_state() {
        let mut nodes = node_grid(2, 2, 9);
        nodes[1][1].group = TreeNodeGroup::InBucket;
        nodes[1][1].incost = 7;
        nodes[1][1].pred = Some(0);
        nodes[1][1].bucket_index = Some(1);
        let mut ground = TreeNode::new(123);
        ground.group = TreeNodeGroup::OnTree;
        ground.incost = 88;
        ground.pred = Some(0);
        ground.bucket_index = Some(4);

        init_nodes(2, 2, &mut nodes, Some(&mut ground)).unwrap();

        assert_eq!(nodes[1][1].group, TreeNodeGroup::NotInBucket);
        assert_eq!(nodes[1][1].incost, VERY_FAR);
        assert_eq!(nodes[1][1].outcost, VERY_FAR);
        assert_eq!(nodes[1][1].pred, None);
        assert_eq!(nodes[1][1].bucket_index, None);

        assert_eq!(ground.group, TreeNodeGroup::NotInBucket);
        assert_eq!(ground.incost, VERY_FAR);
        assert_eq!(ground.outcost, VERY_FAR);
        assert_eq!(ground.pred, None);
        assert_eq!(ground.bucket_index, None);
    }

    #[test]
    fn init_buckets_places_source_in_first_bucket() {
        let mut buckets = FrontierBuckets::new(0, 3, 2).unwrap();
        let mut nodes = vec![TreeNode::new(9), TreeNode::new(10)];

        init_buckets(&mut buckets, &mut nodes, 1).unwrap();

        assert_eq!(buckets.curr, 0);
        assert_eq!(buckets.slots[0], vec![1]);
        assert_eq!(nodes[1].group, TreeNodeGroup::InBucket);
        assert_eq!(nodes[1].outcost, 0);
        assert_eq!(nodes[1].bucket_index, Some(0));
    }

    #[test]
    fn min_out_cost_node_scans_extreme_bucket_for_true_minimum() {
        let mut buckets = FrontierBuckets::new(0, 4, 0).unwrap();
        let mut nodes = vec![TreeNode::new(8), TreeNode::new(3), TreeNode::new(5)];
        buckets.insert(0, 0).unwrap();
        buckets.insert(0, 1).unwrap();
        buckets.insert(0, 2).unwrap();
        nodes[0].bucket_index = Some(0);
        nodes[1].bucket_index = Some(0);
        nodes[2].bucket_index = Some(0);

        let chosen = min_out_cost_node(&mut buckets, &mut nodes)
            .unwrap()
            .unwrap();
        assert_eq!(chosen, 1);
        assert_eq!(nodes[chosen].bucket_index, None);
    }

    #[test]
    fn find_apex_returns_deepest_common_ancestor() {
        let mut nodes = vec![TreeNode::new(0); 5];
        nodes[0].level = 0;
        nodes[0].pred = None;
        nodes[1].level = 1;
        nodes[1].pred = Some(0);
        nodes[2].level = 2;
        nodes[2].pred = Some(1);
        nodes[3].level = 1;
        nodes[3].pred = Some(0);
        nodes[4].level = 2;
        nodes[4].pred = Some(3);

        assert_eq!(find_apex(&nodes, 2, 4), Some(0));
        assert_eq!(find_apex(&nodes, 2, 1), Some(1));
    }

    #[test]
    fn clip_flow_returns_true_when_no_clipping_needed() {
        let nrow = 3;
        let ncol = 3;
        let mut residue = residue_i8(nrow, ncol, 0);
        let mut flows = row_col_i16(nrow, ncol, 0);
        let mut mstcosts = row_col_i16(nrow, ncol, 10);
        flows[0][0] = 3;

        let stable = clip_flow(&mut residue, &mut flows, &mut mstcosts, nrow, ncol, 3).unwrap();
        assert!(stable);
        assert_eq!(flows[0][0], 3);
        assert_eq!(residue[0][0], 0);
    }

    #[test]
    fn clip_flow_clips_updates_residue_and_costs() {
        let nrow = 3;
        let ncol = 3;
        let mut residue = residue_i8(nrow, ncol, 0);
        let mut flows = row_col_i16(nrow, ncol, 0);
        let mut mstcosts = row_col_i16(nrow, ncol, 20);
        flows[1][1] = 10;

        let stable = clip_flow(&mut residue, &mut flows, &mut mstcosts, nrow, ncol, 3).unwrap();
        assert!(!stable);
        // mostflow=10 -> cliplimit=8
        assert_eq!(flows[1][1], 8);
        assert_eq!(residue[1][0], 2);
        assert_eq!(residue[1][1], -2);
        assert_eq!(mstcosts[1][1], 220);
    }

    #[test]
    fn clear_buckets_resets_node_state_and_bucket_window() {
        let mut buckets = FrontierBuckets::new(2, 5, 4).unwrap();
        let mut nodes = vec![TreeNode::new(1), TreeNode::new(2)];
        buckets.insert(2, 0).unwrap();
        buckets.insert(5, 1).unwrap();
        nodes[0].group = TreeNodeGroup::InBucket;
        nodes[1].group = TreeNodeGroup::InBucket;
        nodes[0].pred = Some(1);
        nodes[1].pred = Some(0);
        nodes[0].bucket_index = Some(2);
        nodes[1].bucket_index = Some(5);

        clear_buckets(&mut buckets, &mut nodes).unwrap();

        assert_eq!(buckets.minind, 0);
        assert_eq!(buckets.maxind, 3);
        assert_eq!(buckets.curr, 0);
        assert!(buckets.slots.iter().all(Vec::is_empty));
        assert_eq!(nodes[0].group, TreeNodeGroup::NotInBucket);
        assert_eq!(nodes[0].outcost, VERY_FAR);
        assert_eq!(nodes[0].pred, None);
        assert_eq!(nodes[0].bucket_index, None);
        assert_eq!(nodes[1].group, TreeNodeGroup::NotInBucket);
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

    #[test]
    fn recalc_cost_clips_large_values() {
        let mut incr = vec![vec![IncrCost::default()]];
        let clipped = recalc_cost(&mut incr, 0, 0, 0, 1, 2, |_flow, _r, _c, _nflow, _nrow| {
            (50_000, -50_000)
        })
        .unwrap();
        assert_eq!(clipped, 2);
        assert_eq!(incr[0][0].poscost, LARGE_SHORT);
        assert_eq!(incr[0][0].negcost, -LARGE_SHORT);
    }

    #[test]
    fn setup_incr_flow_costs_reports_clipped_fraction() {
        let mut incr = vec![
            vec![
                IncrCost::default(),
                IncrCost::default(),
                IncrCost::default(),
            ],
            vec![IncrCost::default(), IncrCost::default()],
            vec![IncrCost::default(), IncrCost::default()],
        ];
        let flows = vec![vec![0i16, 1, 2], vec![0i16, 0], vec![0i16, 0]];
        let stats = setup_incr_flow_costs(
            &mut incr,
            &flows,
            1,
            2,
            &[3, 2, 2],
            |_flow, _r, _c, _, _| (40_000, -40_000),
        )
        .unwrap();
        assert_eq!(stats.narcs, 7);
        assert_eq!(stats.clipped_cost_count, 14);
        assert!((stats.clipped_fraction - 1.0).abs() < 1e-12);
    }

    #[test]
    fn evaluate_total_cost_grid_mode_sums_eval_callback() {
        let flows = vec![vec![1i16, -2, 3], vec![4, 5], vec![-6, 7]];
        let total = evaluate_total_cost(&flows, 2, Some(3), None, |arr, row, col, _| {
            f64::from(arr[row][col].abs())
        })
        .unwrap();
        assert_eq!(total, 28.0);
    }

    #[test]
    fn calc_init_max_flow_uses_dzmax_for_topo_mode() {
        let mut params = InitMaxFlowParams {
            initmaxflow: 0,
            costmode: CostMode::Topo,
            nshortcycle: 2,
            arcmaxflowconst: 3,
        };
        let costs = vec![
            vec![
                Cost::new(0, 0, 6, 0),
                Cost::new(0, 0, LARGE_SHORT, 0),
                Cost::new(0, 0, -9, 0),
            ],
            vec![Cost::new(0, 0, 1, 0), Cost::new(0, 0, 2, 0)],
            vec![Cost::new(0, 0, 5, 0), Cost::new(0, 0, 8, 0)],
        ];
        calc_init_max_flow(&mut params, Some(&costs), 2, 3).unwrap();
        assert_eq!(params.initmaxflow, 8);
    }

    #[test]
    fn check_arc_reduced_cost_adds_candidate_when_violated() {
        let nodes = vec![
            TreeNode {
                row: 0,
                col: 0,
                level: 0,
                incost: 0,
                outcost: 0,
                pred: None,
                group: TreeNodeGroup::Normal,
                bucket_index: None,
            },
            TreeNode {
                row: 0,
                col: 1,
                level: 0,
                incost: -10,
                outcost: 0,
                pred: None,
                group: TreeNodeGroup::Normal,
                bucket_index: None,
            },
            TreeNode {
                row: 0,
                col: 2,
                level: 0,
                incost: 0,
                outcost: 0,
                pred: None,
                group: TreeNodeGroup::Normal,
                bucket_index: None,
            },
        ];
        let incr = vec![vec![IncrCost::new(2, 5)]];
        let mut is_candidate = vec![vec![false]];
        let mut bag = Vec::new();

        let added =
            check_arc_reduced_cost(0, 1, 2, 0, 0, 1, &nodes, &incr, &mut is_candidate, &mut bag)
                .unwrap();
        assert!(added);
        assert_eq!(bag.len(), 1);
        assert!(bag[0].violation < 0);
        assert!(is_candidate[0][0]);
    }

    #[test]
    fn renumber_region_relabels_connected_component_only() {
        let mut regions = vec![vec![1i64, 1, 2], vec![1, 3, 2], vec![4, 3, 2]];
        let changed = renumber_region(&mut regions, GridNodeCoord { row: 0, col: 0 }, 9).unwrap();
        assert_eq!(changed, 3);
        assert_eq!(regions, vec![vec![9, 9, 2], vec![9, 3, 2], vec![4, 3, 2]]);
    }

    #[test]
    fn merge_regions_updates_sizes_and_labels() {
        let mut regions = vec![vec![0i64, 0, 1], vec![0, 2, 1], vec![3, 2, 1]];
        let mut sizes = vec![3i64, 3, 2, 1];
        let changed = merge_regions(
            &mut regions,
            GridNodeCoord { row: 0, col: 2 },
            &mut sizes,
            0,
        )
        .unwrap();
        assert_eq!(changed, 3);
        assert_eq!(regions, vec![vec![0, 0, 0], vec![0, 2, 0], vec![3, 2, 0]]);
        assert_eq!(sizes[0], 6);
    }

    #[test]
    fn solve_cs2_generates_row_col_layout_flows() {
        let residue = vec![vec![1i8, -1], vec![0, 0]];
        let mst = row_col_i16(3, 3, 5);
        let flows = solve_cs2(SolveCs2Params {
            residue: &residue,
            mst_costs: &mst,
            nrow: 3,
            ncol: 3,
            cs2_scale_factor: 1,
        })
        .unwrap();
        assert_eq!(flows.len(), 5);
        assert_eq!(flows[0].len(), 3);
        assert!(flows.iter().flatten().any(|&flow| flow != 0));
    }

    #[test]
    fn solve_mst_marks_tree_arcs() {
        let residue = vec![vec![1i8, 0], vec![0, -1]];
        let mst = row_col_i16(3, 3, 1);
        let out = solve_mst(SolveMstParams {
            residue: &residue,
            mst_costs: &mst,
            nrow: 3,
            ncol: 3,
        })
        .unwrap();
        assert_eq!(out.flows.len(), 5);
        assert_eq!(out.arc_status.len(), 5);
        assert!(out.arc_status.iter().flatten().any(|&s| s == -1));
    }

    #[test]
    fn discharge_tree_applies_residue_and_marks_followed_arcs() {
        let mut flows = row_col_i16(3, 3, 0);
        let mut residue = vec![vec![1i8, 0], vec![0, -1]];
        let mut arc_status = vec![
            vec![-1i8, 0, 0],
            vec![0i8, 0, 0],
            vec![0i8, 0],
            vec![0i8, 0],
            vec![0i8, 0],
        ];
        let source_charge = discharge_tree(DischargeTreeParams {
            flows: &mut flows,
            residue: &mut residue,
            arc_status: &mut arc_status,
            nrow: 3,
            ncol: 3,
        })
        .unwrap();
        assert_eq!(source_charge, 0);
        assert_eq!(arc_status[0][0], -3);
        assert!(residue.iter().flatten().all(|&v| v == 0));
    }

    #[test]
    fn init_boundary_marks_boundary_pointer_nodes() {
        let mut nodes = vec![
            RegionTraversalNode {
                row: 0,
                col: 0,
                group: 0,
                level: 0,
            },
            RegionTraversalNode {
                row: 0,
                col: 1,
                group: 0,
                level: 0,
            },
            RegionTraversalNode {
                row: 1,
                col: 0,
                group: 0,
                level: 0,
            },
            RegionTraversalNode {
                row: 1,
                col: 1,
                group: 0,
                level: 0,
            },
        ];
        let adjacency = vec![
            vec![
                RegionTraversalArc {
                    to: 1,
                    arcrow: 0,
                    arccol: 0,
                },
                RegionTraversalArc {
                    to: 2,
                    arcrow: 2,
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
                    arcrow: 3,
                    arccol: 1,
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
                    to: 1,
                    arcrow: 3,
                    arccol: 1,
                },
                RegionTraversalArc {
                    to: 2,
                    arcrow: 1,
                    arccol: 0,
                },
            ],
        ];
        let mag = vec![0.0f32, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0];

        let out = init_boundary(InitBoundaryParams {
            source_idx: 0,
            nodes: &mut nodes,
            adjacency: &adjacency,
            mag: Some(&mag),
            nrow: 3,
            ncol: 3,
        })
        .unwrap();

        assert_eq!(out.connected_count, 4);
        assert!(out.boundary_nodes.len() >= 3);
        assert!(
            nodes
                .iter()
                .filter(|n| n.group == BOUNDARY_PTR_GROUP)
                .count()
                >= 3
        );
    }

    #[test]
    fn non_degen_update_children_updates_path_nodes() {
        let mut nodes = vec![TreeNode::new(0), TreeNode::new(1), TreeNode::new(2)];
        let updated = non_degen_update_children(NonDegenUpdateParams {
            path: &[0, 1, 2],
            nodes: &mut nodes,
            updated_group: TreeNodeGroup::OnTree,
            dincost: 5,
            doutcost: -3,
        })
        .unwrap();
        assert_eq!(updated, 2);
        assert_eq!(nodes[1].group, TreeNodeGroup::OnTree);
        assert_eq!(nodes[2].incost, VERY_FAR + 5);
    }

    #[test]
    fn tree_solve_runs_end_to_end_pipeline() {
        let mut nodes = vec![RegionTraversalNode {
            row: 0,
            col: 0,
            group: 0,
            level: 0,
        }];
        let adjacency = vec![Vec::<RegionTraversalArc>::new()];
        let residue = vec![vec![1i8]];
        let mst = row_col_i16(2, 2, 1);
        let mag = vec![1.0f32, 1.0, 1.0, 1.0];

        let out = tree_solve(TreeSolveParams {
            source_idx: 0,
            nodes: &mut nodes,
            adjacency: &adjacency,
            mag: Some(&mag),
            residue: &residue,
            mst_costs: &mst,
            nrow: 2,
            ncol: 2,
        })
        .unwrap();

        assert_eq!(out.connected_count, 1);
        assert_eq!(out.flows.len(), 3);
        assert_eq!(out.arc_status.len(), 3);
    }
}
