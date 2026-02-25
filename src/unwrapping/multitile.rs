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
    FindNumPathsOutInputs, IntegrateSecondaryFlowsParams, IntegratedSecondaryOutput,
    SecondaryGraph, TileIntegrationInput, TileReadSettings, TraceRegionsParams,
    integrate_secondary_flows, normalize_secondary_arc_costs, set_lower_edge, set_right_edge,
    solve_secondary_network, trace_regions,
};
use std::io;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

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
    pub unw_phase: Vec<Vec<f32>>,
    pub regions: Vec<Vec<i16>>,
    pub mst_costs: Vec<Vec<i16>>,
    pub arc_indices: Vec<usize>,
    pub secondary_flows: Vec<i16>,
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

/// Fill a rectangular tile window into a reusable destination grid buffer.
fn fill_tile_window(
    scene: &[Vec<f32>],
    region: &TileRegion,
    out: &mut Vec<Vec<f32>>,
) -> io::Result<()> {
    let row_end = region.first_row.saturating_add(region.rows);
    if row_end > scene.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "tile rows [{}..{}) exceed scene height {}",
                region.first_row,
                row_end,
                scene.len()
            ),
        ));
    }

    out.resize_with(region.rows, Vec::new);
    for r in 0..region.rows {
        let src_row = region.first_row + r;
        let row = &scene[src_row];
        let start = region.first_col;
        let end = start.saturating_add(region.cols);
        if end > row.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "tile cols [{}..{}) exceed scene row width {} at row {}",
                    start,
                    end,
                    row.len(),
                    src_row
                ),
            ));
        }
        out[r].resize(region.cols, 0.0);
        out[r].copy_from_slice(&row[start..end]);
    }
    Ok(())
}

/// Extract a rectangular window from a scene-level `Vec<Vec<f32>>` grid.
fn extract_tile_window(scene: &[Vec<f32>], region: &TileRegion) -> io::Result<Vec<Vec<f32>>> {
    let mut out = Vec::new();
    fill_tile_window(scene, region, &mut out)?;
    Ok(out)
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

fn empty_mst_costs(nrow: usize, ncol: usize) -> Vec<Vec<i16>> {
    if nrow == 0 || ncol == 0 {
        return Vec::new();
    }
    (0..(2 * nrow - 1))
        .map(|row| {
            let width = if row < nrow - 1 { ncol } else { ncol - 1 };
            vec![0i16; width]
        })
        .collect()
}

fn fit_region_grid(src: &[Vec<i16>], rows: usize, cols: usize) -> Vec<Vec<i16>> {
    let mut out = vec![vec![0i16; cols]; rows];
    let rmax = rows.min(src.len());
    for r in 0..rmax {
        let cmax = cols.min(src[r].len());
        out[r][..cmax].copy_from_slice(&src[r][..cmax]);
    }
    out
}

fn fit_edge_row(src: Option<&[i16]>, cols: usize) -> Vec<i16> {
    let mut out = vec![0i16; cols];
    if let Some(src_row) = src {
        let cmax = cols.min(src_row.len());
        out[..cmax].copy_from_slice(&src_row[..cmax]);
    }
    out
}

/// Run the multi-tile unwrapping pipeline end-to-end.
///
/// Returns the assembled `(mag, unw_phase)` grids at full scene dimensions.
pub fn run_multi_tile(
    mag_grid: &[Vec<f32>],
    wrapped_grid: &[Vec<f32>],
    power_grid: Option<&[Vec<f32>]>,
    corr_grid: Option<&[Vec<f32>]>,
    tile_mask: Option<&[i8]>,
    nlines: usize,
    linelen: usize,
    params: &RunConfig,
) -> io::Result<IntegratedSecondaryOutput> {
    let ntilerow = params.ntilerow;
    let ntilecol = params.ntilecol;
    let ntiles = ntilerow * ntilecol;

    // 1. Build tile plan
    let t_plan = Instant::now();
    let plan = build_tile_plan(nlines, linelen, params)?;
    assert_eq!(plan.len(), ntiles);

    if params.verbose {
        log::info!(
            "multi-tile: {}x{} grid ({} tiles), {} threads",
            ntilerow,
            ntilecol,
            ntiles,
            params.nthreads
        );
        log::info!("multi-tile: plan build took {:?}", t_plan.elapsed());
    }

    // 2. Parallel tile unwrap
    let t_unwrap = Instant::now();
    let worker_count = params.nthreads.min(ntiles).max(1);
    let results: Vec<OnceLock<Result<TileResult, String>>> =
        (0..ntiles).map(|_| OnceLock::new()).collect();
    let next_job = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let tile_config = RunConfig {
        // Force tiled behavior so region data is available for assembly.
        ntilerow,
        ntilecol,
        ..params.clone()
    };
    let do_tile: Vec<bool> = match tile_mask {
        Some(mask) => {
            if mask.len() != ntiles {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "tile mask length {} does not match tile count {}",
                        mask.len(),
                        ntiles
                    ),
                ));
            }
            mask.iter().map(|&v| v != 0).collect()
        }
        None => vec![true; ntiles],
    };
    let active_tiles = do_tile.iter().filter(|&&v| v).count();

    std::thread::scope(|s| {
        for _worker in 0..worker_count {
            let plan = &plan;
            let results = &results;
            let next_job = &next_job;
            let failed = &failed;
            let tile_config = &tile_config;
            let do_tile = &do_tile;

            s.spawn(move || {
                let mut tile_mag: Vec<Vec<f32>> = Vec::new();
                let mut tile_wrapped: Vec<Vec<f32>> = Vec::new();
                let mut tile_power: Option<Vec<Vec<f32>>> = power_grid.map(|_| Vec::new());
                let mut tile_corr: Option<Vec<Vec<f32>>> = corr_grid.map(|_| Vec::new());

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

                    if !do_tile[idx] {
                        let skip_result =
                            extract_tile_window(wrapped_grid, region).map(|unw_phase| TileResult {
                                region: *region,
                                unw_phase,
                                regions: vec![vec![0i16; region.cols]; region.rows],
                                mst_costs: empty_mst_costs(region.rows, region.cols),
                                arc_indices: Vec::new(),
                                secondary_flows: Vec::new(),
                            });
                        let _ = results[idx].set(skip_result.map_err(|e| {
                            format!(
                                "tile ({},{}) masked-pass-through extraction failed: {e}",
                                entry.tilerow, entry.tilecol
                            )
                        }));
                        continue;
                    }

                    if let Err(e) = fill_tile_window(mag_grid, region, &mut tile_mag) {
                        failed.store(true, Ordering::Relaxed);
                        let _ = results[idx].set(Err(format!(
                            "tile ({},{}) magnitude extraction failed: {e}",
                            entry.tilerow, entry.tilecol
                        )));
                        break;
                    }
                    if let Err(e) = fill_tile_window(wrapped_grid, region, &mut tile_wrapped) {
                        failed.store(true, Ordering::Relaxed);
                        let _ = results[idx].set(Err(format!(
                            "tile ({},{}) phase extraction failed: {e}",
                            entry.tilerow, entry.tilecol
                        )));
                        break;
                    }
                    if let (Some(scene), Some(buf)) = (power_grid, tile_power.as_mut())
                        && let Err(e) = fill_tile_window(scene, region, buf)
                    {
                        failed.store(true, Ordering::Relaxed);
                        let _ = results[idx].set(Err(format!(
                            "tile ({},{}) power extraction failed: {e}",
                            entry.tilerow, entry.tilecol
                        )));
                        break;
                    }
                    if let (Some(scene), Some(buf)) = (corr_grid, tile_corr.as_mut())
                        && let Err(e) = fill_tile_window(scene, region, buf)
                    {
                        failed.store(true, Ordering::Relaxed);
                        let _ = results[idx].set(Err(format!(
                            "tile ({},{}) correlation extraction failed: {e}",
                            entry.tilerow, entry.tilecol
                        )));
                        break;
                    }

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
                        tile_config,
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
                                unw_phase,
                                regions: unwrap_result
                                    .regions
                                    .unwrap_or_else(|| vec![vec![0i16; region.cols]; region.rows]),
                                mst_costs: unwrap_result.cost_arrays.mst_costs,
                                arc_indices: Vec::new(),
                                secondary_flows: Vec::new(),
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
        log::info!(
            "multi-tile: all {} tiles processed ({} unwrapped, {} masked) in {:?}, computing bulk offsets",
            ntiles,
            active_tiles,
            ntiles.saturating_sub(active_tiles),
            t_unwrap.elapsed()
        );
    }

    // 3. Bulk offset propagation
    let t_offsets = Instant::now();
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

                if current_right.len() != next_left.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "right-edge seam length mismatch at ({tilerow},{tilecol}): {} vs {}",
                            current_right.len(),
                            next_left.len()
                        ),
                    ));
                }
                let _ = set_right_edge(
                    &current_right,
                    Some(&next_left),
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

                if current_bottom.len() != below_top.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "lower-edge seam length mismatch at ({tilerow},{tilecol}): {} vs {}",
                            current_bottom.len(),
                            below_top.len()
                        ),
                    ));
                }
                let _ = set_lower_edge(
                    &current_bottom,
                    Some(&below_top),
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

    // 4. Trace secondary arcs and solve secondary network
    let t_secondary = Instant::now();
    let mut graph = SecondaryGraph::default();
    let mut traced_arc_len_sum = 0usize;
    for tilerow in 0..ntilerow {
        for tilecol in 0..ntilecol {
            let tilenum = tilerow * ntilecol + tilecol;
            if !do_tile[tilenum] {
                continue;
            }

            let (nrow, ncol) = {
                let tile = &tile_results[tilenum];
                (
                    tile.regions.len(),
                    tile.regions.first().map_or(0, |row| row.len()),
                )
            };
            if nrow == 0 || ncol == 0 {
                continue;
            }

            let this_regions = fit_region_grid(&tile_results[tilenum].regions, nrow, ncol);
            let nextregions = if tilecol + 1 < ntilecol {
                let next_idx = tilerow * ntilecol + tilecol + 1;
                fit_region_grid(&tile_results[next_idx].regions, nrow, ncol)
            } else {
                this_regions.clone()
            };
            let lastregions = if tilecol > 0 {
                let prev_idx = tilerow * ntilecol + tilecol - 1;
                fit_region_grid(&tile_results[prev_idx].regions, nrow, ncol)
            } else {
                this_regions.clone()
            };
            let regions_above = if tilerow > 0 {
                let above_idx = (tilerow - 1) * ntilecol + tilecol;
                fit_edge_row(
                    tile_results[above_idx].regions.last().map(|r| r.as_slice()),
                    ncol,
                )
            } else {
                vec![0i16; ncol]
            };
            let regions_below = if tilerow + 1 < ntilerow {
                let below_idx = (tilerow + 1) * ntilecol + tilecol;
                fit_edge_row(
                    tile_results[below_idx]
                        .regions
                        .first()
                        .map(|r| r.as_slice()),
                    ncol,
                )
            } else {
                vec![0i16; ncol]
            };
            let prevncol = if tilecol > 0 {
                lastregions.first().map_or(ncol, Vec::len).max(1)
            } else {
                ncol
            };

            let trace = trace_regions(
                &mut graph,
                TraceRegionsParams {
                    flowmax: params.scndry_arc_flow_max,
                    nshortcycle: params.nshortcycle,
                    tileedgeweight: params.tile_edge_weight,
                    mst_costs: &tile_results[tilenum].mst_costs,
                    unw_phase: &tile_results[tilenum].unw_phase,
                    inputs: FindNumPathsOutInputs {
                        ntilerow,
                        ntilecol,
                        tilerow,
                        tilecol,
                        nnrow: nrow + 1,
                        nncol: ncol + 1,
                        prevncol,
                        regions: &this_regions,
                        nextregions: &nextregions,
                        lastregions: &lastregions,
                        regionsabove: &regions_above,
                        regionsbelow: &regions_below,
                    },
                },
            )
            .map_err(|e| {
                io::Error::other(format!(
                    "trace_regions failed at tile ({tilerow},{tilecol}): {e:?}"
                ))
            })?;
            traced_arc_len_sum = traced_arc_len_sum.saturating_add(trace.total_arc_len);
            tile_results[tilenum].arc_indices = trace.arc_indices;
        }
    }

    normalize_secondary_arc_costs(&mut graph);
    let secondary_flows = solve_secondary_network(
        &graph,
        params.scndry_arc_flow_max,
        params.max_cycle_fraction,
        params.maxflow,
        params.nshortcycle,
    )
    .map_err(|e| io::Error::other(format!("secondary network solve failed: {e:?}")))?;

    for tile in &mut tile_results {
        tile.secondary_flows = tile
            .arc_indices
            .iter()
            .map(|&arc_idx| {
                secondary_flows.get(arc_idx).copied().ok_or_else(|| {
                    io::Error::other(format!("invalid secondary arc index {arc_idx}"))
                })
            })
            .collect::<io::Result<Vec<i16>>>()?;
    }

    if params.verbose {
        log::info!(
            "multi-tile: bulk offsets in {:?}, traced {} arcs (len sum {}), solved secondary in {:?}",
            t_offsets.elapsed(),
            graph.arcs.len(),
            traced_arc_len_sum,
            t_secondary.elapsed()
        );
    }

    // 5. Global assembly via integrate_secondary_flows
    let t_assembly = Instant::now();
    let mut tiles: Vec<TileIntegrationInput> = Vec::with_capacity(tile_results.len());
    for t in tile_results {
        let mag = extract_tile_window(mag_grid, &t.region).map_err(|e| {
            io::Error::other(format!(
                "assembly magnitude extraction failed for tile starting at ({},{}): {e}",
                t.region.first_row, t.region.first_col
            ))
        })?;
        tiles.push(TileIntegrationInput {
            mag,
            unw_phase: t.unw_phase,
            regions: t.regions,
            arc_indices: t.arc_indices,
            secondary_flows: t.secondary_flows,
        });
    }

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
        log::info!(
            "multi-tile: assembly complete in {:?}",
            t_assembly.elapsed()
        );
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

        let result = run_multi_tile(&mag, &wrapped, None, None, None, 4, 4, &params)
            .expect("multi-tile should succeed");

        assert_eq!(result.mag.len(), 4);
        assert_eq!(result.unw_phase.len(), 4);
        assert_eq!(result.mag[0].len(), 4);
        assert_eq!(result.unw_phase[0].len(), 4);
    }

    #[test]
    fn extract_tile_window_rejects_out_of_bounds_region() {
        let scene = vec![vec![1.0f32; 2]; 2];
        let region = TileRegion::new(1, 1, 2, 2);
        let err = extract_tile_window(&scene, &region).expect_err("expected out-of-bounds error");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn run_multi_tile_accepts_full_zero_tile_mask() {
        // 1x2 scene split into two 1x1 tiles: unwrapping would be invalid for 1x1,
        // so this verifies masked tiles bypass unwrap and still assemble.
        let mag = vec![vec![1.0f32, 1.0f32]];
        let wrapped = vec![vec![0.25f32, -0.5f32]];
        let tile_mask = vec![0i8, 0i8];

        let params = RunConfig {
            ntilerow: 1,
            ntilecol: 2,
            rowovrlp: 0,
            colovrlp: 0,
            nthreads: 2,
            ..RunConfig::default()
        };

        let result = run_multi_tile(&mag, &wrapped, None, None, Some(&tile_mask), 1, 2, &params)
            .expect("masked tiles should bypass unwrap and assemble");

        assert_eq!(result.unw_phase, wrapped);
    }

    #[test]
    fn run_multi_tile_secondary_pipeline_handles_nontrivial_scene() {
        let mag = vec![vec![1.0f32; 4]; 2];
        let wrapped = vec![
            vec![0.0f32, 0.8f32, -0.7f32, 0.6f32],
            vec![0.2f32, 1.0f32, -0.5f32, 0.4f32],
        ];

        let params = RunConfig {
            ntilerow: 1,
            ntilecol: 2,
            rowovrlp: 0,
            colovrlp: 1,
            unwrapped: true,
            nthreads: 2,
            ..RunConfig::default()
        };

        let result =
            run_multi_tile(&mag, &wrapped, None, None, None, 2, 4, &params).expect("run succeeds");
        assert_eq!(result.unw_phase.len(), 2);
        assert_eq!(result.unw_phase[0].len(), 4);
    }
}
