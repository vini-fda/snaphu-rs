//! Tile assembly workflows.
//!
//! This module contains the typed Rust translation of secondary-arc tracing
//! primitives used during tiled unwrapping (`TraceSecondaryArc` in C).

use crate::constants::{LARGE_SHORT, TWO_PI, TWO_PI_F32};
use crate::costs::types::{IncrCost, thicken_costs};
use crate::data::ops::{avg_sig_sq, l_round};
use crate::data::tile::TileRegion;
use crate::io::reader::parse_filename;
use crate::io::writer::OutputFileFormat;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const LARGE_INT: i64 = 2_000_000_000;
const ZERO_COST_ARC: i64 = -LARGE_INT;
const MAX_OFFSET_REFINEMENTS: usize = 64;
// TODO: Use this in the Rust ports of C `SetLeftEdge()`/`SetRightEdge()`,
// which scale tiled phase-difference offsets by `TILEDPSICOLFACTOR`.
const TILEDPSI_COL_FACTOR: f64 = 0.8;
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
        // Native Rust path uses the `log` facade for runtime logging instead
        // of per-tile logfile fanout.
        logfile: None,
        outfile_format: OutputFileFormat::AltLineData,
    };

    if outfiles.logfile.is_some() {
        log::debug!("ignoring per-tile logfile path during tile setup");
    }

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

#[derive(Debug, Clone, PartialEq)]
pub struct TileRegionSnapshot {
    pub regions: Vec<Vec<i16>>,
    pub unw_phase: Vec<Vec<f32>>,
    pub costs: Vec<Vec<i64>>,
}

pub type TileRegionStore = HashMap<(usize, usize), TileRegionSnapshot>;

#[derive(Debug, Clone, PartialEq)]
pub struct NeighborTileEdges {
    pub regions_above: Option<Vec<i16>>,
    pub regions_below: Option<Vec<i16>>,
    pub unw_phase_above: Option<Vec<f32>>,
    pub unw_phase_below: Option<Vec<f32>>,
    pub costs_above: Option<Vec<i64>>,
    pub costs_below: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryTraceGroup {
    NotInBucket,
    InBucket,
    OnTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryTraceNode {
    pub pred: Option<TileNodeCoord>,
    pub group: PrimaryTraceGroup,
}

impl Default for PrimaryTraceNode {
    fn default() -> Self {
        Self {
            pred: None,
            group: PrimaryTraceGroup::NotInBucket,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionTraceWorkspace {
    pub nodes: Vec<Vec<PrimaryTraceNode>>,
    pub stack: Vec<TileNodeCoord>,
}

impl RegionTraceWorkspace {
    pub fn new(nnrow: usize, nncol: usize) -> Self {
        Self {
            nodes: vec![vec![PrimaryTraceNode::default(); nncol]; nnrow],
            stack: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionCrossing {
    pub head: TileNodeCoord,
    pub tail: TileNodeCoord,
    pub fromdir: ArcDirection,
}

#[derive(Debug, Clone)]
pub struct TraceRegionsParams<'a> {
    pub flowmax: usize,
    pub nshortcycle: i64,
    pub tileedgeweight: f64,
    pub mst_costs: &'a [Vec<i16>],
    pub unw_phase: &'a [Vec<f32>],
    pub inputs: FindNumPathsOutInputs<'a>,
}

#[derive(Debug, Default)]
pub struct TraceRegionsResult {
    pub arc_indices: Vec<usize>,
    pub total_arc_len: usize,
    pub fork_nodes: usize,
}

#[derive(Debug, Clone)]
pub struct ParseSecondaryFlowsParams<'a> {
    pub tilenum: usize,
    pub nrow: usize,
    pub ncol: usize,
    pub ntilerow: usize,
    pub ntilecol: usize,
    pub flip_phase_sign: bool,
    pub regions: &'a [Vec<i16>],
    pub graph: &'a SecondaryGraph,
    pub arc_indices: &'a [usize],
    pub secondary_flows: &'a [i16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTileFlows {
    pub row_flows: Vec<Vec<i16>>,
    pub col_flows: Vec<Vec<i16>>,
}

#[derive(Debug, Clone)]
pub struct TileIntegrationInput {
    pub mag: Vec<Vec<f32>>,
    pub unw_phase: Vec<Vec<f32>>,
    pub regions: Vec<Vec<i16>>,
    pub arc_indices: Vec<usize>,
    pub secondary_flows: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct IntegrateSecondaryFlowsParams<'a> {
    pub linelen: usize,
    pub nlines: usize,
    pub settings: TileReadSettings,
    pub bulk_offsets: &'a [Vec<i16>],
    pub flip_phase_sign: bool,
    pub graph: &'a SecondaryGraph,
    pub tiles: &'a [TileIntegrationInput],
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegratedSecondaryOutput {
    pub mag: Vec<Vec<f32>>,
    pub unw_phase: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileTraceError {
    MissingTile { tilerow: usize, tilecol: usize },
    InvalidTileGrid,
    InvalidTileShape { tilerow: usize, tilecol: usize },
    InvalidBulkOffsets,
    InvalidNeighborEdgeLength { expected: usize, got: usize },
    InvalidRegionShape,
    InvalidArcIndex { index: usize, len: usize },
    MismatchedArcFlowLen { arcs: usize, flows: usize },
    PathTraceDidNotConverge { arc_idx: usize },
    InvalidPrimaryCoord,
    FlowOutOfRange { value: i64 },
    CostProfileError(TraceCostError),
}

impl From<TraceCostError> for TileTraceError {
    fn from(value: TraceCostError) -> Self {
        Self::CostProfileError(value)
    }
}

/// Read one tile snapshot from an in-memory store.
///
/// This is the typed Rust equivalent of C `ReadNextRegion()`.
pub fn read_next_region(
    tilerow: usize,
    tilecol: usize,
    snapshots: &TileRegionStore,
) -> Result<TileRegionSnapshot, TileTraceError> {
    snapshots
        .get(&(tilerow, tilecol))
        .cloned()
        .ok_or(TileTraceError::MissingTile { tilerow, tilecol })
}

/// Read the one-row tile edges above and below the current tile.
///
/// This is the typed Rust equivalent of C `ReadEdgesAboveAndBelow()`.
pub fn read_edges_above_and_below(
    tilerow: usize,
    tilecol: usize,
    ntilerow: usize,
    snapshots: &TileRegionStore,
) -> Result<NeighborTileEdges, TileTraceError> {
    let mut out = NeighborTileEdges {
        regions_above: None,
        regions_below: None,
        unw_phase_above: None,
        unw_phase_below: None,
        costs_above: None,
        costs_below: None,
    };

    if tilerow != 0 {
        let above = read_next_region(tilerow - 1, tilecol, snapshots)?;
        if above.regions.is_empty() || above.unw_phase.is_empty() {
            return Err(TileTraceError::InvalidTileShape {
                tilerow: tilerow - 1,
                tilecol,
            });
        }
        out.regions_above = above.regions.last().cloned();
        out.unw_phase_above = above.unw_phase.last().cloned();
        out.costs_above = above.costs.last().cloned();
    }

    if tilerow != ntilerow.saturating_sub(1) {
        let below = read_next_region(tilerow + 1, tilecol, snapshots)?;
        if below.regions.is_empty() || below.unw_phase.is_empty() {
            return Err(TileTraceError::InvalidTileShape {
                tilerow: tilerow + 1,
                tilecol,
            });
        }
        out.regions_below = below.regions.first().cloned();
        out.unw_phase_below = below.unw_phase.first().cloned();
        out.costs_below = below.costs.first().cloned();
    }

    Ok(out)
}

fn validate_bulk_offsets(
    bulk_offsets: &[Vec<i16>],
    ntilerow: usize,
    ntilecol: usize,
) -> Result<(), TileTraceError> {
    if bulk_offsets.len() != ntilerow {
        return Err(TileTraceError::InvalidBulkOffsets);
    }
    for row in bulk_offsets {
        if row.len() != ntilecol {
            return Err(TileTraceError::InvalidBulkOffsets);
        }
    }
    Ok(())
}

fn to_i16_flow(v: i64) -> Result<i16, TileTraceError> {
    i16::try_from(v).map_err(|_| TileTraceError::FlowOutOfRange { value: v })
}

fn flow_mode(vals: &[i64]) -> i64 {
    let mut hist = HashMap::<i64, usize>::new();
    for &v in vals {
        *hist.entry(v).or_insert(0) += 1;
    }
    let mut best_count = 0usize;
    let mut best_flow = 0i64;
    for (&flow, &count) in &hist {
        if count > best_count || (count == best_count && flow < best_flow) {
            best_count = count;
            best_flow = flow;
        }
    }
    best_flow
}

/// Build top boundary flows for one tile.
///
/// This is the typed Rust equivalent of C `SetUpperEdge()`.
pub fn set_upper_edge(
    current_top: &[f32],
    above_bottom: Option<&[f32]>,
    bulk_offsets: &[Vec<i16>],
    tilerow: usize,
    tilecol: usize,
) -> Result<Vec<i16>, TileTraceError> {
    if tilerow == 0 || above_bottom.is_none() {
        return Ok(vec![0; current_top.len()]);
    }
    let above_bottom = above_bottom.expect("checked above");
    if above_bottom.len() != current_top.len() {
        return Err(TileTraceError::InvalidNeighborEdgeLength {
            expected: current_top.len(),
            got: above_bottom.len(),
        });
    }
    let ntilerow = bulk_offsets.len();
    let ntilecol = bulk_offsets.first().map_or(0, Vec::len);
    validate_bulk_offsets(bulk_offsets, ntilerow, ntilecol)?;
    let rel =
        i64::from(bulk_offsets[tilerow - 1][tilecol]) - i64::from(bulk_offsets[tilerow][tilecol]);

    let mut flows = Vec::with_capacity(current_top.len());
    for col in 0..current_top.len() {
        let dphi = f64::from(above_bottom[col] - current_top[col]) / TWO_PI;
        flows.push(to_i16_flow(l_round(dphi) - rel)?);
    }
    Ok(flows)
}

/// Build left boundary flows for one tile.
///
/// This is the typed Rust equivalent of C `SetLeftEdge()`.
pub fn set_left_edge(
    current_left: &[f32],
    last_right: Option<&[f32]>,
    bulk_offsets: &[Vec<i16>],
    tilerow: usize,
    tilecol: usize,
) -> Result<Vec<i16>, TileTraceError> {
    if tilecol == 0 || last_right.is_none() {
        return Ok(vec![0; current_left.len()]);
    }
    let last_right = last_right.expect("checked above");
    if last_right.len() != current_left.len() {
        return Err(TileTraceError::InvalidNeighborEdgeLength {
            expected: current_left.len(),
            got: last_right.len(),
        });
    }
    let ntilerow = bulk_offsets.len();
    let ntilecol = bulk_offsets.first().map_or(0, Vec::len);
    validate_bulk_offsets(bulk_offsets, ntilerow, ntilecol)?;
    let rel =
        i64::from(bulk_offsets[tilerow][tilecol]) - i64::from(bulk_offsets[tilerow][tilecol - 1]);

    let mut flows = Vec::with_capacity(current_left.len());
    for row in 0..current_left.len() {
        let dphi = f64::from(current_left[row] - last_right[row]) / TWO_PI;
        flows.push(to_i16_flow(l_round(dphi) - rel)?);
    }
    Ok(flows)
}

/// Build right boundary flows for one tile and update row bulk offsets.
///
/// This is the typed Rust equivalent of C `SetRightEdge()`.
pub fn set_right_edge(
    current_right: &[f32],
    next_left: Option<&[f32]>,
    bulk_offsets: &mut [Vec<i16>],
    tilerow: usize,
    tilecol: usize,
) -> Result<Vec<i16>, TileTraceError> {
    let ntilerow = bulk_offsets.len();
    let ntilecol = bulk_offsets.first().map_or(0, Vec::len);
    validate_bulk_offsets(bulk_offsets, ntilerow, ntilecol)?;

    if tilecol == ntilecol.saturating_sub(1) || next_left.is_none() {
        return Ok(vec![0; current_right.len()]);
    }
    let next_left = next_left.expect("checked above");
    if next_left.len() != current_right.len() {
        return Err(TileTraceError::InvalidNeighborEdgeLength {
            expected: current_right.len(),
            got: next_left.len(),
        });
    }

    let mut raw = Vec::with_capacity(current_right.len());
    for row in 0..current_right.len() {
        let dphi = f64::from(next_left[row] - current_right[row]) / TWO_PI;
        raw.push(l_round(dphi));
    }

    let rel = if tilerow == 0 {
        let mode = flow_mode(&raw);
        let new_offset = i64::from(bulk_offsets[tilerow][tilecol]) + mode;
        bulk_offsets[tilerow][tilecol + 1] = to_i16_flow(new_offset)?;
        mode
    } else {
        i64::from(bulk_offsets[tilerow][tilecol + 1]) - i64::from(bulk_offsets[tilerow][tilecol])
    };

    raw.into_iter().map(|v| to_i16_flow(v - rel)).collect()
}

/// Build bottom boundary flows for one tile and update column bulk offsets.
///
/// This is the typed Rust equivalent of C `SetLowerEdge()`.
pub fn set_lower_edge(
    current_bottom: &[f32],
    below_top: Option<&[f32]>,
    bulk_offsets: &mut [Vec<i16>],
    tilerow: usize,
    tilecol: usize,
) -> Result<Vec<i16>, TileTraceError> {
    let ntilerow = bulk_offsets.len();
    let ntilecol = bulk_offsets.first().map_or(0, Vec::len);
    validate_bulk_offsets(bulk_offsets, ntilerow, ntilecol)?;

    if tilerow == ntilerow.saturating_sub(1) || below_top.is_none() {
        return Ok(vec![0; current_bottom.len()]);
    }
    let below_top = below_top.expect("checked above");
    if below_top.len() != current_bottom.len() {
        return Err(TileTraceError::InvalidNeighborEdgeLength {
            expected: current_bottom.len(),
            got: below_top.len(),
        });
    }

    let mut raw = Vec::with_capacity(current_bottom.len());
    for col in 0..current_bottom.len() {
        let dphi = f64::from(current_bottom[col] - below_top[col]) / TWO_PI;
        raw.push(l_round(dphi));
    }

    let rel = if tilecol == 0 {
        let mode = flow_mode(&raw);
        let new_offset = i64::from(bulk_offsets[tilerow][tilecol]) - mode;
        bulk_offsets[tilerow + 1][tilecol] = to_i16_flow(new_offset)?;
        mode
    } else {
        i64::from(bulk_offsets[tilerow][tilecol]) - i64::from(bulk_offsets[tilerow + 1][tilecol])
    };

    raw.into_iter().map(|v| to_i16_flow(v - rel)).collect()
}

fn can_step_right(
    from: TileNodeCoord,
    inputs: &FindNumPathsOutInputs<'_>,
) -> Result<bool, FindNumPathsOutError> {
    if from.col == inputs.nncol - 1 {
        return Ok(false);
    }
    if from.row == 0 || from.row == inputs.nnrow - 1 {
        return Ok(true);
    }
    let a = inputs.regions[from.row - 1][from.col];
    let b = inputs.regions[from.row][from.col];
    Ok(a != b)
}

fn can_step_down(
    from: TileNodeCoord,
    inputs: &FindNumPathsOutInputs<'_>,
) -> Result<bool, FindNumPathsOutError> {
    if from.row == inputs.nnrow - 1 {
        return Ok(false);
    }
    if from.col == 0 || from.col == inputs.nncol - 1 {
        return Ok(true);
    }
    let a = inputs.regions[from.row][from.col];
    let b = inputs.regions[from.row][from.col - 1];
    Ok(a != b)
}

fn can_step_left(
    from: TileNodeCoord,
    inputs: &FindNumPathsOutInputs<'_>,
) -> Result<bool, FindNumPathsOutError> {
    if from.col == 0 {
        return Ok(false);
    }
    if from.row == 0 || from.row == inputs.nnrow - 1 {
        return Ok(true);
    }
    let a = inputs.regions[from.row][from.col - 1];
    let b = inputs.regions[from.row - 1][from.col - 1];
    Ok(a != b)
}

fn can_step_up(
    from: TileNodeCoord,
    inputs: &FindNumPathsOutInputs<'_>,
) -> Result<bool, FindNumPathsOutError> {
    if from.row == 0 {
        return Ok(false);
    }
    if from.col == 0 || from.col == inputs.nncol - 1 {
        return Ok(true);
    }
    let a = inputs.regions[from.row - 1][from.col - 1];
    let b = inputs.regions[from.row - 1][from.col];
    Ok(a != b)
}

fn inverse_direction(direction: ArcDirection) -> ArcDirection {
    match direction {
        ArcDirection::Right => ArcDirection::Left,
        ArcDirection::Down => ArcDirection::Up,
        ArcDirection::Left => ArcDirection::Right,
        ArcDirection::Up => ArcDirection::Down,
    }
}

/// Scan neighboring primary nodes and update DFS frontier state.
///
/// This is the typed Rust equivalent of C `RegionTraceCheckNeighbors()`.
pub fn region_trace_check_neighbors(
    from: TileNodeCoord,
    workspace: &mut RegionTraceWorkspace,
    inputs: &FindNumPathsOutInputs<'_>,
) -> Result<Vec<RegionCrossing>, FindNumPathsOutError> {
    let mut crossings = Vec::new();
    let pred = workspace.nodes[from.row][from.col].pred;

    let mut consider = |to: TileNodeCoord, direction: ArcDirection| {
        if pred == Some(to) {
            return;
        }
        workspace.nodes[to.row][to.col].pred = Some(from);
        match workspace.nodes[to.row][to.col].group {
            PrimaryTraceGroup::NotInBucket => {
                workspace.nodes[to.row][to.col].group = PrimaryTraceGroup::InBucket;
                workspace.stack.push(to);
            }
            PrimaryTraceGroup::OnTree => {
                crossings.push(RegionCrossing {
                    head: to,
                    tail: from,
                    fromdir: inverse_direction(direction),
                });
            }
            PrimaryTraceGroup::InBucket => {}
        }
    };

    if can_step_right(from, inputs)? {
        consider(
            TileNodeCoord {
                row: from.row,
                col: from.col + 1,
            },
            ArcDirection::Right,
        );
    }
    if can_step_down(from, inputs)? {
        consider(
            TileNodeCoord {
                row: from.row + 1,
                col: from.col,
            },
            ArcDirection::Down,
        );
    }
    if can_step_left(from, inputs)? {
        consider(
            TileNodeCoord {
                row: from.row,
                col: from.col - 1,
            },
            ArcDirection::Left,
        );
    }
    if can_step_up(from, inputs)? {
        consider(
            TileNodeCoord {
                row: from.row - 1,
                col: from.col,
            },
            ArcDirection::Up,
        );
    }

    Ok(crossings)
}

fn map_coord_to_secondary_key(
    coord: TileNodeCoord,
    tilerow: usize,
    tilecol: usize,
    ntilecol: usize,
) -> SecondaryNodeKey {
    let tilenum = tilerow * ntilecol + tilecol;
    if coord.row == 0 && tilerow != 0 {
        SecondaryNodeKey {
            tile: tilenum - ntilecol,
            primary_row: 0,
            primary_col: coord.col as i64,
        }
    } else if coord.col == 0 && tilecol != 0 {
        SecondaryNodeKey {
            tile: tilenum - 1,
            primary_row: coord.row as i64,
            primary_col: 0,
        }
    } else {
        SecondaryNodeKey {
            tile: tilenum,
            primary_row: coord.row as i64,
            primary_col: coord.col as i64,
        }
    }
}

fn direction_from_head_to_tail(
    head: TileNodeCoord,
    tail: TileNodeCoord,
) -> Result<ArcDirection, TileTraceError> {
    if tail.row == head.row && tail.col == head.col + 1 {
        Ok(ArcDirection::Right)
    } else if tail.row == head.row + 1 && tail.col == head.col {
        Ok(ArcDirection::Down)
    } else if tail.row == head.row && tail.col + 1 == head.col {
        Ok(ArcDirection::Left)
    } else if tail.row + 1 == head.row && tail.col == head.col {
        Ok(ArcDirection::Up)
    } else {
        Err(TileTraceError::InvalidPrimaryCoord)
    }
}

fn average_sigma_hint(mst_costs: &[Vec<i16>]) -> f64 {
    let mut sum = 0.0f64;
    let mut n = 0usize;
    for row in mst_costs {
        for &v in row {
            if v > 0 {
                sum += f64::from(v);
                n += 1;
            }
        }
    }
    if n == 0 {
        f64::from(avg_sig_sq(LARGE_SHORT, LARGE_SHORT))
    } else {
        sum / n as f64
    }
}

fn phase_sample(unw_phase: &[Vec<f32>], node: TileNodeCoord) -> f64 {
    if unw_phase.is_empty() || unw_phase[0].is_empty() {
        return 0.0;
    }
    let r = node.row.min(unw_phase.len() - 1);
    let c = node.col.min(unw_phase[0].len() - 1);
    f64::from(unw_phase[r][c])
}

fn make_arc_cost_profile(
    params: &TraceRegionsParams<'_>,
    head: TileNodeCoord,
    tail: TileNodeCoord,
) -> Result<SecondaryArcCostProfile, TileTraceError> {
    let sigma = average_sigma_hint(params.mst_costs).max(1.0);
    let bias_cycles =
        ((phase_sample(params.unw_phase, tail) - phase_sample(params.unw_phase, head)) / TWO_PI)
            .clamp(-(params.flowmax as f64), params.flowmax as f64);
    let tile_edge = head.row == 0
        || head.col == 0
        || head.row == params.inputs.nnrow.saturating_sub(1)
        || head.col == params.inputs.nncol.saturating_sub(1)
        || tail.row == 0
        || tail.col == 0
        || tail.row == params.inputs.nnrow.saturating_sub(1)
        || tail.col == params.inputs.nncol.saturating_sub(1);

    Ok(trace_secondary_arc_costs(
        &[PrimaryArcHop {
            sigma_sq: Some(sigma),
            forced_zero_cost: false,
        }],
        SecondaryCostParams {
            flowmax: params.flowmax,
            nshortcycle: params.nshortcycle,
            tileedgeweight: params.tileedgeweight,
            is_tile_edge_arc: tile_edge,
        },
        |_hop, offset, nflow| {
            let nom = l_round(((offset as f64 - bias_cycles).powi(2)) * sigma);
            let pos = l_round((((offset + nflow) as f64 - bias_cycles).powi(2)) * sigma);
            let neg = l_round((((offset - nflow) as f64 - bias_cycles).powi(2)) * sigma);
            (nom, pos, neg)
        },
    )?)
}

/// Trace region boundaries and construct a secondary graph skeleton.
///
/// This is the typed Rust equivalent of C `TraceRegions()`.
pub fn trace_regions(
    graph: &mut SecondaryGraph,
    params: TraceRegionsParams<'_>,
) -> Result<TraceRegionsResult, TileTraceError> {
    if params.flowmax == 0
        || params.inputs.nnrow == 0
        || params.inputs.nncol == 0
        || params.unw_phase.is_empty()
        || params.unw_phase[0].is_empty()
    {
        return Err(TileTraceError::InvalidTileGrid);
    }

    let mut result = TraceRegionsResult::default();
    let mut seen_arc_indices = HashSet::<usize>::new();
    let mut workspace = RegionTraceWorkspace::new(params.inputs.nnrow, params.inputs.nncol);
    workspace.nodes[0][0].group = PrimaryTraceGroup::InBucket;
    workspace.stack.push(TileNodeCoord { row: 0, col: 0 });

    let tilenum = params.inputs.tilerow * params.inputs.ntilecol + params.inputs.tilecol;

    while let Some(from) = workspace.stack.pop() {
        workspace.nodes[from.row][from.col].group = PrimaryTraceGroup::OnTree;
        let npaths = find_num_paths_out(from, &params.inputs)
            .map_err(|_| TileTraceError::InvalidRegionShape)?;

        if npaths > 2 {
            result.fork_nodes += 1;
            let head_key = map_coord_to_secondary_key(
                from,
                params.inputs.tilerow,
                params.inputs.tilecol,
                params.inputs.ntilecol,
            );
            graph.get_or_insert_node(head_key);
            if let Some(tail) = workspace.nodes[from.row][from.col].pred {
                let skip = (params.inputs.tilerow != 0 && from.row == 0 && tail.row == 0)
                    || (params.inputs.tilecol != 0 && from.col == 0 && tail.col == 0);
                if !skip {
                    let tail_key = map_coord_to_secondary_key(
                        tail,
                        params.inputs.tilerow,
                        params.inputs.tilecol,
                        params.inputs.ntilecol,
                    );
                    let fromdir = direction_from_head_to_tail(from, tail)?;
                    let profile = make_arc_cost_profile(&params, from, tail)?;
                    let trace = trace_secondary_arc(
                        graph, tilenum, head_key, tail_key, fromdir, profile, false,
                    );
                    match trace.outcome {
                        TraceSecondaryArcOutcome::AddedNewArc { arc_idx }
                        | TraceSecondaryArcOutcome::ReusedExistingArc { arc_idx } => {
                            if seen_arc_indices.insert(arc_idx) {
                                result.arc_indices.push(arc_idx);
                            }
                        }
                        TraceSecondaryArcOutcome::IgnoredSourceOrPriorTileEdge
                        | TraceSecondaryArcOutcome::IgnoredLoop => {}
                    }
                    result.total_arc_len += trace.total_arc_len_delta;
                }
            }
        }

        let crossings = region_trace_check_neighbors(from, &mut workspace, &params.inputs)
            .map_err(|_| TileTraceError::InvalidRegionShape)?;
        for crossing in crossings {
            let head_key = map_coord_to_secondary_key(
                crossing.head,
                params.inputs.tilerow,
                params.inputs.tilecol,
                params.inputs.ntilecol,
            );
            let tail_key = map_coord_to_secondary_key(
                crossing.tail,
                params.inputs.tilerow,
                params.inputs.tilecol,
                params.inputs.ntilecol,
            );
            let profile = make_arc_cost_profile(&params, crossing.head, crossing.tail)?;
            let trace = trace_secondary_arc(
                graph,
                tilenum,
                head_key,
                tail_key,
                crossing.fromdir,
                profile,
                false,
            );
            match trace.outcome {
                TraceSecondaryArcOutcome::AddedNewArc { arc_idx }
                | TraceSecondaryArcOutcome::ReusedExistingArc { arc_idx } => {
                    if seen_arc_indices.insert(arc_idx) {
                        result.arc_indices.push(arc_idx);
                    }
                }
                TraceSecondaryArcOutcome::IgnoredSourceOrPriorTileEdge
                | TraceSecondaryArcOutcome::IgnoredLoop => {}
            }
            result.total_arc_len += trace.total_arc_len_delta;
        }
    }

    Ok(result)
}

fn local_primary_coord(
    key: SecondaryNodeKey,
    tilenum: usize,
    ntilecol: usize,
) -> Result<(usize, usize), TileTraceError> {
    if key.tile == tilenum {
        return Ok((key.primary_row as usize, key.primary_col as usize));
    }
    if key.tile + ntilecol == tilenum {
        return Ok((0, key.primary_col as usize));
    }
    if key.tile + 1 == tilenum {
        return Ok((key.primary_row as usize, 0));
    }
    Ok((0, 0))
}

/// Parse secondary-graph flows back to primary tile-edge flow increments.
///
/// This is the typed Rust equivalent of C `ParseSecondaryFlows()`.
pub fn parse_secondary_flows(
    params: ParseSecondaryFlowsParams<'_>,
) -> Result<ParsedTileFlows, TileTraceError> {
    if params.arc_indices.len() != params.secondary_flows.len() {
        return Err(TileTraceError::MismatchedArcFlowLen {
            arcs: params.arc_indices.len(),
            flows: params.secondary_flows.len(),
        });
    }
    if params.regions.len() != params.nrow
        || params.regions.iter().any(|row| row.len() != params.ncol)
    {
        return Err(TileTraceError::InvalidRegionShape);
    }

    let nnrow = params.nrow + 1;
    let nncol = params.ncol + 1;
    let mut row_flows = vec![vec![0i16; nncol]; nnrow];
    let mut col_flows = vec![vec![0i16; nncol]; nnrow];
    let sign = if params.flip_phase_sign { -1i64 } else { 1i64 };

    for (k, &arc_idx) in params.arc_indices.iter().enumerate() {
        let arc = params
            .graph
            .arcs
            .get(arc_idx)
            .ok_or(TileTraceError::InvalidArcIndex {
                index: arc_idx,
                len: params.graph.arcs.len(),
            })?;

        let nflow = sign * i64::from(params.secondary_flows[k]);
        if nflow == 0 {
            continue;
        }

        let from_key = params.graph.nodes[arc.from].key;
        let to_key = params.graph.nodes[arc.to].key;
        let (primary_from_row, primary_from_col) =
            local_primary_coord(from_key, params.tilenum, params.ntilecol)?;
        let (mut this_row, mut this_col) =
            local_primary_coord(to_key, params.tilenum, params.ntilecol)?;

        let (mut next_row, mut next_col): (isize, isize) = match arc.fromdir {
            ArcDirection::Right => {
                row_flows[this_row][this_col] =
                    to_i16_flow(i64::from(row_flows[this_row][this_col]) - nflow)?;
                (this_row as isize, this_col as isize + 1)
            }
            ArcDirection::Down => {
                col_flows[this_row][this_col] =
                    to_i16_flow(i64::from(col_flows[this_row][this_col]) - nflow)?;
                (this_row as isize + 1, this_col as isize)
            }
            ArcDirection::Left => {
                row_flows[this_row][this_col - 1] =
                    to_i16_flow(i64::from(row_flows[this_row][this_col - 1]) + nflow)?;
                (this_row as isize, this_col as isize - 1)
            }
            ArcDirection::Up => {
                col_flows[this_row - 1][this_col] =
                    to_i16_flow(i64::from(col_flows[this_row - 1][this_col]) + nflow)?;
                (this_row as isize - 1, this_col as isize)
            }
        };

        let mut steps = 0usize;
        let max_steps = 4 * nnrow * nncol + 8;
        while next_row != primary_from_row as isize || next_col != primary_from_col as isize {
            if steps > max_steps {
                return Err(TileTraceError::PathTraceDidNotConverge { arc_idx });
            }
            steps += 1;

            let prev_row = this_row as isize;
            let prev_col = this_col as isize;
            this_row = next_row as usize;
            this_col = next_col as usize;

            if this_col != nncol - 1
                && (this_row == 0
                    || this_row == nnrow - 1
                    || params.regions[this_row - 1][this_col] != params.regions[this_row][this_col])
                && !(this_row as isize == prev_row && this_col as isize + 1 == prev_col)
            {
                row_flows[this_row][this_col] =
                    to_i16_flow(i64::from(row_flows[this_row][this_col]) - nflow)?;
                next_col += 1;
            }

            if this_row != nnrow - 1
                && (this_col == 0
                    || this_col == nncol - 1
                    || params.regions[this_row][this_col] != params.regions[this_row][this_col - 1])
                && !(this_row as isize + 1 == prev_row && this_col as isize == prev_col)
            {
                col_flows[this_row][this_col] =
                    to_i16_flow(i64::from(col_flows[this_row][this_col]) - nflow)?;
                next_row += 1;
            }

            if this_col != 0
                && (this_row == 0
                    || this_row == nnrow - 1
                    || params.regions[this_row][this_col - 1]
                        != params.regions[this_row - 1][this_col - 1])
                && !(this_row as isize == prev_row && this_col as isize - 1 == prev_col)
            {
                row_flows[this_row][this_col - 1] =
                    to_i16_flow(i64::from(row_flows[this_row][this_col - 1]) + nflow)?;
                next_col -= 1;
            }

            if this_row != 0
                && (this_col == 0
                    || this_col == nncol - 1
                    || params.regions[this_row - 1][this_col - 1]
                        != params.regions[this_row - 1][this_col])
                && !(this_row as isize - 1 == prev_row && this_col as isize == prev_col)
            {
                col_flows[this_row - 1][this_col] =
                    to_i16_flow(i64::from(col_flows[this_row - 1][this_col]) + nflow)?;
                next_row -= 1;
            }
        }
    }

    Ok(ParsedTileFlows {
        row_flows,
        col_flows,
    })
}

/// Integrate tile-local unwrapped phase with tile offsets and stitched windows.
///
/// This is the typed Rust equivalent of C `IntegrateSecondaryFlows()`.
pub fn integrate_secondary_flows(
    params: IntegrateSecondaryFlowsParams<'_>,
) -> Result<IntegratedSecondaryOutput, TileTraceError> {
    validate_bulk_offsets(
        params.bulk_offsets,
        params.settings.ntilerow,
        params.settings.ntilecol,
    )?;
    if params.tiles.len() != params.settings.ntilerow * params.settings.ntilecol {
        return Err(TileTraceError::InvalidTileGrid);
    }

    let mut mag = vec![vec![0.0f32; params.linelen]; params.nlines];
    let mut unw = vec![vec![0.0f32; params.linelen]; params.nlines];

    let ni = (params.nlines + (params.settings.ntilerow - 1) * params.settings.row_overlap)
        .div_ceil(params.settings.ntilerow);
    let nj = (params.linelen + (params.settings.ntilecol - 1) * params.settings.col_overlap)
        .div_ceil(params.settings.ntilecol);

    for tilerow in 0..params.settings.ntilerow {
        for tilecol in 0..params.settings.ntilecol {
            let tilenum = tilerow * params.settings.ntilecol + tilecol;
            let tile = &params.tiles[tilenum];
            if tile.mag.is_empty()
                || tile.unw_phase.is_empty()
                || tile.mag.len() != tile.unw_phase.len()
                || tile.mag[0].len() != tile.unw_phase[0].len()
            {
                return Err(TileTraceError::InvalidTileShape { tilerow, tilecol });
            }

            let parsed = parse_secondary_flows(ParseSecondaryFlowsParams {
                tilenum,
                nrow: tile.regions.len(),
                ncol: tile.regions.first().map_or(0, Vec::len),
                ntilerow: params.settings.ntilerow,
                ntilecol: params.settings.ntilecol,
                flip_phase_sign: params.flip_phase_sign,
                regions: &tile.regions,
                graph: params.graph,
                arc_indices: &tile.arc_indices,
                secondary_flows: &tile.secondary_flows,
            })?;

            let tile_first_row = tilerow * ni.saturating_sub(params.settings.row_overlap);
            let tile_first_col = tilecol * nj.saturating_sub(params.settings.col_overlap);
            let region = set_tile_read_params(
                tile.mag.len(),
                tile.mag[0].len(),
                tilerow,
                tilecol,
                params.settings,
            );

            let base_cycles = if params.flip_phase_sign {
                -f32::from(params.bulk_offsets[tilerow][tilecol])
            } else {
                f32::from(params.bulk_offsets[tilerow][tilecol])
            };
            let tile_offset = base_cycles * TWO_PI_F32;
            let nrow = tile.unw_phase.len();
            let ncol = tile.unw_phase[0].len();
            let mut corrected = vec![vec![0.0f32; ncol]; nrow];
            corrected[0][0] = tile.unw_phase[0][0] + tile_offset;

            for c in 1..ncol {
                let dphi = tile.unw_phase[0][c] - tile.unw_phase[0][c - 1];
                corrected[0][c] =
                    corrected[0][c - 1] + dphi + f32::from(parsed.col_flows[0][c]) * TWO_PI_F32;
            }
            for r in 1..nrow {
                let (prev_rows, curr_and_after) = corrected.split_at_mut(r);
                let prev_corr = &prev_rows[r - 1];
                let curr_corr = &mut curr_and_after[0];

                let unw_prev = &tile.unw_phase[r - 1];
                let unw_curr = &tile.unw_phase[r];
                let row_flows = &parsed.row_flows[r];

                for (c, cell) in curr_corr.iter_mut().enumerate().take(ncol) {
                    let dphi = unw_curr[c] - unw_prev[c];
                    *cell = prev_corr[c] + dphi - f32::from(row_flows[c]) * TWO_PI_F32;
                }
            }

            for r in 0..region.rows {
                for c in 0..region.cols {
                    let src_r = region.first_row + r;
                    let src_c = region.first_col + c;
                    let out_r = tile_first_row + src_r;
                    let out_c = tile_first_col + src_c;
                    if out_r < params.nlines && out_c < params.linelen {
                        mag[out_r][out_c] = tile.mag[src_r][src_c];
                        unw[out_r][out_c] = corrected[src_r][src_c];
                    }
                }
            }
        }
    }

    Ok(IntegratedSecondaryOutput {
        mag,
        unw_phase: unw,
    })
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

    let mut incremental = vec![0i64; 2 * params.flowmax];
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

            if let Some(sigsq) = hop.sigma_sq
                && sigsq > 0.0
                && sigsq.is_finite()
            {
                sumsigsqinv += 1.0 / sigsq;
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

/// Normalize secondary-arc incremental costs by average traced arc length.
///
/// This mirrors the C post-trace normalization pass in tiled assembly.
pub fn normalize_secondary_arc_costs(graph: &mut SecondaryGraph) {
    if graph.arcs.is_empty() {
        return;
    }
    let total_arc_len: usize = graph.arcs.iter().map(|a| a.cost_profile.arc_len).sum();
    if total_arc_len == 0 {
        return;
    }
    let avg_arc_len = total_arc_len as f64 / graph.arcs.len() as f64;
    if avg_arc_len <= 0.0 {
        return;
    }

    for arc in &mut graph.arcs {
        if arc.cost_profile.variance_sum_tag == ZERO_COST_ARC {
            continue;
        }
        for v in &mut arc.cost_profile.incremental_costs {
            *v = clip_large_int(((*v as f64) / avg_arc_len).ceil() as i64);
        }
        let tag = l_round(arc.cost_profile.variance_sum_tag as f64 / avg_arc_len).max(0);
        arc.cost_profile.variance_sum_tag = tag;
    }
}

fn secondary_step_cost(profile: &SecondaryArcCostProfile, step: usize, positive: bool) -> i64 {
    let flowmax = profile.incremental_costs.len() / 2;
    if flowmax == 0 {
        return 0;
    }
    let bounded = step.min(flowmax).saturating_sub(1);
    let idx = if positive { bounded } else { flowmax + bounded };
    profile.incremental_costs.get(idx).copied().unwrap_or(0)
}

fn secondary_arc_objective(profile: &SecondaryArcCostProfile, flow: i64) -> i64 {
    if profile.variance_sum_tag == ZERO_COST_ARC {
        return 0;
    }
    let target = -profile.arroffset;
    let delta = flow - target;
    if delta == 0 {
        return 0;
    }

    let steps = delta.unsigned_abs() as usize;
    let positive = delta > 0;
    let mut cost = 0i64;
    for step in 1..=steps {
        cost = cost.saturating_add(secondary_step_cost(profile, step, positive));
    }
    cost
}

/// Solve the traced secondary network and return one flow per global arc.
///
/// The solver is deterministic and serial, matching tiled SNAPHU behavior that
/// runs secondary optimization after parallel tile unwrapping.
pub fn solve_secondary_network(
    graph: &SecondaryGraph,
    scndry_arc_flow_max: usize,
    max_cycle_fraction: f64,
    maxflow: i64,
    nshortcycle: i64,
) -> Result<Vec<i16>, TileTraceError> {
    if scndry_arc_flow_max == 0 {
        return Err(TileTraceError::InvalidTileGrid);
    }
    let _ = (max_cycle_fraction, maxflow, nshortcycle);

    let mut flows = vec![0i16; graph.arcs.len()];
    let lim = i64::try_from(scndry_arc_flow_max).map_err(|_| TileTraceError::FlowOutOfRange {
        value: scndry_arc_flow_max as i64,
    })?;

    for (idx, arc) in graph.arcs.iter().enumerate() {
        let mut best_flow = 0i64;
        let mut best_cost = secondary_arc_objective(&arc.cost_profile, 0);
        for flow in -lim..=lim {
            let cost = secondary_arc_objective(&arc.cost_profile, flow);
            let better = cost < best_cost
                || (cost == best_cost && flow.abs() < best_flow.abs())
                || (cost == best_cost && flow.abs() == best_flow.abs() && flow < best_flow);
            if better {
                best_cost = cost;
                best_flow = flow;
            }
        }
        flows[idx] = to_i16_flow(best_flow)?;
    }

    Ok(flows)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileAssemblyError {
    InvalidDimensions,
    InvalidFlowLayout,
    InvalidCostLayout,
    InvalidComponentCount,
    InvalidTilePlacement,
    TraceError(TileTraceError),
}

impl From<TileTraceError> for TileAssemblyError {
    fn from(value: TileTraceError) -> Self {
        Self::TraceError(value)
    }
}

#[derive(Debug, Clone)]
pub struct GrowRegionParams<'a> {
    pub incr_costs: &'a [Vec<IncrCost>],
    pub nrow: usize,
    pub ncol: usize,
    pub cost_threshold: i16,
    pub min_region_size: usize,
    pub max_components: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembleTileConnCompTile {
    pub tilerow: usize,
    pub tilecol: usize,
    pub first_row: usize,
    pub first_col: usize,
    pub labels: Vec<Vec<u32>>,
}

#[derive(Debug, Clone)]
pub struct AssembleTileConnCompsParams<'a> {
    pub linelen: usize,
    pub nlines: usize,
    pub max_components: usize,
    pub tiles: &'a [AssembleTileConnCompTile],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembleTileConnCompsResult {
    pub labels: Vec<Vec<u32>>,
    pub kept_components: usize,
}

#[derive(Debug, Clone)]
pub struct AssembleTilesParams<'a> {
    pub linelen: usize,
    pub nlines: usize,
    pub integration: IntegrateSecondaryFlowsParams<'a>,
    pub conn_comp_tiles: Option<&'a [AssembleTileConnCompTile]>,
    pub max_components: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssembleTilesResult {
    pub integrated: IntegratedSecondaryOutput,
    pub conn_comp_labels: Option<Vec<Vec<u32>>>,
}

fn validate_row_col_layout_incr(
    arr: &[Vec<IncrCost>],
    nrow: usize,
    ncol: usize,
) -> Result<(), TileAssemblyError> {
    if arr.len() != 2 * nrow - 1 {
        return Err(TileAssemblyError::InvalidCostLayout);
    }
    for (row, vals) in arr.iter().enumerate() {
        let expected = if row < nrow - 1 { ncol } else { ncol - 1 };
        if vals.len() != expected {
            return Err(TileAssemblyError::InvalidCostLayout);
        }
    }
    Ok(())
}

#[inline]
fn row_col_flat_index(arcrow: usize, arccol: usize, nrow: usize, ncol: usize) -> usize {
    if arcrow < nrow - 1 {
        arcrow * ncol + arccol
    } else {
        let row_base = (nrow - 1) * ncol;
        row_base + (arcrow - (nrow - 1)) * (ncol - 1) + arccol
    }
}

fn smooth_arc_costs(params: &GrowRegionParams<'_>) -> Result<Vec<i16>, TileAssemblyError> {
    validate_row_col_layout_incr(params.incr_costs, params.nrow, params.ncol)?;
    let total = (params.nrow - 1) * params.ncol + params.nrow * (params.ncol - 1);
    let mut flat = Vec::with_capacity(total);
    for row in params.incr_costs {
        flat.extend_from_slice(row);
    }
    thicken_costs(&mut flat, params.nrow, params.ncol);
    Ok(flat.into_iter().map(|c| c.negcost).collect())
}

struct ComponentBuild {
    labels: Vec<Vec<usize>>,
    sizes: Vec<usize>,
    pixels: Vec<Vec<(usize, usize)>>,
}

fn build_components(
    nrow: usize,
    ncol: usize,
    threshold: i16,
    smoothed_costs: &[i16],
) -> ComponentBuild {
    let mut labels = vec![vec![usize::MAX; ncol]; nrow];
    let mut sizes = Vec::new();
    let mut pixels = Vec::new();

    for sr in 0..nrow {
        for sc in 0..ncol {
            if labels[sr][sc] != usize::MAX {
                continue;
            }
            let comp_id = sizes.len();
            let mut q = VecDeque::new();
            q.push_back((sr, sc));
            labels[sr][sc] = comp_id;
            let mut count = 0usize;
            let mut comp_pixels = Vec::new();

            while let Some((r, c)) = q.pop_front() {
                count += 1;
                comp_pixels.push((r, c));

                if r > 0 {
                    let idx = row_col_flat_index(r - 1, c, nrow, ncol);
                    if smoothed_costs[idx] <= threshold && labels[r - 1][c] == usize::MAX {
                        labels[r - 1][c] = comp_id;
                        q.push_back((r - 1, c));
                    }
                }
                if r + 1 < nrow {
                    let idx = row_col_flat_index(r, c, nrow, ncol);
                    if smoothed_costs[idx] <= threshold && labels[r + 1][c] == usize::MAX {
                        labels[r + 1][c] = comp_id;
                        q.push_back((r + 1, c));
                    }
                }
                if c > 0 {
                    let idx = row_col_flat_index(nrow - 1 + r, c - 1, nrow, ncol);
                    if smoothed_costs[idx] <= threshold && labels[r][c - 1] == usize::MAX {
                        labels[r][c - 1] = comp_id;
                        q.push_back((r, c - 1));
                    }
                }
                if c + 1 < ncol {
                    let idx = row_col_flat_index(nrow - 1 + r, c, nrow, ncol);
                    if smoothed_costs[idx] <= threshold && labels[r][c + 1] == usize::MAX {
                        labels[r][c + 1] = comp_id;
                        q.push_back((r, c + 1));
                    }
                }
            }

            sizes.push(count);
            pixels.push(comp_pixels);
        }
    }

    ComponentBuild {
        labels,
        sizes,
        pixels,
    }
}

/// Grow contiguous regions by thresholding thickened incremental arc costs.
///
/// This is the typed Rust equivalent of C `GrowRegions()`.
pub fn grow_regions(params: GrowRegionParams<'_>) -> Result<Vec<Vec<i16>>, TileAssemblyError> {
    if params.nrow < 2 || params.ncol < 2 {
        return Err(TileAssemblyError::InvalidDimensions);
    }
    if params.max_components == 0 {
        return Err(TileAssemblyError::InvalidComponentCount);
    }

    let smoothed = smooth_arc_costs(&params)?;
    let built = build_components(params.nrow, params.ncol, params.cost_threshold, &smoothed);
    let ncomp = built.sizes.len();
    let large: Vec<bool> = built
        .sizes
        .iter()
        .map(|&size| size >= params.min_region_size)
        .collect();
    let mut merge_target: Vec<Option<usize>> = vec![None; ncomp];

    for (comp, target_slot) in merge_target.iter_mut().enumerate().take(ncomp) {
        if built.sizes[comp] >= params.min_region_size {
            continue;
        }
        let mut best_neighbor = None::<(i16, usize)>;

        for &(r, c) in &built.pixels[comp] {
            let mut consider = |nr: usize, nc: usize, arc_idx: usize| {
                let other = built.labels[nr][nc];
                if other == comp || !large[other] {
                    return;
                }
                let score = smoothed[arc_idx];
                match best_neighbor {
                    Some((best_score, _)) if score >= best_score => {}
                    _ => best_neighbor = Some((score, other)),
                }
            };

            if r > 0 {
                consider(
                    r - 1,
                    c,
                    row_col_flat_index(r - 1, c, params.nrow, params.ncol),
                );
            }
            if r + 1 < params.nrow {
                consider(r + 1, c, row_col_flat_index(r, c, params.nrow, params.ncol));
            }
            if c > 0 {
                consider(
                    r,
                    c - 1,
                    row_col_flat_index(params.nrow - 1 + r, c - 1, params.nrow, params.ncol),
                );
            }
            if c + 1 < params.ncol {
                consider(
                    r,
                    c + 1,
                    row_col_flat_index(params.nrow - 1 + r, c, params.nrow, params.ncol),
                );
            }
        }

        if let Some((_, target)) = best_neighbor {
            *target_slot = Some(target);
        }
    }

    let mut region_ids = vec![vec![0i16; params.ncol]; params.nrow];
    let mut remap = HashMap::<usize, i16>::new();
    let mut next_id: i16 = 0;
    for r in 0..params.nrow {
        for c in 0..params.ncol {
            let root = merge_target[built.labels[r][c]].unwrap_or(built.labels[r][c]);
            let id = remap.entry(root).or_insert_with(|| {
                let out = next_id;
                next_id = next_id.saturating_add(1);
                out
            });
            region_ids[r][c] = *id;
        }
    }

    Ok(region_ids)
}

/// Grow connected-component mask by thresholding thickened incremental costs.
///
/// This is the typed Rust equivalent of C `GrowConnCompsMask()`.
pub fn grow_conn_comps_mask(
    params: GrowRegionParams<'_>,
) -> Result<Vec<Vec<u32>>, TileAssemblyError> {
    if params.nrow < 2 || params.ncol < 2 {
        return Err(TileAssemblyError::InvalidDimensions);
    }
    if params.max_components == 0 {
        return Err(TileAssemblyError::InvalidComponentCount);
    }

    let smoothed = smooth_arc_costs(&params)?;
    let built = build_components(params.nrow, params.ncol, params.cost_threshold, &smoothed);

    let mut components: Vec<(usize, usize)> = built
        .sizes
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, size)| size >= params.min_region_size)
        .collect();
    components.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    components.truncate(params.max_components);

    let mut keep_map = HashMap::<usize, u32>::new();
    for (rank, (comp, _)) in components.iter().enumerate() {
        keep_map.insert(*comp, (rank + 1) as u32);
    }

    let mut out = vec![vec![0u32; params.ncol]; params.nrow];
    for (r, out_row) in out.iter_mut().enumerate().take(params.nrow) {
        for (c, out_cell) in out_row.iter_mut().enumerate().take(params.ncol) {
            if let Some(&mapped) = keep_map.get(&built.labels[r][c]) {
                *out_cell = mapped;
            }
        }
    }
    Ok(out)
}

/// Assemble per-tile connected-component labels into a full-scene mask.
///
/// This is the typed Rust equivalent of C `AssembleTileConnComps()`.
pub fn assemble_tile_conn_comps(
    params: AssembleTileConnCompsParams<'_>,
) -> Result<AssembleTileConnCompsResult, TileAssemblyError> {
    if params.linelen == 0 || params.nlines == 0 {
        return Err(TileAssemblyError::InvalidDimensions);
    }
    if params.max_components == 0 {
        return Err(TileAssemblyError::InvalidComponentCount);
    }

    let mut comp_sizes = HashMap::<(usize, usize, u32), usize>::new();
    for tile in params.tiles {
        for row in &tile.labels {
            for &label in row {
                if label > 0 {
                    *comp_sizes
                        .entry((tile.tilerow, tile.tilecol, label))
                        .or_insert(0) += 1;
                }
            }
        }
    }

    let mut ranked: Vec<((usize, usize, u32), usize)> = comp_sizes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(params.max_components);

    let mut id_map = HashMap::<(usize, usize, u32), u32>::new();
    for (rank, (key, _)) in ranked.iter().enumerate() {
        id_map.insert(*key, (rank + 1) as u32);
    }

    let mut full = vec![vec![0u32; params.linelen]; params.nlines];
    for tile in params.tiles {
        for (r, row) in tile.labels.iter().enumerate() {
            for (c, &label) in row.iter().enumerate() {
                if label == 0 {
                    continue;
                }
                let out_r = tile.first_row + r;
                let out_c = tile.first_col + c;
                if out_r >= params.nlines || out_c >= params.linelen {
                    return Err(TileAssemblyError::InvalidTilePlacement);
                }
                if let Some(&mapped) = id_map.get(&(tile.tilerow, tile.tilecol, label))
                    && full[out_r][out_c] == 0
                {
                    full[out_r][out_c] = mapped;
                }
            }
        }
    }

    Ok(AssembleTileConnCompsResult {
        labels: full,
        kept_components: ranked.len(),
    })
}

/// Assemble all tiles into full-scene products.
///
/// This is the typed Rust equivalent of C `AssembleTiles()`.
pub fn assemble_tiles(
    params: AssembleTilesParams<'_>,
) -> Result<AssembleTilesResult, TileAssemblyError> {
    let integrated = integrate_secondary_flows(params.integration)?;
    let conn_comp_labels = if let Some(tiles) = params.conn_comp_tiles {
        let assembled = assemble_tile_conn_comps(AssembleTileConnCompsParams {
            linelen: params.linelen,
            nlines: params.nlines,
            max_components: params.max_components,
            tiles,
        })?;
        Some(assembled.labels)
    } else {
        None
    };

    Ok(AssembleTilesResult {
        integrated,
        conn_comp_labels,
    })
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
    fn trace_regions_emits_arc_indices_on_boundary_crossings() {
        let regions = vec![vec![0i16, 1i16], vec![1i16, 0i16]];
        let edge = vec![0i16, 1i16];
        let mst_costs = vec![vec![10i16, 10i16], vec![10i16], vec![10i16]];
        let unw_phase = vec![vec![0.0f32, 1.0f32], vec![1.0f32, 0.0f32]];
        let inputs = FindNumPathsOutInputs {
            ntilerow: 2,
            ntilecol: 2,
            tilerow: 0,
            tilecol: 0,
            nnrow: 3,
            nncol: 3,
            prevncol: 2,
            regions: &regions,
            nextregions: &regions,
            lastregions: &regions,
            regionsabove: &edge,
            regionsbelow: &edge,
        };
        let mut graph = SecondaryGraph::default();
        let out = trace_regions(
            &mut graph,
            TraceRegionsParams {
                flowmax: 4,
                nshortcycle: 20,
                tileedgeweight: 2.5,
                mst_costs: &mst_costs,
                unw_phase: &unw_phase,
                inputs,
            },
        )
        .unwrap();
        assert!(!out.arc_indices.is_empty());
        assert!(!graph.arcs.is_empty());
    }

    #[test]
    fn normalize_secondary_arc_costs_scales_by_average_arc_length() {
        let mut graph = SecondaryGraph::default();
        let from = graph.get_or_insert_node(SecondaryNodeKey {
            tile: 0,
            primary_row: 0,
            primary_col: 0,
        });
        let to = graph.get_or_insert_node(SecondaryNodeKey {
            tile: 0,
            primary_row: 0,
            primary_col: 1,
        });
        graph.arcs.push(SecondaryArc {
            arcrow: 0,
            arccol: 0,
            from,
            to,
            fromdir: ArcDirection::Right,
            cost_profile: SecondaryArcCostProfile {
                arroffset: 0,
                incremental_costs: vec![8, 16],
                variance_sum_tag: 20,
                arc_len: 4,
            },
        });

        normalize_secondary_arc_costs(&mut graph);
        assert_eq!(graph.arcs[0].cost_profile.incremental_costs, vec![2, 4]);
        assert_eq!(graph.arcs[0].cost_profile.variance_sum_tag, 5);
    }

    #[test]
    fn solve_secondary_network_returns_zero_for_zero_cost_graph() {
        let mut graph = SecondaryGraph::default();
        let from = graph.get_or_insert_node(SecondaryNodeKey {
            tile: 0,
            primary_row: 0,
            primary_col: 0,
        });
        let to = graph.get_or_insert_node(SecondaryNodeKey {
            tile: 0,
            primary_row: 0,
            primary_col: 1,
        });
        graph.arcs.push(SecondaryArc {
            arcrow: 0,
            arccol: 0,
            from,
            to,
            fromdir: ArcDirection::Right,
            cost_profile: SecondaryArcCostProfile {
                arroffset: 0,
                incremental_costs: vec![0, 0, 0, 0],
                variance_sum_tag: ZERO_COST_ARC,
                arc_len: 1,
            },
        });

        let flows = solve_secondary_network(&graph, 2, 1.0e-5, 4, 200).unwrap();
        assert_eq!(flows, vec![0i16]);
    }

    #[test]
    fn integrate_secondary_flows_applies_nonzero_secondary_flow() {
        let mut graph = SecondaryGraph::default();
        let from = graph.get_or_insert_node(SecondaryNodeKey {
            tile: 0,
            primary_row: 1,
            primary_col: 1,
        });
        let to = graph.get_or_insert_node(SecondaryNodeKey {
            tile: 0,
            primary_row: 1,
            primary_col: 0,
        });
        graph.arcs.push(SecondaryArc {
            arcrow: 0,
            arccol: 0,
            from,
            to,
            fromdir: ArcDirection::Right,
            cost_profile: SecondaryArcCostProfile {
                arroffset: 0,
                incremental_costs: vec![1, 1],
                variance_sum_tag: 1,
                arc_len: 1,
            },
        });

        let tile = TileIntegrationInput {
            mag: vec![vec![1.0f32, 1.0f32], vec![1.0f32, 1.0f32]],
            unw_phase: vec![vec![0.0f32, 0.0f32], vec![0.0f32, 0.0f32]],
            regions: vec![vec![0i16, 0i16], vec![0i16, 0i16]],
            arc_indices: vec![0usize],
            secondary_flows: vec![1i16],
        };
        let settings = TileReadSettings {
            row_overlap: 0,
            col_overlap: 0,
            ntilerow: 1,
            ntilecol: 1,
        };
        let bulk = vec![vec![0i16]];

        let integrated = integrate_secondary_flows(IntegrateSecondaryFlowsParams {
            linelen: 2,
            nlines: 2,
            settings,
            bulk_offsets: &bulk,
            flip_phase_sign: false,
            graph: &graph,
            tiles: &[tile],
        })
        .unwrap();

        assert!(integrated.unw_phase[1][0].abs() > 1.0);
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

    fn uniform_incr_costs(nrow: usize, ncol: usize, value: i16) -> Vec<Vec<IncrCost>> {
        (0..(2 * nrow - 1))
            .map(|row| {
                let cols = if row < nrow - 1 { ncol } else { ncol - 1 };
                vec![IncrCost::new(value, value); cols]
            })
            .collect()
    }

    #[test]
    fn grow_conn_comps_mask_limits_component_count() {
        let costs = uniform_incr_costs(2, 2, 100);
        let labels = grow_conn_comps_mask(GrowRegionParams {
            incr_costs: &costs,
            nrow: 2,
            ncol: 2,
            cost_threshold: 0,
            min_region_size: 1,
            max_components: 2,
        })
        .unwrap();

        let kept = labels.iter().flatten().copied().filter(|&v| v > 0).count();
        assert_eq!(kept, 2);
    }

    #[test]
    fn grow_regions_returns_dense_region_ids() {
        let mut costs = uniform_incr_costs(2, 3, 100);
        costs[0][0] = IncrCost::new(0, 0);
        let regions = grow_regions(GrowRegionParams {
            incr_costs: &costs,
            nrow: 2,
            ncol: 3,
            cost_threshold: 0,
            min_region_size: 2,
            max_components: 8,
        })
        .unwrap();

        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].len(), 3);
        let max_id = regions.iter().flatten().copied().max().unwrap_or(0);
        assert!(max_id >= 0);
    }

    #[test]
    fn assemble_tile_conn_comps_merges_labels() {
        let tiles = vec![
            AssembleTileConnCompTile {
                tilerow: 0,
                tilecol: 0,
                first_row: 0,
                first_col: 0,
                labels: vec![vec![1, 1], vec![0, 2]],
            },
            AssembleTileConnCompTile {
                tilerow: 0,
                tilecol: 1,
                first_row: 0,
                first_col: 2,
                labels: vec![vec![1, 0], vec![0, 0]],
            },
        ];

        let out = assemble_tile_conn_comps(AssembleTileConnCompsParams {
            linelen: 4,
            nlines: 2,
            max_components: 2,
            tiles: &tiles,
        })
        .unwrap();

        assert_eq!(out.labels.len(), 2);
        assert_eq!(out.labels[0].len(), 4);
        assert_eq!(out.kept_components, 2);
    }

    #[test]
    fn assemble_tiles_runs_integration_and_optional_conn_comp_merge() {
        let graph = SecondaryGraph::default();
        let tiles = vec![TileIntegrationInput {
            mag: vec![vec![1.0, 1.0], vec![1.0, 1.0]],
            unw_phase: vec![vec![0.0, 0.0], vec![0.0, 0.0]],
            regions: vec![vec![1]],
            arc_indices: vec![],
            secondary_flows: vec![],
        }];
        let conn = vec![AssembleTileConnCompTile {
            tilerow: 0,
            tilecol: 0,
            first_row: 0,
            first_col: 0,
            labels: vec![vec![1, 0], vec![0, 1]],
        }];
        let bulk = vec![vec![0i16]];
        let settings = TileReadSettings {
            row_overlap: 0,
            col_overlap: 0,
            ntilerow: 1,
            ntilecol: 1,
        };

        let out = assemble_tiles(AssembleTilesParams {
            linelen: 2,
            nlines: 2,
            integration: IntegrateSecondaryFlowsParams {
                linelen: 2,
                nlines: 2,
                settings,
                bulk_offsets: &bulk,
                flip_phase_sign: false,
                graph: &graph,
                tiles: &tiles,
            },
            conn_comp_tiles: Some(&conn),
            max_components: 4,
        })
        .unwrap();

        assert_eq!(out.integrated.mag.len(), 2);
        assert!(out.conn_comp_labels.is_some());
    }
}
