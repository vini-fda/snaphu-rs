//! Flow-based unwrapping modes.

use crate::config::{InputFiles, OutputFiles, RunConfig};
use crate::costs::types::IncrCost;
use crate::costs::{BuildCostArraysResult, CostBuildError, build_cost_arrays};
use crate::data::ops::cycle_residue;
use crate::network::{NetworkCostError, SolveCs2Params, SolveMstParams, solve_cs2, solve_mst};
use crate::unwrapping::tiles::set_tile_init_outfile;
use crate::unwrapping::tiles::{
    GrowRegionParams, TileAssemblyError, grow_conn_comps_mask, grow_regions,
};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileJob {
    pub tilerow: usize,
    pub tilecol: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwrapIterationPlan {
    pub index: usize,
    pub read_from_tile_init: bool,
    pub write_tile_init: bool,
    pub tiles: Vec<TileJob>,
    pub assemble_tiles: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwrapPlan {
    pub iterations: Vec<UnwrapIterationPlan>,
    pub tile_init_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnwrapError {
    InvalidImageDims,
    InvalidTileGrid,
    InvalidOutfile,
}

/// Build the tile/iteration execution plan for unwrapping.
///
/// This is the typed Rust equivalent of C `Unwrap()`.
pub fn unwrap(
    infiles: &InputFiles,
    outfiles: &OutputFiles,
    params: &RunConfig,
    linelen: usize,
    nlines: usize,
) -> Result<UnwrapPlan, UnwrapError> {
    if linelen == 0 || nlines == 0 {
        return Err(UnwrapError::InvalidImageDims);
    }
    if params.ntilerow == 0 || params.ntilecol == 0 {
        return Err(UnwrapError::InvalidTileGrid);
    }
    if outfiles.outfile.is_empty() {
        return Err(UnwrapError::InvalidOutfile);
    }

    let _ = infiles;
    let noptiter = if params.onetilereopt { 2 } else { 1 };
    let mut tile_init_file = None;
    let mut iterations = Vec::with_capacity(noptiter);

    for optiter in 0..noptiter {
        let mut iter_params = params.clone();
        let mut read_from_tile_init = false;
        let mut write_tile_init = false;

        if optiter == 0 && noptiter > 1 {
            let init_path =
                set_tile_init_outfile(Path::new(&outfiles.outfile), params.parent_pid as u32)
                    .map_err(|_| UnwrapError::InvalidOutfile)?;
            tile_init_file = Some(init_path.to_string_lossy().to_string());
            write_tile_init = true;
        } else if optiter == 1 {
            read_from_tile_init = true;
            iter_params.unwrapped = true;
            iter_params.ntilerow = 1;
            iter_params.ntilecol = 1;
            iter_params.rowovrlp = 0;
            iter_params.colovrlp = 0;
        }

        let mut tiles = Vec::new();
        if iter_params.ntilerow == 1 && iter_params.ntilecol == 1 {
            tiles.push(TileJob {
                tilerow: 0,
                tilecol: 0,
            });
        } else if !iter_params.assemble_only {
            for tilerow in 0..iter_params.ntilerow {
                for tilecol in 0..iter_params.ntilecol {
                    tiles.push(TileJob { tilerow, tilecol });
                }
            }
        }

        iterations.push(UnwrapIterationPlan {
            index: optiter,
            read_from_tile_init,
            write_tile_init,
            tiles,
            assemble_tiles: iter_params.ntilerow != 1 || iter_params.ntilecol != 1,
        });
    }

    Ok(UnwrapPlan {
        iterations,
        tile_init_file,
    })
}

#[derive(Debug, Clone)]
pub struct UnwrapTileParams<'a> {
    pub mag: &'a [Vec<f32>],
    pub wrapped_phase: &'a [Vec<f32>],
    pub power: Option<&'a [Vec<f32>]>,
    pub correlation: Option<&'a [Vec<f32>]>,
    pub initial_flows: Option<Vec<Vec<i16>>>,
    pub cost_threshold: i16,
    pub min_region_size: usize,
    pub max_components: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnwrapTileResult {
    pub cost_arrays: BuildCostArraysResult,
    pub flows: Vec<Vec<i16>>,
    pub conn_comp_mask: Option<Vec<Vec<u32>>>,
    pub regions: Option<Vec<Vec<i16>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnwrapTileError {
    InvalidDimensions,
    InvalidInputShape,
    CostBuild(CostBuildError),
    TileAssembly(TileAssemblyError),
    Network(NetworkCostError),
}

impl From<CostBuildError> for UnwrapTileError {
    fn from(value: CostBuildError) -> Self {
        Self::CostBuild(value)
    }
}

impl From<TileAssemblyError> for UnwrapTileError {
    fn from(value: TileAssemblyError) -> Self {
        Self::TileAssembly(value)
    }
}

impl From<NetworkCostError> for UnwrapTileError {
    fn from(value: NetworkCostError) -> Self {
        Self::Network(value)
    }
}

fn row_col_widths(nrow: usize, ncol: usize) -> Vec<usize> {
    (0..(2 * nrow - 1))
        .map(|row| if row < nrow - 1 { ncol } else { ncol - 1 })
        .collect()
}

fn validate_flows_shape(
    flows: &[Vec<i16>],
    nrow: usize,
    ncol: usize,
) -> Result<(), UnwrapTileError> {
    let widths = row_col_widths(nrow, ncol);
    if flows.len() != widths.len() {
        return Err(UnwrapTileError::InvalidInputShape);
    }
    for (row, &w) in widths.iter().enumerate() {
        if flows[row].len() != w {
            return Err(UnwrapTileError::InvalidInputShape);
        }
    }
    Ok(())
}

fn flatten_grid(arr: &[Vec<f32>], nrow: usize, ncol: usize) -> Result<Vec<f32>, UnwrapTileError> {
    if arr.len() != nrow || arr.iter().any(|row| row.len() != ncol) {
        return Err(UnwrapTileError::InvalidInputShape);
    }
    let mut out = Vec::with_capacity(nrow * ncol);
    for row in arr {
        out.extend_from_slice(row);
    }
    Ok(out)
}

fn cycle_residue_2d(
    wrapped_phase: &[Vec<f32>],
    nrow: usize,
    ncol: usize,
) -> Result<Vec<Vec<i8>>, UnwrapTileError> {
    let flat = flatten_grid(wrapped_phase, nrow, ncol)?;
    let residue = cycle_residue(&flat, nrow, ncol);
    let mut out = vec![vec![0i8; ncol - 1]; nrow - 1];
    for row in 0..(nrow - 1) {
        for col in 0..(ncol - 1) {
            out[row][col] = residue[row * (ncol - 1) + col];
        }
    }
    Ok(out)
}

/// Run single-tile unwrapping orchestration.
///
/// This is the typed Rust equivalent of C `UnwrapTile()`.
pub fn unwrap_tile(
    params: UnwrapTileParams<'_>,
    config: &RunConfig,
) -> Result<UnwrapTileResult, UnwrapTileError> {
    if params.mag.is_empty() || params.mag[0].is_empty() {
        return Err(UnwrapTileError::InvalidDimensions);
    }
    let nrow = params.mag.len();
    let ncol = params.mag[0].len();
    if nrow < 2 || ncol < 2 {
        return Err(UnwrapTileError::InvalidDimensions);
    }
    if params.mag.iter().any(|row| row.len() != ncol)
        || params.wrapped_phase.len() != nrow
        || params.wrapped_phase.iter().any(|row| row.len() != ncol)
    {
        return Err(UnwrapTileError::InvalidInputShape);
    }

    let cost_arrays = build_cost_arrays(
        params.mag,
        params.wrapped_phase,
        params.power,
        params.correlation,
        config,
    )?;

    let flows = if let Some(flows) = params.initial_flows {
        validate_flows_shape(&flows, nrow, ncol)?;
        flows
    } else if config.unwrapped {
        row_col_widths(nrow, ncol)
            .into_iter()
            .map(|w| vec![0i16; w])
            .collect()
    } else {
        let residue = cycle_residue_2d(params.wrapped_phase, nrow, ncol)?;
        if log::log_enabled!(log::Level::Debug) {
            let mut pos = 0usize;
            let mut neg = 0usize;
            let mut zero = 0usize;
            let mut sum = 0i64;
            for row in &residue {
                for &v in row {
                    let vi = i64::from(v);
                    sum += vi;
                    if v > 0 {
                        pos += 1;
                    } else if v < 0 {
                        neg += 1;
                    } else {
                        zero += 1;
                    }
                }
            }
            log::debug!(
                "unwrap_tile init={} nrow={} ncol={} residue(pos={}, neg={}, zero={}, sum={})",
                match config.init_method {
                    crate::config::InitMethod::Mst => "mst",
                    crate::config::InitMethod::Mcf => "mcf",
                },
                nrow,
                ncol,
                pos,
                neg,
                zero,
                sum
            );
        }
        match config.init_method {
            crate::config::InitMethod::Mst => {
                solve_mst(SolveMstParams {
                    residue: &residue,
                    mst_costs: &cost_arrays.mst_costs,
                    nrow,
                    ncol,
                })?
                .flows
            }
            crate::config::InitMethod::Mcf => solve_cs2(SolveCs2Params {
                residue: &residue,
                mst_costs: &cost_arrays.mst_costs,
                nrow,
                ncol,
                cs2_scale_factor: 1,
            })?,
        }
    };
    validate_flows_shape(&flows, nrow, ncol)?;

    let mut incr_costs = Vec::with_capacity(cost_arrays.mst_costs.len());
    for row in &cost_arrays.mst_costs {
        incr_costs.push(row.iter().map(|&c| IncrCost::new(c, c)).collect::<Vec<_>>());
    }

    let grow_params = GrowRegionParams {
        incr_costs: &incr_costs,
        nrow,
        ncol,
        cost_threshold: params.cost_threshold,
        min_region_size: params.min_region_size,
        max_components: params.max_components,
    };

    let conn_comp_mask = if config.regrow_conn_comps {
        Some(grow_conn_comps_mask(grow_params.clone())?)
    } else {
        None
    };

    let regions = if config.ntilerow != 1 || config.ntilecol != 1 {
        Some(grow_regions(grow_params)?)
    } else {
        None
    };

    Ok(UnwrapTileResult {
        cost_arrays,
        flows,
        conn_comp_mask,
        regions,
    })
}

pub fn compute_flow() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwrap_single_tile_has_one_iteration_one_tile() {
        let infiles = InputFiles::default();
        let outfiles = OutputFiles::default();
        let params = RunConfig::default();

        let plan = unwrap(&infiles, &outfiles, &params, 32, 32).unwrap();
        assert_eq!(plan.iterations.len(), 1);
        assert_eq!(plan.iterations[0].tiles.len(), 1);
        assert!(!plan.iterations[0].assemble_tiles);
    }

    #[test]
    fn unwrap_with_one_tile_reopt_has_two_iterations() {
        let infiles = InputFiles::default();
        let outfiles = OutputFiles::default();
        let params = RunConfig {
            onetilereopt: true,
            ntilerow: 2,
            ntilecol: 2,
            ..RunConfig::default()
        };

        let plan = unwrap(&infiles, &outfiles, &params, 64, 64).unwrap();
        assert_eq!(plan.iterations.len(), 2);
        assert!(plan.iterations[0].write_tile_init);
        assert!(plan.iterations[1].read_from_tile_init);
    }

    #[test]
    fn unwrap_tile_builds_costs_and_initializes_flows() {
        let mag = vec![vec![1.0f32, 1.0], vec![1.0, 1.0]];
        let wrapped = vec![vec![0.0f32, 0.1], vec![0.2, 0.3]];
        let cfg = RunConfig {
            unwrapped: true,
            ..RunConfig::default()
        };
        let out = unwrap_tile(
            UnwrapTileParams {
                mag: &mag,
                wrapped_phase: &wrapped,
                power: None,
                correlation: None,
                initial_flows: None,
                cost_threshold: 0,
                min_region_size: 1,
                max_components: 8,
            },
            &cfg,
        )
        .unwrap();
        assert_eq!(out.flows.len(), 3);
        assert_eq!(out.cost_arrays.mst_costs.len(), 3);
    }

    #[test]
    fn unwrap_tile_runs_region_growth_modes() {
        let mag = vec![vec![1.0f32, 1.0], vec![1.0, 1.0]];
        let wrapped = vec![vec![0.0f32, 0.0], vec![0.0, 0.0]];
        let cfg = RunConfig {
            regrow_conn_comps: true,
            ntilerow: 2,
            ntilecol: 1,
            ..RunConfig::default()
        };
        let out = unwrap_tile(
            UnwrapTileParams {
                mag: &mag,
                wrapped_phase: &wrapped,
                power: None,
                correlation: None,
                initial_flows: None,
                cost_threshold: 0,
                min_region_size: 1,
                max_components: 4,
            },
            &cfg,
        )
        .unwrap();
        assert!(out.conn_comp_mask.is_some());
        assert!(out.regions.is_some());
    }
}
