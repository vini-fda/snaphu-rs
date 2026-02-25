#![allow(dead_code)]

//! Multi-tile parallel unwrapping orchestration.
//!
//! This module implements the parallel tile unwrapping pipeline:
//! 1. Precompute tile grid geometry via `setup_tile`.
//! 2. Parallel per-tile unwrap using scoped threads with atomic work queue.
//! 3. Bulk offset propagation across tile borders.
//! 4. Global assembly via `integrate_secondary_flows`.

use crate::config::RunConfig;
use crate::data::ops::integrate_phase;
use crate::data::tile::TileRegion;
use crate::unwrapping::flow::{UnwrapTileParams, unwrap_tile};
use crate::unwrapping::tiles::{
    IntegrateSecondaryFlowsParams, IntegratedSecondaryOutput, SecondaryGraph, TileIntegrationInput,
    TileReadSettings, integrate_secondary_flows, set_lower_edge, set_right_edge,
};
use std::io;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Entry in the precomputed tile plan grid.
#[derive(Debug, Clone)]
pub struct TilePlanEntry {
    pub tilerow: usize,
    pub tilecol: usize,
    pub tilenum: usize,
    pub region: TileRegion,
}

/// Result produced by unwrapping a single tile, stored in the result slots.
#[derive(Debug, Clone)]
pub struct TileResult {
    pub region: TileRegion,
    pub mag: Vec<Vec<f32>>,
    pub unw_phase: Vec<Vec<f32>>,
    pub regions: Vec<Vec<i16>>,
    pub flows: Vec<Vec<i16>>,
}

/// Compute tile region geometry (equivalent to setup_tile's geometry logic).
fn compute_tile_region(
    nlines: usize,
    linelen: usize,
    ntilerow: usize,
    ntilecol: usize,
    rowovrlp: usize,
    colovrlp: usize,
    tilerow: usize,
    tilecol: usize,
) -> TileRegion {
    let ni = (nlines + (ntilerow - 1) * rowovrlp).div_ceil(ntilerow);
    let nj = (linelen + (ntilecol - 1) * colovrlp).div_ceil(ntilecol);

    let first_row = tilerow.saturating_mul(ni.saturating_sub(rowovrlp));
    let first_col = tilecol.saturating_mul(nj.saturating_sub(colovrlp));
    let tile_nrow = if tilerow == ntilerow - 1 {
        nlines.saturating_sub((ntilerow - 1) * ni.saturating_sub(rowovrlp))
    } else {
        ni
    };
    let tile_ncol = if tilecol == ntilecol - 1 {
        linelen.saturating_sub((ntilecol - 1) * nj.saturating_sub(colovrlp))
    } else {
        nj
    };

    TileRegion::new(first_row, first_col, tile_nrow, tile_ncol)
}

/// Build the tile plan grid: one `TilePlanEntry` per tile in raster order.
pub fn build_tile_plan(
    nlines: usize,
    linelen: usize,
    params: &RunConfig,
) -> io::Result<Vec<TilePlanEntry>> {
    let ntilerow = params.ntilerow;
    let ntilecol = params.ntilecol;
    if ntilerow == 0 || ntilecol == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "tile grid dimensions must be non-zero",
        ));
    }

    let mut plan = Vec::with_capacity(ntilerow * ntilecol);
    for tilerow in 0..ntilerow {
        for tilecol in 0..ntilecol {
            let tilenum = tilerow * ntilecol + tilecol;
            let region = compute_tile_region(
                nlines,
                linelen,
                ntilerow,
                ntilecol,
                params.rowovrlp,
                params.colovrlp,
                tilerow,
                tilecol,
            );
            plan.push(TilePlanEntry {
                tilerow,
                tilecol,
                tilenum,
                region,
            });
        }
    }
    Ok(plan)
}

/// Extract a rectangular window from a scene-level `Vec<Vec<f32>>` grid.
fn extract_tile_window(scene: &[Vec<f32>], region: &TileRegion) -> Vec<Vec<f32>> {
    let mut out = Vec::with_capacity(region.rows);
    for r in 0..region.rows {
        let src_row = region.first_row + r;
        let row = &scene[src_row];
        let start = region.first_col;
        let end = start + region.cols;
        out.push(row[start..end].to_vec());
    }
    out
}

fn row_col_widths(nrow: usize, ncol: usize) -> Vec<usize> {
    (0..(2 * nrow - 1))
        .map(|row| if row < nrow - 1 { ncol } else { ncol - 1 })
        .collect()
}

fn row_col_to_flat_flows(flows: &[Vec<i16>], nrow: usize, ncol: usize) -> io::Result<Vec<i16>> {
    let widths = row_col_widths(nrow, ncol);
    if flows.len() != widths.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "row/col flow row count mismatch",
        ));
    }
    let mut out = Vec::with_capacity(widths.iter().sum());
    for (row, &width) in widths.iter().enumerate() {
        if flows[row].len() != width {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "row/col flow width mismatch at row {row}: got {}, expected {width}",
                    flows[row].len()
                ),
            ));
        }
        out.extend_from_slice(&flows[row]);
    }
    Ok(out)
}

fn to_grid_f32(data: &[f32], width: usize) -> Vec<Vec<f32>> {
    data.chunks_exact(width).map(|row| row.to_vec()).collect()
}

/// Run the multi-tile unwrapping pipeline end-to-end.
///
/// Returns the assembled `(mag, unw_phase)` grids at full scene dimensions.
pub fn run_multi_tile(
    mag_grid: &[Vec<f32>],
    wrapped_grid: &[Vec<f32>],
    power_grid: Option<&[Vec<f32>]>,
    corr_grid: Option<&[Vec<f32>]>,
    nlines: usize,
    linelen: usize,
    params: &RunConfig,
) -> io::Result<IntegratedSecondaryOutput> {
    let ntilerow = params.ntilerow;
    let ntilecol = params.ntilecol;
    let ntiles = ntilerow * ntilecol;

    // 1. Build tile plan
    let plan = build_tile_plan(nlines, linelen, params)?;
    assert_eq!(plan.len(), ntiles);

    if params.verbose {
        eprintln!(
            "multi-tile: {}x{} grid ({} tiles), {} threads",
            ntilerow, ntilecol, ntiles, params.nthreads
        );
    }

    // 2. Parallel tile unwrap
    let worker_count = params.nthreads.min(ntiles).max(1);
    let results: Vec<OnceLock<Result<TileResult, String>>> =
        (0..ntiles).map(|_| OnceLock::new()).collect();
    let next_job = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);

    std::thread::scope(|s| {
        for _worker in 0..worker_count {
            let plan = &plan;
            let results = &results;
            let next_job = &next_job;
            let failed = &failed;

            s.spawn(move || {
                loop {
                    if failed.load(Ordering::Relaxed) {
                        break;
                    }
                    let idx = next_job.fetch_add(1, Ordering::Relaxed);
                    if idx >= ntiles {
                        break;
                    }
                    let entry = &plan[idx];
                    let region = &entry.region;

                    let tile_mag = extract_tile_window(mag_grid, region);
                    let tile_wrapped = extract_tile_window(wrapped_grid, region);
                    let tile_power = power_grid.map(|p| extract_tile_window(p, region));
                    let tile_corr = corr_grid.map(|c| extract_tile_window(c, region));

                    let tile_config = RunConfig {
                        // Force region growth for tiles (needed for assembly)
                        ntilerow,
                        ntilecol,
                        ..params.clone()
                    };

                    let result = unwrap_tile(
                        UnwrapTileParams {
                            mag: &tile_mag,
                            wrapped_phase: &tile_wrapped,
                            power: tile_power.as_deref(),
                            correlation: tile_corr.as_deref(),
                            initial_flows: None,
                            cost_threshold: 0,
                            min_region_size: 1,
                            max_components: region.rows.saturating_mul(region.cols).max(1),
                        },
                        &tile_config,
                    );

                    let tile_result = match result {
                        Ok(unwrap_result) => {
                            // Integrate phase for this tile
                            let flat_flows = match row_col_to_flat_flows(
                                &unwrap_result.flows,
                                region.rows,
                                region.cols,
                            ) {
                                Ok(f) => f,
                                Err(e) => {
                                    failed.store(true, Ordering::Relaxed);
                                    let _ = results[idx].set(Err(format!(
                                        "tile ({},{}) flow conversion: {e}",
                                        entry.tilerow, entry.tilecol
                                    )));
                                    break;
                                }
                            };

                            let tile_wrapped_flat: Vec<f32> = tile_wrapped
                                .iter()
                                .flat_map(|r| r.iter().copied())
                                .collect();
                            let unw_phase_flat = integrate_phase(
                                &tile_wrapped_flat,
                                &flat_flows,
                                region.rows,
                                region.cols,
                            );
                            let unw_phase = to_grid_f32(&unw_phase_flat, region.cols);

                            Ok(TileResult {
                                region: *region,
                                mag: tile_mag,
                                unw_phase,
                                regions: unwrap_result.regions.unwrap_or_default(),
                                flows: unwrap_result.flows,
                            })
                        }
                        Err(e) => {
                            failed.store(true, Ordering::Relaxed);
                            Err(format!(
                                "tile ({},{}) unwrap failed: {e:?}",
                                entry.tilerow, entry.tilecol
                            ))
                        }
                    };

                    let _ = results[idx].set(tile_result);
                }
            });
        }
    });

    // Collect results — check for first error
    let mut tile_results: Vec<TileResult> = Vec::with_capacity(ntiles);
    for (idx, slot) in results.into_iter().enumerate() {
        let result = slot
            .into_inner()
            .unwrap_or_else(|| Err(format!("tile {} was never processed", idx)));
        match result {
            Ok(tr) => tile_results.push(tr),
            Err(e) => return Err(io::Error::other(e)),
        }
    }

    if params.verbose {
        eprintln!(
            "multi-tile: all {} tiles unwrapped, computing bulk offsets",
            ntiles
        );
    }

    // 3. Bulk offset propagation
    let mut bulk_offsets = vec![vec![0i16; ntilecol]; ntilerow];

    for tilerow in 0..ntilerow {
        for tilecol in 0..ntilecol {
            let tilenum = tilerow * ntilecol + tilecol;
            let tile = &tile_results[tilenum];

            // Right edge
            if tilecol < ntilecol - 1 {
                let next_tilenum = tilerow * ntilecol + (tilecol + 1);
                let next_tile = &tile_results[next_tilenum];

                let current_right: Vec<f32> = tile
                    .unw_phase
                    .iter()
                    .map(|row| *row.last().unwrap_or(&0.0))
                    .collect();
                let next_left: Vec<f32> = next_tile
                    .unw_phase
                    .iter()
                    .map(|row| *row.first().unwrap_or(&0.0))
                    .collect();

                // The tiles may have different row counts; use the shorter length
                let min_rows = current_right.len().min(next_left.len());
                let _ = set_right_edge(
                    &current_right[..min_rows],
                    Some(&next_left[..min_rows]),
                    &mut bulk_offsets,
                    tilerow,
                    tilecol,
                )
                .map_err(|e| {
                    io::Error::other(format!("set_right_edge ({tilerow},{tilecol}): {e:?}"))
                })?;
            }

            // Lower edge
            if tilerow < ntilerow - 1 {
                let below_tilenum = (tilerow + 1) * ntilecol + tilecol;
                let below_tile = &tile_results[below_tilenum];

                let current_bottom: Vec<f32> = tile.unw_phase.last().cloned().unwrap_or_default();
                let below_top: Vec<f32> = below_tile.unw_phase.first().cloned().unwrap_or_default();

                let min_cols = current_bottom.len().min(below_top.len());
                let _ = set_lower_edge(
                    &current_bottom[..min_cols],
                    Some(&below_top[..min_cols]),
                    &mut bulk_offsets,
                    tilerow,
                    tilecol,
                )
                .map_err(|e| {
                    io::Error::other(format!("set_lower_edge ({tilerow},{tilecol}): {e:?}"))
                })?;
            }
        }
    }

    if params.verbose {
        eprintln!("multi-tile: bulk offsets computed, assembling tiles");
    }

    // 4. Global assembly via integrate_secondary_flows
    let tiles: Vec<TileIntegrationInput> = tile_results
        .into_iter()
        .map(|t| TileIntegrationInput {
            mag: t.mag,
            unw_phase: t.unw_phase,
            regions: t.regions,
            arc_indices: vec![],
            secondary_flows: vec![],
        })
        .collect();

    let graph = SecondaryGraph::default();
    let settings = TileReadSettings {
        row_overlap: params.rowovrlp,
        col_overlap: params.colovrlp,
        ntilerow,
        ntilecol,
    };

    let integrated = integrate_secondary_flows(IntegrateSecondaryFlowsParams {
        linelen,
        nlines,
        settings,
        bulk_offsets: &bulk_offsets,
        flip_phase_sign: false,
        graph: &graph,
        tiles: &tiles,
    })
    .map_err(|e| io::Error::other(format!("assembly failed: {e:?}")))?;

    if params.verbose {
        eprintln!("multi-tile: assembly complete");
    }

    Ok(integrated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_tile_plan_computes_expected_grid() {
        let params = RunConfig {
            ntilerow: 2,
            ntilecol: 2,
            rowovrlp: 0,
            colovrlp: 0,
            ..RunConfig::default()
        };
        let plan = build_tile_plan(10, 10, &params).unwrap();
        assert_eq!(plan.len(), 4);
        assert_eq!(plan[0].tilerow, 0);
        assert_eq!(plan[0].tilecol, 0);
        assert_eq!(plan[3].tilerow, 1);
        assert_eq!(plan[3].tilecol, 1);
    }

    #[test]
    fn run_multi_tile_2x2_smoke() {
        // 4x4 scene, 2x2 tiles
        let mag = vec![vec![1.0f32; 4]; 4];
        let wrapped = vec![vec![0.0f32; 4]; 4];

        let params = RunConfig {
            ntilerow: 2,
            ntilecol: 2,
            rowovrlp: 0,
            colovrlp: 0,
            unwrapped: true,
            nthreads: 2,
            ..RunConfig::default()
        };

        let result = run_multi_tile(&mag, &wrapped, None, None, 4, 4, &params)
            .expect("multi-tile should succeed");

        assert_eq!(result.mag.len(), 4);
        assert_eq!(result.unw_phase.len(), 4);
        assert_eq!(result.mag[0].len(), 4);
        assert_eq!(result.unw_phase[0].len(), 4);
    }
}
