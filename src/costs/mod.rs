#![allow(dead_code)]

//! Cost calculation traits and helpers.

use crate::config::{CostMode, RunConfig};

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
    nshortcycle: i64,
) -> Result<Vec<Vec<Cost>>, CostBuildError> {
    let mut costs = build_stat_costs_topo(corr, nshortcycle)?;
    for row in &mut costs {
        for cost in row {
            cost.dz_max = (3 * nshortcycle).clamp(1, i64::from(i16::MAX)) as i16;
            cost.lay_cost = (i32::from(cost.sigma_sq) / 3) as i16;
        }
    }
    Ok(costs)
}

/// Build statistical cost records for smooth mode.
///
/// This is the typed Rust equivalent of C `BuildStatCostsSmooth()`.
pub fn build_stat_costs_smooth(corr: &[Vec<f32>]) -> Result<Vec<Vec<SmoothCost>>, CostBuildError> {
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
            row.push(SmoothCost {
                offset: 0,
                sigma_sq: corr_to_sigma(corr[r][c]),
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
    let widths = row_col_widths(nrow, ncol);
    let mut mst = widths.iter().map(|&w| vec![0i16; w]).collect::<Vec<_>>();
    for (arcrow, mst_row) in mst.iter_mut().enumerate() {
        for (arccol, mst_cell) in mst_row.iter_mut().enumerate() {
            let (r, c) = if arcrow < nrow - 1 {
                (arcrow, arccol.min(ncol - 1))
            } else {
                (arcrow - (nrow - 1), arccol)
            };
            *mst_cell = corr_to_sigma(ic.correlation[r][c]);
        }
    }

    let costs = match params.cost_mode {
        CostMode::Topo => {
            CostArrayData::Topo(build_stat_costs_topo(&ic.correlation, params.nshortcycle)?)
        }
        CostMode::Defo => {
            CostArrayData::Defo(build_stat_costs_defo(&ic.correlation, params.nshortcycle)?)
        }
        CostMode::Smooth => CostArrayData::Smooth(build_stat_costs_smooth(&ic.correlation)?),
        CostMode::NoStatCosts => {
            let scalar = widths.iter().map(|&w| vec![1i16; w]).collect::<Vec<_>>();
            CostArrayData::Scalar(scalar)
        }
    };

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
