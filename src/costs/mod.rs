#![allow(dead_code)]

//! Cost calculation traits and helpers.

use crate::config::{CostMode, RunConfig};
use crate::constants::LARGE_SHORT;
use crate::data::ops::l_clip;

pub mod gradients;
pub mod lookup;
pub mod types;

/// Trait implemented by each cost component once translated.
pub trait CostBuilder {
    fn name(&self) -> &str;
}

pub use gradients::{
    GradientDirection, WrappedGradientError, WrappedGradientField, compute_azimuth_gradients,
    compute_range_gradients,
};
pub use types::{
    Cost, IncrCost, MaskWeightsError, SmoothCost, mask_cost, mask_prespecified_arc_costs,
    mask_smooth_cost,
};

#[derive(Debug, Clone, PartialEq)]
pub struct IntensityCorrelation {
    pub power: Vec<Vec<f32>>,
    pub correlation: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CostArrayData {
    Topo(Vec<Vec<Cost>>),
    Defo(Vec<Vec<Cost>>),
    Smooth(Vec<Vec<SmoothCost>>),
    Scalar(Vec<Vec<i16>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildCostArraysResult {
    pub costs: CostArrayData,
    pub mst_costs: Vec<Vec<i16>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostBuildError {
    InvalidDimensions,
    InvalidInputShape,
    WrappedGradient,
}

fn validate_grid_shape(arr: &[Vec<f32>], nrow: usize, ncol: usize) -> Result<(), CostBuildError> {
    if arr.len() != nrow {
        return Err(CostBuildError::InvalidInputShape);
    }
    if arr.iter().any(|row| row.len() != ncol) {
        return Err(CostBuildError::InvalidInputShape);
    }
    Ok(())
}

fn row_col_widths(nrow: usize, ncol: usize) -> Vec<usize> {
    let mut w = Vec::with_capacity(2 * nrow - 1);
    for r in 0..(2 * nrow - 1) {
        w.push(if r < nrow - 1 { ncol } else { ncol - 1 });
    }
    w
}

const NOCOSTSHELF: i16 = -LARGE_SHORT;
const MINSCALARCOST: i16 = 1;

fn clamp_short(v: f64) -> i16 {
    if !v.is_finite() {
        0
    } else if v <= -(LARGE_SHORT as f64) {
        -LARGE_SHORT
    } else if v >= LARGE_SHORT as f64 {
        LARGE_SHORT
    } else {
        v.trunc() as i16
    }
}

fn c_style_scale_increment(delta: i64, nflow: i64) -> i64 {
    let denom = (nflow * nflow) as f64;
    let scaled = delta as f64 / denom;
    if delta > 0 {
        scaled.ceil() as i64
    } else {
        scaled.floor() as i64
    }
}

fn apply_corner_arc_guards(mst: &mut [Vec<i16>], nrow: usize, ncol: usize) {
    if nrow < 2 || ncol < 2 {
        return;
    }
    mst[nrow - 1][0] = LARGE_SHORT;
    mst[nrow - 1][ncol - 2] = LARGE_SHORT;
    mst[2 * nrow - 2][0] = LARGE_SHORT;
    mst[2 * nrow - 2][ncol - 2] = LARGE_SHORT;
}

fn default_arc_weights_from_magnitude(mag: &[Vec<f32>]) -> Result<Vec<Vec<i16>>, CostBuildError> {
    let nrow = mag.len();
    let ncol = mag.first().map_or(0, Vec::len);
    if nrow < 2 || ncol < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    validate_grid_shape(mag, nrow, ncol)?;
    let widths = row_col_widths(nrow, ncol);
    let mut weights = widths
        .into_iter()
        .map(|w| vec![1i16; w])
        .collect::<Vec<Vec<i16>>>();

    for row in 0..nrow {
        for col in 0..ncol {
            if mag[row][col] == 0.0 {
                if row > 0 {
                    weights[row - 1][col] = 0;
                }
                if row + 1 < nrow {
                    weights[row][col] = 0;
                }
                if col > 0 {
                    weights[nrow - 1 + row][col - 1] = 0;
                }
                if col + 1 < ncol {
                    weights[nrow - 1 + row][col] = 0;
                }
            }
        }
    }

    Ok(weights)
}

/// Compute intensity/power and correlation fields used by cost builders.
///
/// This is the idiomatic Rust equivalent of C `GetIntensityAndCorrelation()`.
pub fn get_intensity_and_correlation(
    mag: &[Vec<f32>],
    wrapped_phase: &[Vec<f32>],
    power: Option<&[Vec<f32>]>,
    correlation: Option<&[Vec<f32>]>,
    default_corr: f32,
) -> Result<IntensityCorrelation, CostBuildError> {
    if mag.is_empty() || mag[0].is_empty() {
        return Err(CostBuildError::InvalidDimensions);
    }
    let nrow = mag.len();
    let ncol = mag[0].len();
    validate_grid_shape(mag, nrow, ncol)?;
    validate_grid_shape(wrapped_phase, nrow, ncol)?;

    let out_power = if let Some(p) = power {
        if p.len() != nrow || p.iter().any(|row| row.len() != ncol) {
            return Err(CostBuildError::InvalidInputShape);
        }
        p.to_vec()
    } else {
        mag.iter()
            .map(|row| row.iter().map(|&v| v * v).collect::<Vec<f32>>())
            .collect::<Vec<Vec<f32>>>()
    };

    let out_corr = if let Some(c) = correlation {
        if c.len() != nrow || c.iter().any(|row| row.len() != ncol) {
            return Err(CostBuildError::InvalidInputShape);
        }
        c.to_vec()
    } else {
        vec![vec![default_corr; ncol]; nrow]
    };

    Ok(IntensityCorrelation {
        power: out_power,
        correlation: out_corr,
    })
}

fn corr_to_sigma(c: f32) -> i16 {
    let c = c.clamp(1.0e-6, 1.0);
    ((1.0 / c) * 100.0).round().clamp(1.0, 32_000.0) as i16
}

fn calc_cost_topo_increment(
    cost: &Cost,
    arcrow: usize,
    nrow: usize,
    params: &RunConfig,
) -> (i64, i64) {
    if cost.sigma_sq == LARGE_SHORT {
        return (0, 0);
    }
    let nshortcycle = params.nshortcycle;
    let layfalloffconst = params.layfalloffconst.max(1);
    let flow = 0i64;
    let nflow = 1i64;

    let mut dzmax = i64::from(cost.dz_max);
    let offset = i64::from(cost.offset);
    let sigsq = i64::from(cost.sigma_sq).max(1);
    let laycost = i64::from(cost.lay_cost);

    let (idz1, idz2pos, idz2neg) = if arcrow < nrow - 1 {
        (
            (flow * nshortcycle + offset).abs(),
            ((flow + nflow) * nshortcycle + offset).abs(),
            ((flow - nflow) * nshortcycle + offset).abs(),
        )
    } else if dzmax < 0 {
        dzmax = -dzmax;
        (
            -(flow * nshortcycle + offset),
            -((flow + nflow) * nshortcycle + offset),
            -((flow - nflow) * nshortcycle + offset),
        )
    } else {
        (
            flow * nshortcycle + offset,
            (flow + nflow) * nshortcycle + offset,
            (flow - nflow) * nshortcycle + offset,
        )
    };

    let mut cost1 = if idz1 > dzmax {
        let d = idz1 - dzmax;
        (d * d) / (layfalloffconst * sigsq) + laycost
    } else {
        (idz1 * idz1) / sigsq
    };
    if laycost != i64::from(NOCOSTSHELF) && idz1 > 0 && cost1 > laycost {
        cost1 = laycost;
    }

    let mut poscost = if idz2pos > dzmax {
        let d = idz2pos - dzmax;
        (d * d) / (layfalloffconst * sigsq) + laycost - cost1
    } else {
        (idz2pos * idz2pos) / sigsq
    };
    if laycost != i64::from(NOCOSTSHELF) && idz2pos > 0 && (idz2pos * idz2pos) / sigsq > laycost {
        poscost = laycost - cost1;
    } else if idz2pos <= dzmax {
        poscost -= cost1;
    }

    let mut negcost = if idz2neg > dzmax {
        let d = idz2neg - dzmax;
        (d * d) / (layfalloffconst * sigsq) + laycost - cost1
    } else {
        (idz2neg * idz2neg) / sigsq
    };
    if laycost != i64::from(NOCOSTSHELF) && idz2neg > 0 && (idz2neg * idz2neg) / sigsq > laycost {
        negcost = laycost - cost1;
    } else if idz2neg <= dzmax {
        negcost -= cost1;
    }

    (
        c_style_scale_increment(poscost, nflow),
        c_style_scale_increment(negcost, nflow),
    )
}

fn calc_cost_defo_increment(cost: &Cost, params: &RunConfig) -> (i64, i64) {
    if cost.sigma_sq == LARGE_SHORT {
        return (0, 0);
    }
    let nshortcycle = params.nshortcycle;
    let layfalloffconst = params.layfalloffconst.max(1);
    let flow = 0i64;
    let nflow = 1i64;
    let offset = i64::from(cost.offset);
    let sigsq = i64::from(cost.sigma_sq).max(1);
    let dzmax = i64::from(cost.dz_max).abs();
    let laycost = i64::from(cost.lay_cost);

    let idz1 = (flow * nshortcycle + offset).abs();
    let idz2pos = ((flow + nflow) * nshortcycle + offset).abs();
    let idz2neg = ((flow - nflow) * nshortcycle + offset).abs();

    let mut cost1 = if idz1 > dzmax {
        let d = idz1 - dzmax;
        (d * d) / (layfalloffconst * sigsq) + laycost
    } else {
        (idz1 * idz1) / sigsq
    };
    if laycost != i64::from(NOCOSTSHELF) && cost1 > laycost {
        cost1 = laycost;
    }

    let mut poscost = if idz2pos > dzmax {
        let d = idz2pos - dzmax;
        (d * d) / (layfalloffconst * sigsq) + laycost - cost1
    } else {
        (idz2pos * idz2pos) / sigsq
    };
    if laycost != i64::from(NOCOSTSHELF) && (idz2pos * idz2pos) / sigsq > laycost {
        poscost = laycost - cost1;
    } else if idz2pos <= dzmax {
        poscost -= cost1;
    }

    let mut negcost = if idz2neg > dzmax {
        let d = idz2neg - dzmax;
        (d * d) / (layfalloffconst * sigsq) + laycost - cost1
    } else {
        (idz2neg * idz2neg) / sigsq
    };
    if laycost != i64::from(NOCOSTSHELF) && (idz2neg * idz2neg) / sigsq > laycost {
        negcost = laycost - cost1;
    } else if idz2neg <= dzmax {
        negcost -= cost1;
    }

    (
        c_style_scale_increment(poscost, nflow),
        c_style_scale_increment(negcost, nflow),
    )
}

fn calc_cost_smooth_increment(cost: &SmoothCost, params: &RunConfig) -> (i64, i64) {
    if cost.sigma_sq == LARGE_SHORT {
        return (0, 0);
    }
    let nshortcycle = params.nshortcycle;
    let flow = 0i64;
    let nflow = 1i64;
    let offset = i64::from(cost.offset);
    let sigsq = i64::from(cost.sigma_sq).max(1);

    let idz1 = (flow * nshortcycle + offset).abs();
    let idz2pos = ((flow + nflow) * nshortcycle + offset).abs();
    let idz2neg = ((flow - nflow) * nshortcycle + offset).abs();

    let cost1 = (idz1 * idz1) / sigsq;
    let poscost = (idz2pos * idz2pos) / sigsq - cost1;
    let negcost = (idz2neg * idz2neg) / sigsq - cost1;

    (
        c_style_scale_increment(poscost, nflow),
        c_style_scale_increment(negcost, nflow),
    )
}

fn build_mst_from_stat_costs_topo_defo(
    costs: &[Vec<Cost>],
    nrow: usize,
    ncol: usize,
    params: &RunConfig,
) -> Vec<Vec<i16>> {
    let maxcost = params
        .maxcost
        .clamp(i64::from(MINSCALARCOST), i64::from(LARGE_SHORT)) as i32;
    let mut mst = costs
        .iter()
        .enumerate()
        .map(|(arcrow, row)| {
            row.iter()
                .map(|cost| {
                    let (pos, neg) = match params.cost_mode {
                        CostMode::Topo => calc_cost_topo_increment(cost, arcrow, nrow, params),
                        _ => calc_cost_defo_increment(cost, params),
                    };
                    let mincost =
                        pos.min(neg).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                    l_clip(mincost, i32::from(MINSCALARCOST), maxcost) as i16
                })
                .collect::<Vec<i16>>()
        })
        .collect::<Vec<Vec<i16>>>();
    apply_corner_arc_guards(&mut mst, nrow, ncol);
    mst
}

fn build_mst_from_smooth_costs(
    costs: &[Vec<SmoothCost>],
    nrow: usize,
    ncol: usize,
    params: &RunConfig,
) -> Vec<Vec<i16>> {
    let maxcost = params
        .maxcost
        .clamp(i64::from(MINSCALARCOST), i64::from(LARGE_SHORT)) as i32;
    let mut mst = costs
        .iter()
        .map(|row| {
            row.iter()
                .map(|cost| {
                    let (pos, neg) = calc_cost_smooth_increment(cost, params);
                    let mincost =
                        pos.min(neg).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                    l_clip(mincost, i32::from(MINSCALARCOST), maxcost) as i16
                })
                .collect::<Vec<i16>>()
        })
        .collect::<Vec<Vec<i16>>>();
    apply_corner_arc_guards(&mut mst, nrow, ncol);
    mst
}

/// Build statistical cost records for topography mode.
///
/// This is the typed Rust equivalent of C `BuildStatCostsTopo()`.
pub fn build_stat_costs_topo(
    corr: &[Vec<f32>],
    nshortcycle: i64,
) -> Result<Vec<Vec<Cost>>, CostBuildError> {
    if corr.len() < 2 || corr[0].len() < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    let nrow = corr.len();
    let ncol = corr[0].len();
    if corr.iter().any(|row| row.len() != ncol) {
        return Err(CostBuildError::InvalidInputShape);
    }

    let mut costs = Vec::with_capacity(2 * nrow - 1);
    for arcrow in 0..(2 * nrow - 1) {
        let maxcol = if arcrow < nrow - 1 { ncol } else { ncol - 1 };
        let mut row = Vec::with_capacity(maxcol);
        for arccol in 0..maxcol {
            let (r, c) = if arcrow < nrow - 1 {
                (arcrow, arccol.min(ncol - 1))
            } else {
                (arcrow - (nrow - 1), arccol)
            };
            let sigma = corr_to_sigma(corr[r][c]);
            row.push(Cost {
                offset: 0,
                sigma_sq: sigma,
                dz_max: (2 * nshortcycle).clamp(1, i64::from(i16::MAX)) as i16,
                lay_cost: (i32::from(sigma) / 2) as i16,
            });
        }
        costs.push(row);
    }
    Ok(costs)
}

/// Build statistical cost records for deformation mode.
///
/// This is the typed Rust equivalent of C `BuildStatCostsDefo()`.
pub fn build_stat_costs_defo(
    corr: &[Vec<f32>],
    wrapped_phase: &[Vec<f32>],
    arc_weights: &[Vec<i16>],
    params: &RunConfig,
) -> Result<Vec<Vec<Cost>>, CostBuildError> {
    if corr.len() < 2 || corr[0].len() < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    let nrow = corr.len();
    let ncol = corr[0].len();
    if corr.iter().any(|row| row.len() != ncol)
        || wrapped_phase.len() != nrow
        || wrapped_phase.iter().any(|row| row.len() != ncol)
    {
        return Err(CostBuildError::InvalidInputShape);
    }

    let widths = row_col_widths(nrow, ncol);
    if arc_weights.len() != widths.len()
        || widths
            .iter()
            .enumerate()
            .any(|(arcrow, &w)| arc_weights[arcrow].len() != w)
    {
        return Err(CostBuildError::InvalidInputShape);
    }

    let wrapped_flat = wrapped_phase
        .iter()
        .flat_map(|row| row.iter().copied())
        .collect::<Vec<f32>>();
    let range_grads =
        compute_range_gradients(&wrapped_flat, nrow, ncol, params.kperpdpsi, params.kpardpsi)
            .map_err(|_| CostBuildError::WrappedGradient)?;
    let az_grads =
        compute_azimuth_gradients(&wrapped_flat, nrow, ncol, params.kpardpsi, params.kperpdpsi)
            .map_err(|_| CostBuildError::WrappedGradient)?;

    let rho0 = params.rhosconst1 / params.ncorrlooks + params.rhosconst2;
    let defocorrthresh = params.defothreshfactor * rho0;
    let rhopow = 2.0 * params.cstd1
        + params.cstd2 * params.ncorrlooks.ln()
        + params.cstd3 * params.ncorrlooks;
    let sigsqrhoconst = 2.0 / 12.0;
    let nshortcycle = params.nshortcycle as f64;
    let nshortcyclesq = nshortcycle * nshortcycle;
    let glay = -params.costscale * params.defolayconst.ln();
    let defomax = (params.defomax * nshortcycle).ceil();

    let mut masked = 0usize;
    let mut low_corr = 0usize;
    let mut costs = Vec::with_capacity(2 * nrow - 1);
    for arcrow in 0..(2 * nrow - 1) {
        let maxcol = if arcrow < nrow - 1 { ncol } else { ncol - 1 };
        let mut row = Vec::with_capacity(maxcol);
        for arccol in 0..maxcol {
            let weight = arc_weights[arcrow][arccol];
            if weight == 0 {
                let mut masked_cost = Cost::default();
                mask_cost(&mut masked_cost);
                row.push(masked_cost);
                masked += 1;
                continue;
            }

            let (rho_raw, dpsi, avgdpsi) = if arcrow < nrow - 1 {
                let idx = arcrow * ncol + arccol;
                (
                    (corr[arcrow][arccol] + corr[arcrow + 1][arccol]) as f64 / 2.0,
                    az_grads.gradients()[idx] as f64,
                    az_grads.averages()[idx] as f64,
                )
            } else {
                let r = arcrow - (nrow - 1);
                let idx = r * (ncol - 1) + arccol;
                (
                    (corr[r][arccol] + corr[r][arccol + 1]) as f64 / 2.0,
                    range_grads.gradients()[idx] as f64,
                    range_grads.averages()[idx] as f64,
                )
            };

            let low = rho_raw < defocorrthresh;
            if low {
                low_corr += 1;
            }
            let rho = if low { 0.0 } else { rho_raw };
            let sigsqrho =
                (sigsqrhoconst * (1.0 - rho).powf(rhopow) + params.sigsqcorr) * nshortcyclesq;
            let offset = if rho > 0.0 {
                nshortcycle * (dpsi - avgdpsi)
            } else {
                nshortcycle * (dpsi - 0.5 * avgdpsi)
            };
            let mut sigsq = sigsqrho / (params.costscale * f64::from(weight));
            if sigsq < params.sigsqshortmin as f64 {
                sigsq = params.sigsqshortmin as f64;
            }

            let (dzmax, laycost) = if low {
                let mut laycost = f64::from(weight) * glay;
                let mut dzmax = defomax;
                if dzmax < (laycost * sigsq).sqrt().floor() {
                    laycost = f64::from(NOCOSTSHELF);
                    dzmax = f64::from(LARGE_SHORT);
                }
                (dzmax, laycost)
            } else {
                (f64::from(LARGE_SHORT), f64::from(NOCOSTSHELF))
            };

            row.push(Cost {
                offset: clamp_short(offset),
                sigma_sq: clamp_short(sigsq),
                dz_max: clamp_short(dzmax),
                lay_cost: clamp_short(laycost),
            });
        }
        costs.push(row);
    }
    if log::log_enabled!(log::Level::Debug) {
        let total = (2 * nrow - 1) * ncol - nrow;
        log::debug!(
            "build_stat_costs_defo: arcs={}, masked={}, low_corr={}, defocorrthresh={:.6}, kperpdpsi={}, kpardpsi={}",
            total,
            masked,
            low_corr,
            defocorrthresh,
            params.kperpdpsi,
            params.kpardpsi
        );
    }
    Ok(costs)
}

/// Build statistical cost records for smooth mode.
pub fn build_stat_costs_smooth(
    corr: &[Vec<f32>],
    wrapped_phase: &[Vec<f32>],
    arc_weights: &[Vec<i16>],
    params: &RunConfig,
) -> Result<Vec<Vec<SmoothCost>>, CostBuildError> {
    if corr.len() < 2 || corr[0].len() < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    let nrow = corr.len();
    let ncol = corr[0].len();
    if corr.iter().any(|row| row.len() != ncol)
        || wrapped_phase.len() != nrow
        || wrapped_phase.iter().any(|row| row.len() != ncol)
    {
        return Err(CostBuildError::InvalidInputShape);
    }
    let widths = row_col_widths(nrow, ncol);
    if arc_weights.len() != widths.len()
        || widths
            .iter()
            .enumerate()
            .any(|(arcrow, &w)| arc_weights[arcrow].len() != w)
    {
        return Err(CostBuildError::InvalidInputShape);
    }

    let wrapped_flat = wrapped_phase
        .iter()
        .flat_map(|row| row.iter().copied())
        .collect::<Vec<f32>>();
    let range_grads =
        compute_range_gradients(&wrapped_flat, nrow, ncol, params.kperpdpsi, params.kpardpsi)
            .map_err(|_| CostBuildError::WrappedGradient)?;
    let az_grads =
        compute_azimuth_gradients(&wrapped_flat, nrow, ncol, params.kpardpsi, params.kperpdpsi)
            .map_err(|_| CostBuildError::WrappedGradient)?;

    let rho0 = params.rhosconst1 / params.ncorrlooks + params.rhosconst2;
    let defocorrthresh = params.defothreshfactor * rho0;
    let rhopow = 2.0 * params.cstd1
        + params.cstd2 * params.ncorrlooks.ln()
        + params.cstd3 * params.ncorrlooks;
    let sigsqrhoconst = 2.0 / 12.0;
    let nshortcycle = params.nshortcycle as f64;
    let nshortcyclesq = nshortcycle * nshortcycle;

    let mut costs = Vec::with_capacity(2 * nrow - 1);
    for arcrow in 0..(2 * nrow - 1) {
        let maxcol = if arcrow < nrow - 1 { ncol } else { ncol - 1 };
        let mut row = Vec::with_capacity(maxcol);
        for arccol in 0..maxcol {
            let weight = arc_weights[arcrow][arccol];
            if weight == 0 {
                let mut masked_cost = SmoothCost::default();
                mask_smooth_cost(&mut masked_cost);
                row.push(masked_cost);
                continue;
            }
            let (rho_raw, dpsi, avgdpsi) = if arcrow < nrow - 1 {
                let idx = arcrow * ncol + arccol;
                (
                    (corr[arcrow][arccol] + corr[arcrow + 1][arccol]) as f64 / 2.0,
                    az_grads.gradients()[idx] as f64,
                    az_grads.averages()[idx] as f64,
                )
            } else {
                let r = arcrow - (nrow - 1);
                let idx = r * (ncol - 1) + arccol;
                (
                    (corr[r][arccol] + corr[r][arccol + 1]) as f64 / 2.0,
                    range_grads.gradients()[idx] as f64,
                    range_grads.averages()[idx] as f64,
                )
            };
            let rho = if rho_raw < defocorrthresh {
                0.0
            } else {
                rho_raw
            };
            let sigsqrho =
                (sigsqrhoconst * (1.0 - rho).powf(rhopow) + params.sigsqcorr) * nshortcyclesq;
            let offset = if rho > 0.0 {
                nshortcycle * (dpsi - avgdpsi)
            } else {
                nshortcycle * (dpsi - 0.5 * avgdpsi)
            };
            let mut sigsq = sigsqrho / (params.costscale * f64::from(weight));
            if sigsq < params.sigsqshortmin as f64 {
                sigsq = params.sigsqshortmin as f64;
            }
            row.push(SmoothCost {
                offset: clamp_short(offset),
                sigma_sq: clamp_short(sigsq),
            });
        }
        costs.push(row);
    }
    Ok(costs)
}

/// Build arc-cost arrays (and MST costs) for the current cost mode.
///
/// This is the typed Rust equivalent of C `BuildCostArrays()`.
pub fn build_cost_arrays(
    mag: &[Vec<f32>],
    wrapped_phase: &[Vec<f32>],
    power: Option<&[Vec<f32>]>,
    correlation: Option<&[Vec<f32>]>,
    params: &RunConfig,
) -> Result<BuildCostArraysResult, CostBuildError> {
    if mag.len() < 2 || mag[0].len() < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    let nrow = mag.len();
    let ncol = mag[0].len();

    let ic = get_intensity_and_correlation(mag, wrapped_phase, power, correlation, 0.01)?;
    let arc_weights = default_arc_weights_from_magnitude(mag)?;

    let costs = match params.cost_mode {
        CostMode::Topo => {
            CostArrayData::Topo(build_stat_costs_topo(&ic.correlation, params.nshortcycle)?)
        }
        CostMode::Defo => CostArrayData::Defo(build_stat_costs_defo(
            &ic.correlation,
            wrapped_phase,
            &arc_weights,
            params,
        )?),
        CostMode::Smooth => CostArrayData::Smooth(build_stat_costs_smooth(
            &ic.correlation,
            wrapped_phase,
            &arc_weights,
            params,
        )?),
        CostMode::NoStatCosts => {
            let scalar = arc_weights.clone();
            CostArrayData::Scalar(scalar)
        }
    };

    let mst = match &costs {
        CostArrayData::Topo(c) | CostArrayData::Defo(c) => {
            build_mst_from_stat_costs_topo_defo(c, nrow, ncol, params)
        }
        CostArrayData::Smooth(c) => build_mst_from_smooth_costs(c, nrow, ncol, params),
        CostArrayData::Scalar(c) => c.clone(),
    };

    if log::log_enabled!(log::Level::Debug) {
        let mut min = i16::MAX;
        let mut max = i16::MIN;
        let mut sum = 0i64;
        let mut count = 0usize;
        let mut clipped = 0usize;
        for row in &mst {
            for &v in row {
                min = min.min(v);
                max = max.max(v);
                sum += i64::from(v);
                if v == LARGE_SHORT {
                    clipped += 1;
                }
                count += 1;
            }
        }
        let mean = if count == 0 {
            0.0
        } else {
            sum as f64 / count as f64
        };
        log::debug!(
            "build_cost_arrays: mode={:?}, mst(count={}, min={}, max={}, mean={:.3}, clipped_large_short={})",
            params.cost_mode,
            count,
            min,
            max,
            mean,
            clipped
        );
    }

    Ok(BuildCostArraysResult {
        costs,
        mst_costs: mst,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_intensity_and_correlation_uses_defaults() {
        let mag = vec![vec![2.0, 3.0], vec![4.0, 5.0]];
        let wrapped = vec![vec![0.0, 1.0], vec![2.0, 3.0]];
        let ic = get_intensity_and_correlation(&mag, &wrapped, None, None, 0.25).unwrap();
        assert_eq!(ic.power[0][0], 4.0);
        assert_eq!(ic.correlation[1][1], 0.25);
    }

    #[test]
    fn build_stat_costs_topo_has_row_col_layout() {
        let corr = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let costs = build_stat_costs_topo(&corr, 200).unwrap();
        assert_eq!(costs.len(), 3);
        assert_eq!(costs[0].len(), 2);
        assert_eq!(costs[2].len(), 1);
    }

    #[test]
    fn build_cost_arrays_dispatches_modes() {
        let mag = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let wrapped = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let params = RunConfig::default();
        let out = build_cost_arrays(&mag, &wrapped, None, None, &params).unwrap();
        assert!(matches!(out.costs, CostArrayData::Topo(_)));
        assert_eq!(out.mst_costs.len(), 3);
    }
}
