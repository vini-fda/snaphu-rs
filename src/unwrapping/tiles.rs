#![allow(dead_code)]

//! Tile assembly workflows.
//!
//! This module contains the typed Rust translation of secondary-arc tracing
//! primitives used during tiled unwrapping (`TraceSecondaryArc` in C).

use crate::data::ops::l_round;
use crate::data::tile::TileRegion;
use crate::io::reader::parse_filename;
use crate::io::writer::OutputFileFormat;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const LARGE_INT: i64 = 2_000_000_000;
const ZERO_COST_ARC: i64 = -LARGE_INT;
const MAX_OFFSET_REFINEMENTS: usize = 64;
const TMP_TILE_DIR_ROOT: &str = "snaphu_tiles_";
const TILE_INIT_FILE_ROOT: &str = "snaphu_tileinit_";
const TMP_TILE_ROOT: &str = "tmptile_";
const TMP_TILE_COST_SUFFIX: &str = "cost_";

/// Tile-grid parameters needed to compute non-overlapping read windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileReadSettings {
    pub row_overlap: usize,
    pub col_overlap: usize,
    pub ntilerow: usize,
    pub ntilecol: usize,
}

/// Parameters required to set up one tile run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileSetupParams {
    pub ntilerow: usize,
    pub ntilecol: usize,
    pub rowovrlp: usize,
    pub colovrlp: usize,
    pub minregionsize: usize,
    pub tiledir: PathBuf,
}

/// Output file configuration (global or per-tile).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileOutputFiles {
    pub outfile: PathBuf,
    pub initfile: Option<PathBuf>,
    pub flowfile: Option<PathBuf>,
    pub eifile: Option<PathBuf>,
    pub rowcostfile: Option<PathBuf>,
    pub colcostfile: Option<PathBuf>,
    pub mstrowcostfile: Option<PathBuf>,
    pub mstcolcostfile: Option<PathBuf>,
    pub mstcostsfile: Option<PathBuf>,
    pub corrdumpfile: Option<PathBuf>,
    pub rawcorrdumpfile: Option<PathBuf>,
    pub conncompfile: Option<PathBuf>,
    pub costoutfile: Option<PathBuf>,
    pub logfile: Option<PathBuf>,
    pub outfile_format: OutputFileFormat,
}

impl Default for TileOutputFiles {
    fn default() -> Self {
        Self {
            outfile: PathBuf::new(),
            initfile: None,
            flowfile: None,
            eifile: None,
            rowcostfile: None,
            colcostfile: None,
            mstrowcostfile: None,
            mstcolcostfile: None,
            mstcostsfile: None,
            corrdumpfile: None,
            rawcorrdumpfile: None,
            conncompfile: None,
            costoutfile: None,
            logfile: None,
            outfile_format: OutputFileFormat::Unknown,
        }
    }
}

/// Result bundle from `setup_tile()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileSetupResult {
    pub tile_region: TileRegion,
    pub tile_outfiles: TileOutputFiles,
}

/// Node coordinate in a tile connectivity grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileNodeCoord {
    pub row: usize,
    pub col: usize,
}

/// Inputs for `find_num_paths_out()`.
#[derive(Debug, Clone)]
pub struct FindNumPathsOutInputs<'a> {
    pub ntilerow: usize,
    pub ntilecol: usize,
    pub tilerow: usize,
    pub tilecol: usize,
    pub nnrow: usize,
    pub nncol: usize,
    pub prevncol: usize,
    pub regions: &'a [Vec<i16>],
    pub nextregions: &'a [Vec<i16>],
    pub lastregions: &'a [Vec<i16>],
    pub regionsabove: &'a [i16],
    pub regionsbelow: &'a [i16],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindNumPathsOutError {
    InvalidNode,
    InvalidShape,
}

/// Set up tile geometry and per-tile output filenames.
///
/// This is the idiomatic Rust equivalent of C `SetupTile()`.
pub fn setup_tile(
    nlines: usize,
    linelen: usize,
    params: &TileSetupParams,
    outfiles: &TileOutputFiles,
    tilerow: usize,
    tilecol: usize,
) -> io::Result<TileSetupResult> {
    if params.ntilerow == 0 || params.ntilecol == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile grid dimensions must be non-zero",
        ));
    }
    let ni = (nlines + (params.ntilerow - 1) * params.rowovrlp).div_ceil(params.ntilerow);
    let nj = (linelen + (params.ntilecol - 1) * params.colovrlp).div_ceil(params.ntilecol);

    let first_row = tilerow.saturating_mul(ni.saturating_sub(params.rowovrlp));
    let first_col = tilecol.saturating_mul(nj.saturating_sub(params.colovrlp));
    let tile_nrow = if tilerow == params.ntilerow - 1 {
        nlines.saturating_sub((params.ntilerow - 1) * (ni.saturating_sub(params.rowovrlp)))
    } else {
        ni
    };
    let tile_ncol = if tilecol == params.ntilecol - 1 {
        linelen.saturating_sub((params.ntilecol - 1) * (nj.saturating_sub(params.colovrlp)))
    } else {
        nj
    };

    if params.minregionsize > tile_nrow.saturating_mul(tile_ncol) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "minimum region size cannot exceed tile size",
        ));
    }

    let suffix = format!("_{tilerow}_{tilecol}.{tile_ncol}");
    let map_named = |base: &Path| -> io::Result<PathBuf> {
        let (_, basename) = parse_filename(base)?;
        Ok(params.tiledir.join(format!(
            "{TMP_TILE_ROOT}{}{}",
            basename.to_string_lossy(),
            suffix
        )))
    };

    let tile_outfiles = TileOutputFiles {
        outfile: map_named(&outfiles.outfile)?,
        initfile: outfiles.initfile.as_deref().map(map_named).transpose()?,
        flowfile: outfiles.flowfile.as_deref().map(map_named).transpose()?,
        eifile: outfiles.eifile.as_deref().map(map_named).transpose()?,
        rowcostfile: outfiles.rowcostfile.as_deref().map(map_named).transpose()?,
        colcostfile: outfiles.colcostfile.as_deref().map(map_named).transpose()?,
        mstrowcostfile: outfiles
            .mstrowcostfile
            .as_deref()
            .map(map_named)
            .transpose()?,
        mstcolcostfile: outfiles
            .mstcolcostfile
            .as_deref()
            .map(map_named)
            .transpose()?,
        mstcostsfile: outfiles
            .mstcostsfile
            .as_deref()
            .map(map_named)
            .transpose()?,
        corrdumpfile: outfiles
            .corrdumpfile
            .as_deref()
            .map(map_named)
            .transpose()?,
        rawcorrdumpfile: outfiles
            .rawcorrdumpfile
            .as_deref()
            .map(map_named)
            .transpose()?,
        conncompfile: outfiles
            .conncompfile
            .as_deref()
            .map(map_named)
            .transpose()?,
        costoutfile: match outfiles.costoutfile.as_deref() {
            Some(path) => Some(map_named(path)?),
            None => Some(params.tiledir.join(format!(
                "{TMP_TILE_ROOT}{TMP_TILE_COST_SUFFIX}{tilerow}_{tilecol}.{tile_ncol}"
            ))),
        },
        logfile: outfiles.logfile.as_deref().map(map_named).transpose()?,
        outfile_format: OutputFileFormat::AltLineData,
    };

    Ok(TileSetupResult {
        tile_region: TileRegion::new(first_row, first_col, tile_nrow, tile_ncol),
        tile_outfiles,
    })
}

/// Count how many outgoing paths from a node cross region boundaries.
///
/// This is the idiomatic Rust equivalent of C `FindNumPathsOut()`.
pub fn find_num_paths_out(
    from: TileNodeCoord,
    inputs: &FindNumPathsOutInputs<'_>,
) -> Result<usize, FindNumPathsOutError> {
    let fromrow = from.row;
    let fromcol = from.col;
    let nnrow = inputs.nnrow;
    let nncol = inputs.nncol;
    if fromrow >= nnrow || fromcol >= nncol {
        return Err(FindNumPathsOutError::InvalidNode);
    }

    let get_regions = |r: usize, c: usize| -> Result<i16, FindNumPathsOutError> {
        inputs
            .regions
            .get(r)
            .and_then(|row| row.get(c))
            .copied()
            .ok_or(FindNumPathsOutError::InvalidShape)
    };
    let get_next = |r: usize, c: usize| -> Result<i16, FindNumPathsOutError> {
        inputs
            .nextregions
            .get(r)
            .and_then(|row| row.get(c))
            .copied()
            .ok_or(FindNumPathsOutError::InvalidShape)
    };
    let get_last = |r: usize, c: usize| -> Result<i16, FindNumPathsOutError> {
        inputs
            .lastregions
            .get(r)
            .and_then(|row| row.get(c))
            .copied()
            .ok_or(FindNumPathsOutError::InvalidShape)
    };
    let get_above = |c: usize| -> Result<i16, FindNumPathsOutError> {
        inputs
            .regionsabove
            .get(c)
            .copied()
            .ok_or(FindNumPathsOutError::InvalidShape)
    };
    let get_below = |c: usize| -> Result<i16, FindNumPathsOutError> {
        inputs
            .regionsbelow
            .get(c)
            .copied()
            .ok_or(FindNumPathsOutError::InvalidShape)
    };

    let mut npathsout = 0usize;

    if fromcol != nncol - 1 {
        if fromrow == 0
            || fromrow == nnrow - 1
            || get_regions(fromrow - 1, fromcol)? != get_regions(fromrow, fromcol)?
        {
            npathsout += 1;
        }
    } else if fromrow == 0
        || fromrow == nnrow - 1
        || (inputs.tilecol != inputs.ntilecol - 1
            && get_next(fromrow - 1, 0)? != get_next(fromrow, 0)?)
    {
        npathsout += 1;
    }

    if fromrow != nnrow - 1 {
        if fromcol == 0
            || fromcol == nncol - 1
            || get_regions(fromrow, fromcol)? != get_regions(fromrow, fromcol - 1)?
        {
            npathsout += 1;
        }
    } else if fromcol == 0
        || fromcol == nncol - 1
        || (inputs.tilerow != inputs.ntilerow - 1 && get_below(fromcol)? != get_below(fromcol - 1)?)
    {
        npathsout += 1;
    }

    if fromcol != 0 {
        if fromrow == 0
            || fromrow == nnrow - 1
            || get_regions(fromrow, fromcol - 1)? != get_regions(fromrow - 1, fromcol - 1)?
        {
            npathsout += 1;
        }
    } else if fromrow == 0
        || fromrow == nnrow - 1
        || (inputs.tilecol != 0
            && get_last(fromrow, inputs.prevncol - 1)?
                != get_last(fromrow - 1, inputs.prevncol - 1)?)
    {
        npathsout += 1;
    }

    if fromrow != 0 {
        if fromcol == 0
            || fromcol == nncol - 1
            || get_regions(fromrow - 1, fromcol - 1)? != get_regions(fromrow - 1, fromcol)?
        {
            npathsout += 1;
        }
    } else if fromcol == 0
        || fromcol == nncol - 1
        || (inputs.tilerow != 0 && get_above(fromcol - 1)? != get_above(fromcol)?)
    {
        npathsout += 1;
    }

    Ok(npathsout)
}

/// Create a temporary directory used for tile artifacts.
///
/// This is the idiomatic Rust equivalent of C `MakeTileDir()`.
pub fn make_tile_dir(
    tiledir: Option<&Path>,
    outfile: &Path,
    parent_pid: u32,
) -> io::Result<PathBuf> {
    let target = match tiledir {
        Some(path) => path.to_path_buf(),
        None => {
            let (path, _) = parse_filename(outfile)?;
            path.join(format!("{TMP_TILE_DIR_ROOT}{parent_pid}"))
        }
    };

    if fs::metadata(&target).is_ok() {
        return Ok(target);
    }

    fs::create_dir(&target)?;
    Ok(target)
}

/// Build the temporary tile-init output filename and ensure it does not exist.
///
/// This is the idiomatic Rust equivalent of C `SetTileInitOutfile()`.
pub fn set_tile_init_outfile(outfile: &Path, pid: u32) -> io::Result<PathBuf> {
    let (path, basename) = parse_filename(outfile)?;
    let mut tile_base = OsString::from(format!("{TILE_INIT_FILE_ROOT}{pid}_"));
    tile_base.push(basename);
    let tile_init = path.join(tile_base);

    if fs::metadata(&tile_init).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "refusing to write tile init to existing file {}",
                tile_init.display()
            ),
        ));
    }

    Ok(tile_init)
}

/// Set parameters for reading the non-overlapping interior of one tile.
///
/// This is the idiomatic Rust equivalent of C `SetTileReadParams()`.
pub fn set_tile_read_params(
    next_tile_nlines: usize,
    next_tile_linelen: usize,
    tilerow: usize,
    tilecol: usize,
    settings: TileReadSettings,
) -> TileRegion {
    let first_row = if tilerow == 0 {
        0
    } else {
        settings.row_overlap.div_ceil(2)
    };
    let row_trim_tail = if tilerow != settings.ntilerow.saturating_sub(1) {
        settings.row_overlap / 2
    } else {
        0
    };
    let rows = next_tile_nlines
        .saturating_sub(first_row)
        .saturating_sub(row_trim_tail);

    let first_col = if tilecol == 0 {
        0
    } else {
        settings.col_overlap.div_ceil(2)
    };
    let col_trim_tail = if tilecol != settings.ntilecol.saturating_sub(1) {
        settings.col_overlap / 2
    } else {
        0
    };
    let cols = next_tile_linelen
        .saturating_sub(first_col)
        .saturating_sub(col_trim_tail);

    TileRegion::new(first_row, first_col, rows, cols)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcDirection {
    Right = 1,
    Down = 2,
    Left = 3,
    Up = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrimaryCoord {
    pub row: i64,
    pub col: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimaryArcHop {
    /// `None` matches the C `LARGESHORT` sentinel path where the arc is
    /// considered zero-cost for variance accumulation.
    pub sigma_sq: Option<f64>,
    /// Marks arcs that are forced zero-cost because they lie on global edges.
    pub forced_zero_cost: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecondaryCostParams {
    pub flowmax: usize,
    pub nshortcycle: i64,
    pub tileedgeweight: f64,
    pub is_tile_edge_arc: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryArcCostProfile {
    /// Offset used while sampling per-flow costs (`scndrycostarr[0]` in C).
    pub arroffset: i64,
    /// Incremental costs for `+/-nflow` (`scndrycostarr[1..=2*flowmax]`).
    pub incremental_costs: Vec<i64>,
    /// Variance metadata stored in the final slot (`scndrycostarr[2*flowmax+1]`).
    pub variance_sum_tag: i64,
    /// Number of traversed primary arcs while tracing the secondary arc.
    pub arc_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceCostError {
    EmptyPath,
    InvalidFlowMax,
    OffsetRefinementDidNotConverge,
}

/// Compute secondary-arc cost profile from a traced primary-arc path.
///
/// This is the typed Rust equivalent of the cost-building core in C
/// `TraceSecondaryArc()`, including:
/// - centered cost-table search via `arroffset`
/// - zero-cost arc handling
/// - tile-edge weighting
/// - variance-tag storage in the trailing slot
///
/// The `eval_costs` callback must return `(nominal, pos, neg)` costs for a
/// hop at the current `arroffset` and requested `nflow` increment.
pub fn trace_secondary_arc_costs<F>(
    hops: &[PrimaryArcHop],
    params: SecondaryCostParams,
    mut eval_costs: F,
) -> Result<SecondaryArcCostProfile, TraceCostError>
where
    F: FnMut(usize, i64, i64) -> (i64, i64, i64),
{
    if hops.is_empty() {
        return Err(TraceCostError::EmptyPath);
    }
    if params.flowmax == 0 {
        return Err(TraceCostError::InvalidFlowMax);
    }

    let flowmax = params.flowmax as i64;
    let mut arroffset = 0i64;
    let mut refinements = 0usize;
    let mut zerocost: bool;

    let mut incremental = vec![0i64; (2 * params.flowmax) as usize];
    let mut sumsigsqinv: f64;

    loop {
        incremental.fill(0);
        sumsigsqinv = 0.0;
        zerocost = false;

        for (hop_idx, hop) in hops.iter().enumerate() {
            if hop.forced_zero_cost {
                zerocost = true;
                break;
            }

            for nflow in 1..=flowmax {
                let (nom, pos, neg) = eval_costs(hop_idx, arroffset, nflow);
                let pos_idx = (nflow - 1) as usize;
                let neg_idx = (flowmax + nflow - 1) as usize;
                incremental[pos_idx] =
                    clip_large_int(incremental[pos_idx].saturating_add(pos.saturating_sub(nom)));
                incremental[neg_idx] =
                    clip_large_int(incremental[neg_idx].saturating_add(neg.saturating_sub(nom)));
            }

            if let Some(sigsq) = hop.sigma_sq {
                if sigsq > 0.0 && sigsq.is_finite() {
                    sumsigsqinv += 1.0 / sigsq;
                }
            }
        }

        if zerocost {
            break;
        }

        let mut min_cost = 0i64;
        let mut max_cost = 0i64;
        let mut min_cost_flow = 0i64;
        for nflow in 1..=flowmax {
            let pos = incremental[(nflow - 1) as usize];
            if pos < min_cost {
                min_cost = pos;
                min_cost_flow = nflow;
            }
            if pos > max_cost {
                max_cost = pos;
            }

            let neg = incremental[(flowmax + nflow - 1) as usize];
            if neg < min_cost {
                min_cost = neg;
                min_cost_flow = -nflow;
            }
            if neg > max_cost {
                max_cost = neg;
            }
        }

        // All-zero profile behaves as zero-cost arc in the C implementation.
        if max_cost == min_cost {
            zerocost = true;
            sumsigsqinv = 0.0;
            break;
        }

        if min_cost_flow == 0 {
            break;
        }

        if min_cost_flow == flowmax {
            arroffset -= (1.5 * flowmax as f64).floor() as i64;
        } else if min_cost_flow == -flowmax {
            arroffset += (1.5 * flowmax as f64).floor() as i64;
        } else {
            arroffset -= min_cost_flow;
        }

        refinements += 1;
        if refinements > MAX_OFFSET_REFINEMENTS {
            return Err(TraceCostError::OffsetRefinementDidNotConverge);
        }
    }

    let variance_sum_tag = if zerocost {
        incremental.fill(0);
        0
    } else {
        if params.is_tile_edge_arc {
            for v in &mut incremental {
                *v = clip_large_int(l_round(*v as f64 * params.tileedgeweight));
            }
            sumsigsqinv *= params.tileedgeweight;
        }

        let weighted = sumsigsqinv * (params.nshortcycle * params.nshortcycle) as f64;
        if weighted < LARGE_INT as f64 {
            l_round(weighted)
        } else {
            LARGE_INT
        }
    };

    Ok(SecondaryArcCostProfile {
        arroffset,
        incremental_costs: incremental,
        variance_sum_tag: if zerocost {
            ZERO_COST_ARC
        } else {
            variance_sum_tag
        },
        arc_len: hops.len(),
    })
}

#[inline]
fn clip_large_int(value: i64) -> i64 {
    value.clamp(-LARGE_INT, LARGE_INT)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecondaryNodeKey {
    pub tile: usize,
    pub primary_row: i64,
    pub primary_col: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryNode {
    pub key: SecondaryNodeKey,
    pub neighbors: Vec<usize>,
    pub outarcs: Vec<Option<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryArc {
    pub arcrow: usize,
    pub arccol: usize,
    pub from: usize,
    pub to: usize,
    pub fromdir: ArcDirection,
    pub cost_profile: SecondaryArcCostProfile,
}

#[derive(Debug, Default)]
pub struct SecondaryGraph {
    pub nodes: Vec<SecondaryNode>,
    pub arcs: Vec<SecondaryArc>,
    node_map: HashMap<SecondaryNodeKey, usize>,
}

impl SecondaryGraph {
    pub fn get_or_insert_node(&mut self, key: SecondaryNodeKey) -> usize {
        if let Some(&idx) = self.node_map.get(&key) {
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(SecondaryNode {
            key,
            neighbors: Vec::new(),
            outarcs: Vec::new(),
        });
        self.node_map.insert(key, idx);
        idx
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonTileArcUpdate {
    pub node_idx: usize,
    pub outarc_slot: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceSecondaryArcOutcome {
    IgnoredSourceOrPriorTileEdge,
    IgnoredLoop,
    AddedNewArc { arc_idx: usize },
    ReusedExistingArc { arc_idx: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceSecondaryArcResult {
    pub outcome: TraceSecondaryArcOutcome,
    pub updated_non_tile_nodes: Vec<NonTileArcUpdate>,
    pub total_arc_len_delta: usize,
}

/// Register one traced secondary arc in the mutable secondary graph.
///
/// This is the idiomatic Rust equivalent of the graph-update half of C
/// `TraceSecondaryArc()` (node lookup/creation, duplicate-arc handling,
/// neighbor/outarc bookkeeping, and non-tile update tracking).
pub fn trace_secondary_arc(
    graph: &mut SecondaryGraph,
    current_tile: usize,
    head_key: SecondaryNodeKey,
    tail_key: SecondaryNodeKey,
    fromdir: ArcDirection,
    cost_profile: SecondaryArcCostProfile,
    skip_for_source_or_prev_tile: bool,
) -> TraceSecondaryArcResult {
    if skip_for_source_or_prev_tile {
        return TraceSecondaryArcResult {
            outcome: TraceSecondaryArcOutcome::IgnoredSourceOrPriorTileEdge,
            updated_non_tile_nodes: Vec::new(),
            total_arc_len_delta: 0,
        };
    }

    if head_key == tail_key {
        return TraceSecondaryArcResult {
            outcome: TraceSecondaryArcOutcome::IgnoredLoop,
            updated_non_tile_nodes: Vec::new(),
            total_arc_len_delta: 0,
        };
    }

    let tail_idx = graph.get_or_insert_node(tail_key);
    let head_idx = graph.get_or_insert_node(head_key);

    // Find existing adjacency.
    let mut existing_tail_slot = None;
    for (i, &neighbor) in graph.nodes[tail_idx].neighbors.iter().enumerate() {
        if neighbor == head_idx {
            existing_tail_slot = Some(i);
            break;
        }
    }

    // Existing relation: update arc if present, else create and bind one.
    if let Some(tail_slot) = existing_tail_slot {
        let arc_idx = if let Some(arc_idx) = graph.nodes[tail_idx].outarcs[tail_slot] {
            graph.arcs[arc_idx].fromdir = fromdir;
            graph.arcs[arc_idx].cost_profile = cost_profile.clone();
            arc_idx
        } else {
            let arc_idx = graph.arcs.len();
            let arccol = arc_idx;
            graph.arcs.push(SecondaryArc {
                arcrow: current_tile,
                arccol,
                from: tail_idx,
                to: head_idx,
                fromdir,
                cost_profile: cost_profile.clone(),
            });

            graph.nodes[tail_idx].outarcs[tail_slot] = Some(arc_idx);
            if let Some(head_slot) = find_neighbor_slot(&graph.nodes[head_idx], tail_idx) {
                graph.nodes[head_idx].outarcs[head_slot] = Some(arc_idx);
            }
            arc_idx
        };

        return TraceSecondaryArcResult {
            outcome: TraceSecondaryArcOutcome::ReusedExistingArc { arc_idx },
            updated_non_tile_nodes: collect_non_tile_updates(
                graph,
                current_tile,
                tail_idx,
                head_idx,
            ),
            total_arc_len_delta: cost_profile.arc_len,
        };
    }

    // New relation + new arc.
    let arc_idx = graph.arcs.len();
    graph.arcs.push(SecondaryArc {
        arcrow: current_tile,
        arccol: arc_idx,
        from: tail_idx,
        to: head_idx,
        fromdir,
        cost_profile: cost_profile.clone(),
    });

    graph.nodes[tail_idx].neighbors.push(head_idx);
    graph.nodes[tail_idx].outarcs.push(Some(arc_idx));
    graph.nodes[head_idx].neighbors.push(tail_idx);
    graph.nodes[head_idx].outarcs.push(Some(arc_idx));

    TraceSecondaryArcResult {
        outcome: TraceSecondaryArcOutcome::AddedNewArc { arc_idx },
        updated_non_tile_nodes: collect_non_tile_updates(graph, current_tile, tail_idx, head_idx),
        total_arc_len_delta: cost_profile.arc_len,
    }
}

fn find_neighbor_slot(node: &SecondaryNode, neighbor_idx: usize) -> Option<usize> {
    node.neighbors.iter().position(|&n| n == neighbor_idx)
}

fn collect_non_tile_updates(
    graph: &SecondaryGraph,
    current_tile: usize,
    tail_idx: usize,
    head_idx: usize,
) -> Vec<NonTileArcUpdate> {
    let mut updates = Vec::new();
    if graph.nodes[tail_idx].key.tile != current_tile {
        let outarc_slot = find_neighbor_slot(&graph.nodes[tail_idx], head_idx).unwrap_or(0);
        updates.push(NonTileArcUpdate {
            node_idx: tail_idx,
            outarc_slot,
        });
    }
    if graph.nodes[head_idx].key.tile != current_tile {
        let outarc_slot = find_neighbor_slot(&graph.nodes[head_idx], tail_idx).unwrap_or(0);
        updates.push(NonTileArcUpdate {
            node_idx: head_idx,
            outarc_slot,
        });
    }
    updates
}

/// Placeholder while higher-level tile orchestration is still under migration.
pub fn assemble_tiles() {
    // TODO: call the translated secondary-arc helpers from TraceRegions.
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_suffix() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}_{}", std::process::id(), nanos)
    }

    #[test]
    fn make_tile_dir_creates_default_directory_near_output() {
        let root = std::env::temp_dir().join(format!("snaphu_tiles_root_{}", unique_suffix()));
        fs::create_dir_all(&root).unwrap();
        let outfile = root.join("result.out");
        let dir = make_tile_dir(None, &outfile, 12345).unwrap();

        assert_eq!(dir, root.join("snaphu_tiles_12345"));
        assert!(dir.exists());

        fs::remove_dir_all(&dir).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn make_tile_dir_returns_existing_path_without_recreating() {
        let existing = std::env::temp_dir().join(format!("snaphu_tiles_exist_{}", unique_suffix()));
        fs::create_dir_all(&existing).unwrap();
        let out = make_tile_dir(Some(&existing), Path::new("ignored.out"), 7).unwrap();
        assert_eq!(out, existing);
        fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn set_tile_init_outfile_builds_expected_name_and_rejects_existing_files() {
        let root = std::env::temp_dir().join(format!("snaphu_tileinit_root_{}", unique_suffix()));
        fs::create_dir_all(&root).unwrap();
        let nominal = root.join("tile.out");
        let computed = set_tile_init_outfile(&nominal, 4242).unwrap();
        assert_eq!(computed, root.join("snaphu_tileinit_4242_tile.out"));

        fs::write(&computed, b"already here").unwrap();
        let err = set_tile_init_outfile(&nominal, 4242).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);

        fs::remove_file(computed).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn set_tile_read_params_trims_overlap_for_interior_tiles() {
        let region = set_tile_read_params(
            20,
            30,
            1,
            2,
            TileReadSettings {
                row_overlap: 5,
                col_overlap: 6,
                ntilerow: 3,
                ntilecol: 4,
            },
        );
        assert_eq!(region.first_row, 3); // ceil(5/2)
        assert_eq!(region.first_col, 3); // ceil(6/2)
        assert_eq!(region.rows, 15); // 20 - ceil(5/2) - floor(5/2)
        assert_eq!(region.cols, 24); // 30 - ceil(6/2) - floor(6/2)
    }

    #[test]
    fn set_tile_read_params_keeps_edge_tiles_untrimmed_on_outer_side() {
        let region = set_tile_read_params(
            12,
            10,
            0,
            0,
            TileReadSettings {
                row_overlap: 4,
                col_overlap: 4,
                ntilerow: 2,
                ntilecol: 2,
            },
        );
        assert_eq!(region.first_row, 0);
        assert_eq!(region.first_col, 0);
        assert_eq!(region.rows, 10);
        assert_eq!(region.cols, 8);
    }

    #[test]
    fn trace_secondary_arc_costs_handles_zero_cost_arcs() {
        let hops = vec![
            PrimaryArcHop {
                sigma_sq: Some(4.0),
                forced_zero_cost: true,
            },
            PrimaryArcHop {
                sigma_sq: Some(2.0),
                forced_zero_cost: false,
            },
        ];
        let profile = trace_secondary_arc_costs(
            &hops,
            SecondaryCostParams {
                flowmax: 3,
                nshortcycle: 5,
                tileedgeweight: 2.0,
                is_tile_edge_arc: true,
            },
            |_hop, _offset, _n| (0, 10, 10),
        )
        .unwrap();

        assert_eq!(profile.incremental_costs, vec![0; 6]);
        assert_eq!(profile.variance_sum_tag, ZERO_COST_ARC);
    }

    #[test]
    fn trace_secondary_arc_costs_scales_tile_edge_costs() {
        let hops = vec![PrimaryArcHop {
            sigma_sq: Some(2.0),
            forced_zero_cost: false,
        }];
        let profile = trace_secondary_arc_costs(
            &hops,
            SecondaryCostParams {
                flowmax: 2,
                nshortcycle: 3,
                tileedgeweight: 2.0,
                is_tile_edge_arc: true,
            },
            |_hop, _offset, n| {
                // nom=10, positive increment goes +n, negative +2n
                (10, 10 + n, 10 + 2 * n)
            },
        )
        .unwrap();

        // Original [1,2,2,4] then scaled by x2.
        assert_eq!(profile.incremental_costs, vec![2, 4, 4, 8]);
        assert_eq!(profile.variance_sum_tag, 9); // (1/2)*2 * 3^2 = 9
    }

    #[test]
    fn trace_secondary_arc_costs_recenters_using_arroffset() {
        let hops = vec![PrimaryArcHop {
            sigma_sq: Some(1.0),
            forced_zero_cost: false,
        }];
        let profile = trace_secondary_arc_costs(
            &hops,
            SecondaryCostParams {
                flowmax: 3,
                nshortcycle: 1,
                tileedgeweight: 1.0,
                is_tile_edge_arc: false,
            },
            |_hop, offset, n| {
                // Synthetic convergent model:
                // at offset 0, n=1 gives negative forward delta -> arroffset -= 1
                // at offset -1, all deltas are non-negative -> converged.
                let nom = 10;
                if offset == 0 && n == 1 {
                    (nom, 8, 12)
                } else {
                    (nom, 12, 12)
                }
            },
        )
        .unwrap();

        assert_eq!(profile.arroffset, -1);
    }

    #[test]
    fn trace_secondary_arc_adds_new_arc_and_nodes() {
        let mut graph = SecondaryGraph::default();
        let profile = SecondaryArcCostProfile {
            arroffset: 0,
            incremental_costs: vec![1, 2, 3, 4],
            variance_sum_tag: 7,
            arc_len: 2,
        };
        let tail = SecondaryNodeKey {
            tile: 1,
            primary_row: 4,
            primary_col: 5,
        };
        let head = SecondaryNodeKey {
            tile: 1,
            primary_row: 4,
            primary_col: 6,
        };

        let result = trace_secondary_arc(
            &mut graph,
            1,
            head,
            tail,
            ArcDirection::Right,
            profile,
            false,
        );
        assert!(matches!(
            result.outcome,
            TraceSecondaryArcOutcome::AddedNewArc { .. }
        ));
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.arcs.len(), 1);
        assert_eq!(result.total_arc_len_delta, 2);
    }

    #[test]
    fn trace_secondary_arc_reuses_existing_arc() {
        let mut graph = SecondaryGraph::default();
        let tail = SecondaryNodeKey {
            tile: 2,
            primary_row: 1,
            primary_col: 1,
        };
        let head = SecondaryNodeKey {
            tile: 2,
            primary_row: 1,
            primary_col: 2,
        };
        let p1 = SecondaryArcCostProfile {
            arroffset: 0,
            incremental_costs: vec![1, 1],
            variance_sum_tag: 1,
            arc_len: 1,
        };
        let p2 = SecondaryArcCostProfile {
            arroffset: 3,
            incremental_costs: vec![9, 9],
            variance_sum_tag: 5,
            arc_len: 1,
        };

        let _ = trace_secondary_arc(
            &mut graph,
            2,
            head,
            tail,
            ArcDirection::Right,
            p1.clone(),
            false,
        );
        let result = trace_secondary_arc(
            &mut graph,
            2,
            head,
            tail,
            ArcDirection::Left,
            p2.clone(),
            false,
        );

        assert!(matches!(
            result.outcome,
            TraceSecondaryArcOutcome::ReusedExistingArc { .. }
        ));
        assert_eq!(graph.arcs.len(), 1);
        assert_eq!(graph.arcs[0].fromdir, ArcDirection::Left);
        assert_eq!(graph.arcs[0].cost_profile, p2);
    }

    #[test]
    fn trace_secondary_arc_ignores_source_or_loop_cases() {
        let mut graph = SecondaryGraph::default();
        let key = SecondaryNodeKey {
            tile: 0,
            primary_row: 0,
            primary_col: 0,
        };
        let profile = SecondaryArcCostProfile {
            arroffset: 0,
            incremental_costs: vec![1, 2],
            variance_sum_tag: 3,
            arc_len: 1,
        };

        let source_skip = trace_secondary_arc(
            &mut graph,
            0,
            key,
            SecondaryNodeKey {
                tile: 0,
                primary_row: 0,
                primary_col: 1,
            },
            ArcDirection::Right,
            profile.clone(),
            true,
        );
        assert!(matches!(
            source_skip.outcome,
            TraceSecondaryArcOutcome::IgnoredSourceOrPriorTileEdge
        ));

        let loop_skip =
            trace_secondary_arc(&mut graph, 0, key, key, ArcDirection::Right, profile, false);
        assert!(matches!(
            loop_skip.outcome,
            TraceSecondaryArcOutcome::IgnoredLoop
        ));
    }

    #[test]
    fn setup_tile_builds_expected_tile_geometry_and_names() {
        let params = TileSetupParams {
            ntilerow: 2,
            ntilecol: 2,
            rowovrlp: 2,
            colovrlp: 2,
            minregionsize: 1,
            tiledir: PathBuf::from("/tmp/snaphu_tiles"),
        };
        let outfiles = TileOutputFiles {
            outfile: PathBuf::from("/data/out.bin"),
            initfile: Some(PathBuf::from("/data/init.bin")),
            costoutfile: None,
            ..TileOutputFiles::default()
        };

        let setup = setup_tile(10, 8, &params, &outfiles, 0, 1).unwrap();
        assert_eq!(setup.tile_region.first_row, 0);
        assert_eq!(setup.tile_region.first_col, 3);
        assert_eq!(setup.tile_region.rows, 6);
        assert_eq!(setup.tile_region.cols, 5);
        assert_eq!(
            setup.tile_outfiles.outfile,
            PathBuf::from("/tmp/snaphu_tiles/tmptile_out.bin_0_1.5")
        );
        assert_eq!(
            setup.tile_outfiles.initfile,
            Some(PathBuf::from("/tmp/snaphu_tiles/tmptile_init.bin_0_1.5"))
        );
        assert_eq!(
            setup.tile_outfiles.costoutfile,
            Some(PathBuf::from("/tmp/snaphu_tiles/tmptile_cost_0_1.5"))
        );
        assert_eq!(
            setup.tile_outfiles.outfile_format,
            OutputFileFormat::AltLineData
        );
    }

    #[test]
    fn setup_tile_rejects_too_large_min_region_size() {
        let params = TileSetupParams {
            ntilerow: 1,
            ntilecol: 1,
            rowovrlp: 0,
            colovrlp: 0,
            minregionsize: 100,
            tiledir: PathBuf::from("/tmp/snaphu_tiles"),
        };
        let outfiles = TileOutputFiles {
            outfile: PathBuf::from("/data/out.bin"),
            ..TileOutputFiles::default()
        };
        let err = setup_tile(3, 3, &params, &outfiles, 0, 0).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn find_num_paths_out_matches_boundary_and_interior_cases() {
        let regions = vec![vec![1i16; 3]; 3];
        let neighbors = vec![vec![1i16; 3]; 3];
        let edge = vec![1i16; 3];
        let ctx = FindNumPathsOutInputs {
            ntilerow: 2,
            ntilecol: 2,
            tilerow: 0,
            tilecol: 0,
            nnrow: 3,
            nncol: 3,
            prevncol: 3,
            regions: &regions,
            nextregions: &neighbors,
            lastregions: &neighbors,
            regionsabove: &edge,
            regionsbelow: &edge,
        };

        let boundary_paths = find_num_paths_out(TileNodeCoord { row: 0, col: 0 }, &ctx).unwrap();
        assert_eq!(boundary_paths, 4);

        let interior_paths = find_num_paths_out(TileNodeCoord { row: 1, col: 1 }, &ctx).unwrap();
        assert_eq!(interior_paths, 0);
    }
}
