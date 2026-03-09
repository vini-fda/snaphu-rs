//! Cost calculation traits and helpers.

use crate::config::{CostMode, RunConfig, TransmitMode};
use crate::constants::{LARGE_SHORT, TWO_PI};
use crate::costs::lookup::{
    LookupParameters, build_dz_rho_max_lookup, build_dzrcrit_lookup, solve_ei_model_params,
};
use crate::data::ops::{despeckle, l_clip, remove_mean};
use crate::data::raster::Raster;
use crate::data::tile::TileRegion;
use crate::io::reader::RowColTile;

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

/// Contiguous intensity/correlation fields used by the statistical cost builders.
#[derive(Debug, Clone, PartialEq)]
pub struct IntensityCorrelation {
    pub power: Raster<f32>,
    pub correlation: Raster<f32>,
}

/// Inputs for statistical cost construction.
///
/// All rasters are contiguous row-major grids with `tile.cols` samples per row
/// and `tile.rows` rows. When present, optional rasters must match those exact
/// dimensions. `arc_weights`, when present, must use SNAPHU's packed row/col
/// layout: row arcs are `(tile.rows - 1) x tile.cols`, and column arcs are
/// `tile.rows x (tile.cols - 1)`.
#[derive(Debug, Clone)]
pub struct CostBuildInputs<'a> {
    pub tile: TileRegion,
    pub mag: &'a Raster<f32>,
    pub wrapped_phase: &'a Raster<f32>,
    pub power: Option<&'a Raster<f32>>,
    pub correlation: Option<&'a Raster<f32>>,
    pub unwrapped_estimate: Option<&'a Raster<f32>>,
    pub arc_weights: Option<&'a RowColTile<i16>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CostArrayData {
    Topo(Vec<Vec<Cost>>),
    Defo(Vec<Vec<Cost>>),
    Smooth(Vec<Vec<SmoothCost>>),
    Scalar(Vec<Vec<i16>>),
}

/// Extra contiguous diagnostic products emitted by topography-mode costs.
#[derive(Debug, Clone, PartialEq)]
pub struct TopoCostDiagnostics {
    pub normalized_intensity: Raster<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildCostArraysResult {
    pub costs: CostArrayData,
    pub mst_costs: Vec<Vec<i16>>,
    pub topo_diagnostics: Option<TopoCostDiagnostics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostBuildError {
    InvalidDimensions,
    InvalidInputShape,
    WrappedGradient,
    Lookup,
    IntensityNormalization,
    InvalidConfiguration,
}

#[derive(Debug, Clone, Copy)]
struct TopoColumnParams {
    ambiguity_height: f64,
    dzr0: f64,
    eicrit: f64,
    dphilaypeak: f64,
    sigsqrhoconst: f64,
    ztoshort: f64,
    ztoshortsq: f64,
    sigsqlay: f64,
    slope1: f64,
    slope2: f64,
    const1: f64,
    const2: f64,
    nomincangle: f64,
}

const NOCOSTSHELF: i16 = -LARGE_SHORT;
const MINSCALARCOST: i16 = 1;

fn validate_raster_shape<T>(
    raster: &Raster<T>,
    rows: usize,
    cols: usize,
) -> Result<(), CostBuildError> {
    if raster.height != rows || raster.width != cols || !raster.has_valid_shape() {
        return Err(CostBuildError::InvalidInputShape);
    }
    Ok(())
}

fn validate_optional_raster_shape<T>(
    raster: Option<&Raster<T>>,
    rows: usize,
    cols: usize,
) -> Result<(), CostBuildError> {
    if let Some(raster) = raster {
        validate_raster_shape(raster, rows, cols)?;
    }
    Ok(())
}

#[cfg(test)]
fn row_col_widths(nrow: usize, ncol: usize) -> Vec<usize> {
    let mut w = Vec::with_capacity(2 * nrow - 1);
    for r in 0..(2 * nrow - 1) {
        w.push(if r < nrow - 1 { ncol } else { ncol - 1 });
    }
    w
}

fn row_arc_idx(row: usize, col: usize, ncol: usize) -> usize {
    row * ncol + col
}

fn col_arc_idx(row: usize, col: usize, ncol: usize) -> usize {
    row * (ncol - 1) + col
}

fn row_major_idx(row: usize, col: usize, ncol: usize) -> usize {
    row * ncol + col
}

fn row_col_to_nested<T: Clone>(
    row_data: &[T],
    col_data: &[T],
    nrow: usize,
    ncol: usize,
) -> Vec<Vec<T>> {
    let mut out = Vec::with_capacity(2 * nrow - 1);
    for row in 0..(nrow - 1) {
        let start = row * ncol;
        out.push(row_data[start..start + ncol].to_vec());
    }
    let col_width = ncol - 1;
    for row in 0..nrow {
        let start = row * col_width;
        out.push(col_data[start..start + col_width].to_vec());
    }
    out
}

fn compute_range_gradients_fallback(
    wrapped_phase: &Raster<f32>,
    kernel_perpendicular: usize,
    kernel_parallel: usize,
) -> Result<WrappedGradientField, CostBuildError> {
    match compute_range_gradients(
        &wrapped_phase.data,
        wrapped_phase.height,
        wrapped_phase.width,
        kernel_perpendicular,
        kernel_parallel,
    ) {
        Ok(field) => Ok(field),
        Err(WrappedGradientError::KernelTooLarge { .. }) => compute_range_gradients(
            &wrapped_phase.data,
            wrapped_phase.height,
            wrapped_phase.width,
            1,
            1,
        )
        .map_err(|_| CostBuildError::WrappedGradient),
        Err(_) => Err(CostBuildError::WrappedGradient),
    }
}

fn compute_azimuth_gradients_fallback(
    wrapped_phase: &Raster<f32>,
    kernel_parallel: usize,
    kernel_perpendicular: usize,
) -> Result<WrappedGradientField, CostBuildError> {
    match compute_azimuth_gradients(
        &wrapped_phase.data,
        wrapped_phase.height,
        wrapped_phase.width,
        kernel_parallel,
        kernel_perpendicular,
    ) {
        Ok(field) => Ok(field),
        Err(WrappedGradientError::KernelTooLarge { .. }) => compute_azimuth_gradients(
            &wrapped_phase.data,
            wrapped_phase.height,
            wrapped_phase.width,
            1,
            1,
        )
        .map_err(|_| CostBuildError::WrappedGradient),
        Err(_) => Err(CostBuildError::WrappedGradient),
    }
}

#[cfg(test)]
fn nested_to_row_col<T: Clone>(
    values: &[Vec<T>],
    nrow: usize,
    ncol: usize,
) -> Result<(Vec<T>, Vec<T>), CostBuildError> {
    let widths = row_col_widths(nrow, ncol);
    if values.len() != widths.len() {
        return Err(CostBuildError::InvalidInputShape);
    }
    let mut row_data = Vec::with_capacity((nrow - 1) * ncol);
    let mut col_data = Vec::with_capacity(nrow * (ncol - 1));
    for (row, &width) in widths.iter().enumerate() {
        if values[row].len() != width {
            return Err(CostBuildError::InvalidInputShape);
        }
        if row < nrow - 1 {
            row_data.extend_from_slice(&values[row]);
        } else {
            col_data.extend_from_slice(&values[row]);
        }
    }
    Ok((row_data, col_data))
}

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

fn default_arc_weights_from_magnitude(
    mag: &Raster<f32>,
) -> Result<RowColTile<i16>, CostBuildError> {
    let nrow = mag.height;
    let ncol = mag.width;
    if nrow < 2 || ncol < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    validate_raster_shape(mag, nrow, ncol)?;
    let mut row_data = vec![1i16; (nrow - 1) * ncol];
    let mut col_data = vec![1i16; nrow * (ncol - 1)];

    for row in 0..nrow {
        for col in 0..ncol {
            if mag.data[row_major_idx(row, col, ncol)] == 0.0 {
                if row > 0 {
                    row_data[row_arc_idx(row - 1, col, ncol)] = 0;
                }
                if row + 1 < nrow {
                    row_data[row_arc_idx(row, col, ncol)] = 0;
                }
                if col > 0 {
                    col_data[col_arc_idx(row, col - 1, ncol)] = 0;
                }
                if col + 1 < ncol {
                    col_data[col_arc_idx(row, col, ncol)] = 0;
                }
            }
        }
    }

    Ok(RowColTile {
        row_arcs: Raster::new(ncol, nrow - 1, row_data),
        col_arcs: Raster::new(ncol - 1, nrow, col_data),
    })
}

fn mask_arc_weights_with_magnitude(
    arc_weights: &mut RowColTile<i16>,
    mag: &Raster<f32>,
) -> Result<(), CostBuildError> {
    validate_raster_shape(mag, mag.height, mag.width)?;
    let nrow = mag.height;
    let ncol = mag.width;
    if arc_weights.row_arcs.width != ncol
        || arc_weights.row_arcs.height != nrow - 1
        || arc_weights.col_arcs.width != ncol - 1
        || arc_weights.col_arcs.height != nrow
        || !arc_weights.row_arcs.has_valid_shape()
        || !arc_weights.col_arcs.has_valid_shape()
    {
        return Err(CostBuildError::InvalidInputShape);
    }

    for row in 0..nrow {
        for col in 0..ncol {
            if mag.data[row_major_idx(row, col, ncol)] == 0.0 {
                if row > 0 {
                    arc_weights.row_arcs.data[row_arc_idx(row - 1, col, ncol)] = 0;
                }
                if row + 1 < nrow {
                    arc_weights.row_arcs.data[row_arc_idx(row, col, ncol)] = 0;
                }
                if col > 0 {
                    arc_weights.col_arcs.data[col_arc_idx(row, col - 1, ncol)] = 0;
                }
                if col + 1 < ncol {
                    arc_weights.col_arcs.data[col_arc_idx(row, col, ncol)] = 0;
                }
            }
        }
    }

    Ok(())
}

/// Compute intensity/power and correlation fields used by cost builders.
///
/// The input rasters are contiguous row-major grids with identical dimensions.
/// When `power` is absent, power defaults to `mag^2`. When `correlation` is
/// absent, the returned correlation raster is filled with `default_corr`.
pub fn get_intensity_and_correlation(
    inputs: &CostBuildInputs<'_>,
    default_corr: f32,
) -> Result<IntensityCorrelation, CostBuildError> {
    let nrow = inputs.tile.rows;
    let ncol = inputs.tile.cols;
    if nrow == 0 || ncol == 0 {
        return Err(CostBuildError::InvalidDimensions);
    }
    validate_raster_shape(inputs.mag, nrow, ncol)?;
    validate_raster_shape(inputs.wrapped_phase, nrow, ncol)?;
    validate_optional_raster_shape(inputs.power, nrow, ncol)?;
    validate_optional_raster_shape(inputs.correlation, nrow, ncol)?;

    let power = if let Some(p) = inputs.power {
        p.clone()
    } else {
        Raster::new(
            ncol,
            nrow,
            inputs.mag.data.iter().map(|&v| v * v).collect::<Vec<f32>>(),
        )
    };

    let correlation = if let Some(c) = inputs.correlation {
        c.clone()
    } else {
        Raster::new(ncol, nrow, vec![default_corr; nrow * ncol])
    };

    Ok(IntensityCorrelation { power, correlation })
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

fn topo_lookup_params(params: &RunConfig, baseline: f64, baseline_angle: f64) -> LookupParameters {
    LookupParameters {
        orbit_radius: params.orbitradius,
        earth_radius: params.earthradius,
        near_range: params.nearrange,
        range_spacing: params.dr,
        azimuth_spacing: params.da,
        baseline,
        baseline_angle,
        kds: params.kds,
        sloperatio_factor: params.sloperatio_factor,
        specular_exponent: params.specular_exponent,
        dzrcrit_factor: params.dzrcrit_factor,
        initial_dzr: params.initdzr,
        initial_dz_step: params.initdzstep,
        incidence_angle_step: params.dnomincangle,
        range_resolution: params.range_resolution,
        wavelength: params.lambda,
        threshold: params.threshold,
    }
}

fn incidence_geometry(a: f64, re: f64, slantrange: f64) -> Result<(f64, f64, f64), CostBuildError> {
    let denom = 2.0 * slantrange * re;
    if denom == 0.0 {
        return Err(CostBuildError::InvalidConfiguration);
    }
    let cosnomincangle = ((a * a - slantrange * slantrange - re * re) / denom).clamp(-1.0, 1.0);
    let nomincangle = cosnomincangle.acos();
    let sinnomincangle = nomincangle.sin();
    if !cosnomincangle.is_finite() || !nomincangle.is_finite() || !sinnomincangle.is_finite() {
        return Err(CostBuildError::InvalidConfiguration);
    }
    Ok((cosnomincangle, nomincangle, sinnomincangle))
}

fn build_topo_column_params(
    col: usize,
    nearrange: f64,
    baseline: f64,
    baseline_angle: f64,
    dzrcrit_lookup: &lookup::DzrCritLookup,
    lookup_params: &LookupParameters,
    params: &RunConfig,
) -> Result<TopoColumnParams, CostBuildError> {
    let slantrange = nearrange + col as f64 * params.dr;
    let (cosnomincangle, nomincangle, sinnomincangle) =
        incidence_geometry(params.orbitradius, params.earthradius, slantrange)?;
    let lookangle = (params.earthradius / params.orbitradius * sinnomincangle)
        .clamp(-1.0, 1.0)
        .asin();
    let dzr0 = -params.dr * cosnomincangle;
    let bperp = baseline * (lookangle - baseline_angle).cos();
    if !bperp.is_finite() || bperp == 0.0 {
        return Err(CostBuildError::InvalidConfiguration);
    }
    let ambiguity_height = -(params.lambda * slantrange * sinnomincangle) / (2.0 * bperp);
    if !ambiguity_height.is_finite() || ambiguity_height == 0.0 {
        return Err(CostBuildError::InvalidConfiguration);
    }
    let sigsqrhoconst = 2.0 * ambiguity_height * ambiguity_height / 12.0;
    let ztoshort = params.nshortcycle as f64 / ambiguity_height;
    let ztoshortsq = ztoshort * ztoshort;
    let sigsqlay = ambiguity_height * ambiguity_height * params.sigsqlayfactor;
    let dzrcrit = dzrcrit_lookup.interp(nomincangle) as f64;
    let model = solve_ei_model_params(dzrcrit, dzr0, sinnomincangle, cosnomincangle, lookup_params)
        .map_err(|_| CostBuildError::Lookup)?;
    let eicrit = (dzrcrit - model.const1) / model.slope1;
    let dphilaypeak = params.dzlaypeak / ambiguity_height;

    Ok(TopoColumnParams {
        ambiguity_height,
        dzr0,
        eicrit,
        dphilaypeak,
        sigsqrhoconst,
        ztoshort,
        ztoshortsq,
        sigsqlay,
        slope1: model.slope1,
        slope2: model.slope2,
        const1: model.const1,
        const2: model.const2,
        nomincangle,
    })
}

/// Build statistical cost records for topography mode.
///
/// `inputs` contains contiguous row-major image rasters. Returned costs use the
/// legacy row/column nested layout for compatibility with the current solver
/// stack: the first `nrow - 1` rows are azimuth arcs, followed by `nrow` rows
/// of range arcs.
pub fn build_stat_costs_topo(
    inputs: &CostBuildInputs<'_>,
    intensity: &IntensityCorrelation,
    arc_weights: &RowColTile<i16>,
    params: &RunConfig,
) -> Result<(Vec<Vec<Cost>>, TopoCostDiagnostics), CostBuildError> {
    let nrow = inputs.tile.rows;
    let ncol = inputs.tile.cols;
    if nrow < 2 || ncol < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    validate_raster_shape(&intensity.power, nrow, ncol)?;
    validate_raster_shape(&intensity.correlation, nrow, ncol)?;
    validate_raster_shape(inputs.wrapped_phase, nrow, ncol)?;
    validate_optional_raster_shape(inputs.unwrapped_estimate, nrow, ncol)?;
    if arc_weights.row_arcs.width != ncol
        || arc_weights.row_arcs.height != nrow - 1
        || arc_weights.col_arcs.width != ncol - 1
        || arc_weights.col_arcs.height != nrow
        || !arc_weights.row_arcs.has_valid_shape()
        || !arc_weights.col_arcs.has_valid_shape()
    {
        return Err(CostBuildError::InvalidInputShape);
    }

    let rho0 = params.rhosconst1 / params.ncorrlooks + params.rhosconst2;
    let rhomin = params.rhominfactor * rho0;
    let rhopow = 2.0 * params.cstd1
        + params.cstd2 * params.ncorrlooks.ln()
        + params.cstd3 * params.ncorrlooks;
    let nearrange = params.nearrange + params.dr * inputs.tile.first_col as f64;

    let mut ei = if nrow >= crate::constants::ARMLEN && ncol >= crate::constants::ARMLEN {
        despeckle(&intensity.power.data, nrow, ncol)
    } else {
        intensity.power.data.clone()
    };
    if remove_mean(&mut ei, nrow, ncol, params.krowei, params.kcolei).is_err() {
        // Small synthetic fixtures do not have enough support for the full EI
        // normalization window; keep the despeckled intensity instead.
    }
    let normalized_intensity = Raster::new(ncol, nrow, ei.clone());

    let midswath_range = nearrange + (ncol / 2) as f64 * params.dr;
    let (_, _, midswath_sin_nominc) =
        incidence_geometry(params.orbitradius, params.earthradius, midswath_range)?;
    let midswath_lookangle = (params.earthradius / params.orbitradius * midswath_sin_nominc)
        .clamp(-1.0, 1.0)
        .asin();

    let mut baseline = params.baseline;
    let mut baseline_angle = params.baseline_angle;
    if params.bperp != 0.0 {
        baseline_angle = if params.bperp > 0.0 {
            midswath_lookangle
        } else {
            midswath_lookangle + std::f64::consts::PI
        };
        baseline = params.bperp.abs();
    }
    if matches!(params.transmit_mode, TransmitMode::SingleAntenna) {
        baseline /= 2.0;
    }

    let lookup_params = topo_lookup_params(params, baseline, baseline_angle);
    let dzrcrit_lookup =
        build_dzrcrit_lookup(&inputs.tile, &lookup_params).map_err(|_| CostBuildError::Lookup)?;
    let nrho = (((1.0 - rhomin) / params.drho).floor() as isize + 1).max(1) as usize;
    let dzrho_lookup =
        build_dz_rho_max_lookup(&dzrcrit_lookup, rhomin, params.drho, nrho, &lookup_params)
            .map_err(|_| CostBuildError::Lookup)?;

    let midswath_bperp = baseline * (midswath_lookangle - baseline_angle).cos();
    if !midswath_bperp.is_finite() || midswath_bperp == 0.0 {
        return Err(CostBuildError::InvalidConfiguration);
    }
    let midrange_ambiguity_height =
        (params.lambda * midswath_range * midswath_sin_nominc / (2.0 * midswath_bperp)).abs();
    if !midrange_ambiguity_height.is_finite() || midrange_ambiguity_height == 0.0 {
        return Err(CostBuildError::InvalidConfiguration);
    }
    let costscale = params.costscale * (params.costscaleambight / midrange_ambiguity_height).abs();
    let mut glay = -costscale * params.layconst.ln();
    let noshadow = !params.shadow;

    let range_grads =
        compute_range_gradients_fallback(inputs.wrapped_phase, params.kperpdpsi, params.kpardpsi)?;
    let az_grads = compute_azimuth_gradients_fallback(
        inputs.wrapped_phase,
        params.kpardpsi,
        params.kperpdpsi,
    )?;

    let mut col_costs = vec![Cost::default(); nrow * (ncol - 1)];
    for col in 0..(ncol - 1) {
        let geom = build_topo_column_params(
            col,
            nearrange,
            baseline,
            baseline_angle,
            &dzrcrit_lookup,
            &lookup_params,
            params,
        )?;

        for row in 0..nrow {
            let weight = arc_weights.col_arcs.data[col_arc_idx(row, col, ncol)];
            let dst = col_arc_idx(row, col, ncol);
            if weight == 0 {
                let mut masked = Cost::default();
                mask_cost(&mut masked);
                col_costs[dst] = masked;
                continue;
            }

            let corr_idx = row_major_idx(row, col, ncol);
            let rho_raw = intensity.correlation.data[corr_idx] as f64;
            let rho = if rho_raw < rhomin { 0.0 } else { rho_raw };
            let sigsqrho = geom.sigsqrhoconst * (1.0 - rho).powf(rhopow);
            let ei_sample = ei[corr_idx] as f64;
            let mut dzei = if ei_sample > geom.eicrit {
                (geom.slope2 * ei_sample + geom.const2) * params.dzeifactor
            } else {
                (geom.slope1 * ei_sample + geom.const1) * params.dzeifactor
            };
            if noshadow && dzei < params.dzeimin {
                dzei = params.dzeimin;
            }

            let mut dzlay = 0.0;
            let mut lay_samples = 0usize;
            if ei_sample > params.layminei {
                let end_col = (col + params.laywidth).min(ncol);
                for sample_col in col..end_col {
                    let sample_ei = ei[row_major_idx(row, sample_col, ncol)] as f64;
                    dzlay += if sample_ei > geom.eicrit {
                        geom.slope2 * sample_ei + geom.const2
                    } else {
                        geom.slope1 * sample_ei + geom.const1
                    };
                    lay_samples += 1;
                }
            }
            if dzlay != 0.0 {
                dzlay = (dzlay + lay_samples as f64 * (-2.0 * geom.dzr0)) * params.dzlayfactor;
            }
            if rho > 0.0 {
                let dzrhomax = dzrho_lookup.interp(geom.nomincangle, rho) as f64;
                if dzrhomax < dzlay {
                    dzlay = dzrhomax;
                }
            }

            let mut nolayover = true;
            if dzlay != 0.0 {
                let offset = if rho > 0.0 {
                    params.nshortcycle as f64
                        * (range_grads.gradients()[dst] as f64
                            - 0.5 * (range_grads.averages()[dst] as f64 + geom.dphilaypeak))
                } else {
                    params.nshortcycle as f64
                        * (range_grads.gradients()[dst] as f64
                            - 0.25 * range_grads.averages()[dst] as f64
                            - 0.75 * geom.dphilaypeak)
                };
                let mut sigsq = (sigsqrho + params.sigsqei + geom.sigsqlay) * geom.ztoshortsq
                    / (costscale * f64::from(weight));
                if sigsq < params.sigsqshortmin as f64 {
                    sigsq = params.sigsqshortmin as f64;
                }
                let dzmax = dzlay * geom.ztoshort;
                let laycost = f64::from(weight) * glay;
                let cost = Cost {
                    offset: clamp_short(offset),
                    sigma_sq: clamp_short(sigsq),
                    dz_max: clamp_short(dzmax),
                    lay_cost: clamp_short(laycost),
                };
                if i64::from(cost.dz_max).abs()
                    <= ((f64::from(cost.lay_cost) * f64::from(cost.sigma_sq))
                        .sqrt()
                        .floor() as i64)
                {
                    nolayover = false;
                }
                col_costs[dst] = cost;
            }
            if nolayover {
                let mut sigsq =
                    (sigsqrho + params.sigsqei) * geom.ztoshortsq / (costscale * f64::from(weight));
                if sigsq < params.sigsqshortmin as f64 {
                    sigsq = params.sigsqshortmin as f64;
                }
                let offset = if rho > 0.0 {
                    geom.ztoshort
                        * (geom.ambiguity_height
                            * (range_grads.gradients()[dst] as f64
                                - 0.5 * range_grads.averages()[dst] as f64)
                            - 0.5 * params.dzeiweight * dzei)
                } else {
                    geom.ztoshort
                        * (geom.ambiguity_height
                            * (range_grads.gradients()[dst] as f64
                                - 0.25 * range_grads.averages()[dst] as f64)
                            - 0.75 * params.dzeiweight * dzei)
                };
                col_costs[dst] = Cost {
                    offset: clamp_short(offset),
                    sigma_sq: clamp_short(sigsq),
                    dz_max: LARGE_SHORT,
                    lay_cost: NOCOSTSHELF,
                };
            }
            if let Some(unwrapped_estimate) = inputs.unwrapped_estimate {
                let shift = params.nshortcycle as f64 / TWO_PI
                    * (unwrapped_estimate.data[row_major_idx(row, col + 1, ncol)] as f64
                        - unwrapped_estimate.data[row_major_idx(row, col, ncol)] as f64);
                let updated = f64::from(col_costs[dst].offset) + shift;
                col_costs[dst].offset = clamp_short(updated);
            }
        }
    }

    glay += -costscale * params.azdzfactor.ln();

    let mut row_costs = vec![Cost::default(); (nrow - 1) * ncol];
    for col in 0..ncol {
        let geom = build_topo_column_params(
            col,
            nearrange,
            baseline,
            baseline_angle,
            &dzrcrit_lookup,
            &lookup_params,
            params,
        )?;

        for row in 0..(nrow - 1) {
            let weight = arc_weights.row_arcs.data[row_arc_idx(row, col, ncol)];
            let dst = row_arc_idx(row, col, ncol);
            if weight == 0 {
                let mut masked = Cost::default();
                mask_cost(&mut masked);
                row_costs[dst] = masked;
                continue;
            }

            let rho_raw = (intensity.correlation.data[row_major_idx(row, col, ncol)] as f64
                + intensity.correlation.data[row_major_idx(row + 1, col, ncol)] as f64)
                / 2.0;
            let rho = if rho_raw < rhomin { 0.0 } else { rho_raw };
            let sigsqrho = geom.sigsqrhoconst * (1.0 - rho).powf(rhopow);
            let mut dzlay = 0.0;
            let mut lay_samples = 0usize;
            let avg_ei = (ei[row_major_idx(row, col, ncol)] as f64
                + ei[row_major_idx(row + 1, col, ncol)] as f64)
                / 2.0;
            if avg_ei > params.layminei {
                let end_col = (col + params.laywidth).min(ncol);
                for sample_col in col..end_col {
                    let sample_ei = (ei[row_major_idx(row, sample_col, ncol)] as f64
                        + ei[row_major_idx(row + 1, sample_col, ncol)] as f64)
                        / 2.0;
                    dzlay += if sample_ei > geom.eicrit {
                        geom.slope2 * sample_ei + geom.const2
                    } else {
                        geom.slope1 * sample_ei + geom.const1
                    };
                    lay_samples += 1;
                }
            }
            if dzlay != 0.0 {
                dzlay = (dzlay + lay_samples as f64 * (-2.0 * geom.dzr0)) * params.dzlayfactor;
            }
            if rho > 0.0 {
                let dzrhomax = dzrho_lookup.interp(geom.nomincangle, rho) as f64;
                if dzrhomax < dzlay {
                    dzlay = dzrhomax;
                }
            }

            let offset = if rho > 0.0 {
                params.nshortcycle as f64
                    * (az_grads.gradients()[dst] as f64 - az_grads.averages()[dst] as f64)
            } else {
                params.nshortcycle as f64
                    * (az_grads.gradients()[dst] as f64 - 0.5 * az_grads.averages()[dst] as f64)
            };
            let mut nolayover = true;
            if dzlay != 0.0 {
                let mut sigsq = (sigsqrho + params.sigsqei + geom.sigsqlay) * geom.ztoshortsq
                    / (costscale * f64::from(weight));
                if sigsq < params.sigsqshortmin as f64 {
                    sigsq = params.sigsqshortmin as f64;
                }
                let dzmax = (dzlay * geom.ztoshort).abs();
                let laycost = f64::from(weight) * glay;
                let cost = Cost {
                    offset: clamp_short(offset),
                    sigma_sq: clamp_short(sigsq),
                    dz_max: clamp_short(dzmax),
                    lay_cost: clamp_short(laycost),
                };
                if i64::from(cost.dz_max).abs()
                    <= ((f64::from(cost.lay_cost) * f64::from(cost.sigma_sq))
                        .sqrt()
                        .floor() as i64)
                {
                    nolayover = false;
                }
                row_costs[dst] = cost;
            }
            if nolayover {
                let mut sigsq =
                    (sigsqrho + params.sigsqei) * geom.ztoshortsq / (costscale * f64::from(weight));
                if sigsq < params.sigsqshortmin as f64 {
                    sigsq = params.sigsqshortmin as f64;
                }
                row_costs[dst] = Cost {
                    offset: clamp_short(offset),
                    sigma_sq: clamp_short(sigsq),
                    dz_max: LARGE_SHORT,
                    lay_cost: NOCOSTSHELF,
                };
            }
            if let Some(unwrapped_estimate) = inputs.unwrapped_estimate {
                let shift = params.nshortcycle as f64 / TWO_PI
                    * (unwrapped_estimate.data[row_major_idx(row + 1, col, ncol)] as f64
                        - unwrapped_estimate.data[row_major_idx(row, col, ncol)] as f64);
                let updated = f64::from(row_costs[dst].offset) + shift;
                row_costs[dst].offset = clamp_short(updated);
            }
        }
    }

    Ok((
        row_col_to_nested(&row_costs, &col_costs, nrow, ncol),
        TopoCostDiagnostics {
            normalized_intensity,
        },
    ))
}

/// Build statistical cost records for deformation mode.
pub fn build_stat_costs_defo(
    corr: &Raster<f32>,
    wrapped_phase: &Raster<f32>,
    arc_weights: &RowColTile<i16>,
    params: &RunConfig,
) -> Result<Vec<Vec<Cost>>, CostBuildError> {
    let nrow = corr.height;
    let ncol = corr.width;
    if nrow < 2 || ncol < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    validate_raster_shape(corr, nrow, ncol)?;
    validate_raster_shape(wrapped_phase, nrow, ncol)?;
    if arc_weights.row_arcs.width != ncol
        || arc_weights.row_arcs.height != nrow - 1
        || arc_weights.col_arcs.width != ncol - 1
        || arc_weights.col_arcs.height != nrow
    {
        return Err(CostBuildError::InvalidInputShape);
    }

    let range_grads =
        compute_range_gradients_fallback(wrapped_phase, params.kperpdpsi, params.kpardpsi)?;
    let az_grads =
        compute_azimuth_gradients_fallback(wrapped_phase, params.kpardpsi, params.kperpdpsi)?;

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

    let mut row_costs = vec![Cost::default(); (nrow - 1) * ncol];
    let mut col_costs = vec![Cost::default(); nrow * (ncol - 1)];
    let mut masked = 0usize;
    let mut low_corr = 0usize;

    for row in 0..(nrow - 1) {
        for col in 0..ncol {
            let dst = row_arc_idx(row, col, ncol);
            let weight = arc_weights.row_arcs.data[dst];
            if weight == 0 {
                let mut masked_cost = Cost::default();
                mask_cost(&mut masked_cost);
                row_costs[dst] = masked_cost;
                masked += 1;
                continue;
            }
            let rho_raw = (corr.data[row_major_idx(row, col, ncol)] as f64
                + corr.data[row_major_idx(row + 1, col, ncol)] as f64)
                / 2.0;
            let low = rho_raw < defocorrthresh;
            if low {
                low_corr += 1;
            }
            let rho = if low { 0.0 } else { rho_raw };
            let sigsqrho =
                (sigsqrhoconst * (1.0 - rho).powf(rhopow) + params.sigsqcorr) * nshortcyclesq;
            let offset = if rho > 0.0 {
                nshortcycle * (az_grads.gradients()[dst] as f64 - az_grads.averages()[dst] as f64)
            } else {
                nshortcycle
                    * (az_grads.gradients()[dst] as f64 - 0.5 * az_grads.averages()[dst] as f64)
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
            row_costs[dst] = Cost {
                offset: clamp_short(offset),
                sigma_sq: clamp_short(sigsq),
                dz_max: clamp_short(dzmax),
                lay_cost: clamp_short(laycost),
            };
        }
    }

    for row in 0..nrow {
        for col in 0..(ncol - 1) {
            let dst = col_arc_idx(row, col, ncol);
            let weight = arc_weights.col_arcs.data[dst];
            if weight == 0 {
                let mut masked_cost = Cost::default();
                mask_cost(&mut masked_cost);
                col_costs[dst] = masked_cost;
                masked += 1;
                continue;
            }
            let rho_raw = (corr.data[row_major_idx(row, col, ncol)] as f64
                + corr.data[row_major_idx(row, col + 1, ncol)] as f64)
                / 2.0;
            let low = rho_raw < defocorrthresh;
            if low {
                low_corr += 1;
            }
            let rho = if low { 0.0 } else { rho_raw };
            let sigsqrho =
                (sigsqrhoconst * (1.0 - rho).powf(rhopow) + params.sigsqcorr) * nshortcyclesq;
            let offset = if rho > 0.0 {
                nshortcycle
                    * (range_grads.gradients()[dst] as f64 - range_grads.averages()[dst] as f64)
            } else {
                nshortcycle
                    * (range_grads.gradients()[dst] as f64
                        - 0.5 * range_grads.averages()[dst] as f64)
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
            col_costs[dst] = Cost {
                offset: clamp_short(offset),
                sigma_sq: clamp_short(sigsq),
                dz_max: clamp_short(dzmax),
                lay_cost: clamp_short(laycost),
            };
        }
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
    Ok(row_col_to_nested(&row_costs, &col_costs, nrow, ncol))
}

/// Build statistical cost records for smooth mode.
pub fn build_stat_costs_smooth(
    corr: &Raster<f32>,
    wrapped_phase: &Raster<f32>,
    arc_weights: &RowColTile<i16>,
    params: &RunConfig,
) -> Result<Vec<Vec<SmoothCost>>, CostBuildError> {
    let nrow = corr.height;
    let ncol = corr.width;
    if nrow < 2 || ncol < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    validate_raster_shape(corr, nrow, ncol)?;
    validate_raster_shape(wrapped_phase, nrow, ncol)?;
    if arc_weights.row_arcs.width != ncol
        || arc_weights.row_arcs.height != nrow - 1
        || arc_weights.col_arcs.width != ncol - 1
        || arc_weights.col_arcs.height != nrow
    {
        return Err(CostBuildError::InvalidInputShape);
    }

    let range_grads =
        compute_range_gradients_fallback(wrapped_phase, params.kperpdpsi, params.kpardpsi)?;
    let az_grads =
        compute_azimuth_gradients_fallback(wrapped_phase, params.kpardpsi, params.kperpdpsi)?;

    let rho0 = params.rhosconst1 / params.ncorrlooks + params.rhosconst2;
    let defocorrthresh = params.defothreshfactor * rho0;
    let rhopow = 2.0 * params.cstd1
        + params.cstd2 * params.ncorrlooks.ln()
        + params.cstd3 * params.ncorrlooks;
    let sigsqrhoconst = 2.0 / 12.0;
    let nshortcycle = params.nshortcycle as f64;
    let nshortcyclesq = nshortcycle * nshortcycle;

    let mut row_costs = vec![SmoothCost::default(); (nrow - 1) * ncol];
    let mut col_costs = vec![SmoothCost::default(); nrow * (ncol - 1)];

    for row in 0..(nrow - 1) {
        for col in 0..ncol {
            let dst = row_arc_idx(row, col, ncol);
            let weight = arc_weights.row_arcs.data[dst];
            if weight == 0 {
                let mut masked_cost = SmoothCost::default();
                mask_smooth_cost(&mut masked_cost);
                row_costs[dst] = masked_cost;
                continue;
            }
            let rho_raw = (corr.data[row_major_idx(row, col, ncol)] as f64
                + corr.data[row_major_idx(row + 1, col, ncol)] as f64)
                / 2.0;
            let rho = if rho_raw < defocorrthresh {
                0.0
            } else {
                rho_raw
            };
            let sigsqrho =
                (sigsqrhoconst * (1.0 - rho).powf(rhopow) + params.sigsqcorr) * nshortcyclesq;
            let offset = if rho > 0.0 {
                nshortcycle * (az_grads.gradients()[dst] as f64 - az_grads.averages()[dst] as f64)
            } else {
                nshortcycle
                    * (az_grads.gradients()[dst] as f64 - 0.5 * az_grads.averages()[dst] as f64)
            };
            let mut sigsq = sigsqrho / (params.costscale * f64::from(weight));
            if sigsq < params.sigsqshortmin as f64 {
                sigsq = params.sigsqshortmin as f64;
            }
            row_costs[dst] = SmoothCost {
                offset: clamp_short(offset),
                sigma_sq: clamp_short(sigsq),
            };
        }
    }

    for row in 0..nrow {
        for col in 0..(ncol - 1) {
            let dst = col_arc_idx(row, col, ncol);
            let weight = arc_weights.col_arcs.data[dst];
            if weight == 0 {
                let mut masked_cost = SmoothCost::default();
                mask_smooth_cost(&mut masked_cost);
                col_costs[dst] = masked_cost;
                continue;
            }
            let rho_raw = (corr.data[row_major_idx(row, col, ncol)] as f64
                + corr.data[row_major_idx(row, col + 1, ncol)] as f64)
                / 2.0;
            let rho = if rho_raw < defocorrthresh {
                0.0
            } else {
                rho_raw
            };
            let sigsqrho =
                (sigsqrhoconst * (1.0 - rho).powf(rhopow) + params.sigsqcorr) * nshortcyclesq;
            let offset = if rho > 0.0 {
                nshortcycle
                    * (range_grads.gradients()[dst] as f64 - range_grads.averages()[dst] as f64)
            } else {
                nshortcycle
                    * (range_grads.gradients()[dst] as f64
                        - 0.5 * range_grads.averages()[dst] as f64)
            };
            let mut sigsq = sigsqrho / (params.costscale * f64::from(weight));
            if sigsq < params.sigsqshortmin as f64 {
                sigsq = params.sigsqshortmin as f64;
            }
            col_costs[dst] = SmoothCost {
                offset: clamp_short(offset),
                sigma_sq: clamp_short(sigsq),
            };
        }
    }

    Ok(row_col_to_nested(&row_costs, &col_costs, nrow, ncol))
}

/// Build arc-cost arrays (and MST costs) for the current cost mode.
///
/// `inputs` carries contiguous row-major tile rasters. The returned `costs`
/// keep the existing nested row/column arc layout only as a compatibility
/// adapter for solver code that has not yet been flattened.
pub fn build_cost_arrays(
    inputs: &CostBuildInputs<'_>,
    params: &RunConfig,
) -> Result<BuildCostArraysResult, CostBuildError> {
    let nrow = inputs.tile.rows;
    let ncol = inputs.tile.cols;
    if nrow < 2 || ncol < 2 {
        return Err(CostBuildError::InvalidDimensions);
    }
    validate_raster_shape(inputs.mag, nrow, ncol)?;
    validate_raster_shape(inputs.wrapped_phase, nrow, ncol)?;
    validate_optional_raster_shape(inputs.power, nrow, ncol)?;
    validate_optional_raster_shape(inputs.correlation, nrow, ncol)?;
    validate_optional_raster_shape(inputs.unwrapped_estimate, nrow, ncol)?;

    let ic = get_intensity_and_correlation(inputs, params.defaultcorr as f32)?;
    let mut arc_weights = if let Some(weights) = inputs.arc_weights {
        weights.clone()
    } else {
        default_arc_weights_from_magnitude(inputs.mag)?
    };
    mask_arc_weights_with_magnitude(&mut arc_weights, inputs.mag)?;

    let (costs, topo_diagnostics) = match params.cost_mode {
        CostMode::Topo => {
            let (costs, diagnostics) = build_stat_costs_topo(inputs, &ic, &arc_weights, params)?;
            (CostArrayData::Topo(costs), Some(diagnostics))
        }
        CostMode::Defo => (
            CostArrayData::Defo(build_stat_costs_defo(
                &ic.correlation,
                inputs.wrapped_phase,
                &arc_weights,
                params,
            )?),
            None,
        ),
        CostMode::Smooth => (
            CostArrayData::Smooth(build_stat_costs_smooth(
                &ic.correlation,
                inputs.wrapped_phase,
                &arc_weights,
                params,
            )?),
            None,
        ),
        CostMode::NoStatCosts => (
            CostArrayData::Scalar(row_col_to_nested(
                &arc_weights.row_arcs.data,
                &arc_weights.col_arcs.data,
                nrow,
                ncol,
            )),
            None,
        ),
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
        topo_diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_raster(width: usize, height: usize, value: f32) -> Raster<f32> {
        Raster::new(width, height, vec![value; width * height])
    }

    #[test]
    fn default_arc_weights_mask_zero_magnitude_pixels() {
        let mag = Raster::new(2, 2, vec![1.0, 0.0, 1.0, 1.0]);
        let weights = default_arc_weights_from_magnitude(&mag).unwrap();
        assert_eq!(weights.row_arcs.data, vec![1, 0]);
        assert_eq!(weights.col_arcs.data, vec![0, 1]);
    }

    #[test]
    fn get_intensity_and_correlation_uses_contiguous_defaults() {
        let mag = sample_raster(2, 2, 2.0);
        let wrapped = sample_raster(2, 2, 0.0);
        let inputs = CostBuildInputs {
            tile: TileRegion::new(0, 0, 2, 2),
            mag: &mag,
            wrapped_phase: &wrapped,
            power: None,
            correlation: None,
            unwrapped_estimate: None,
            arc_weights: None,
        };
        let ic = get_intensity_and_correlation(&inputs, 0.25).unwrap();
        assert_eq!(ic.power.data, vec![4.0; 4]);
        assert_eq!(ic.correlation.data, vec![0.25; 4]);
    }

    #[test]
    fn build_cost_arrays_dispatches_modes() {
        let mag = sample_raster(2, 2, 1.0);
        let wrapped = sample_raster(2, 2, 0.0);
        let params = RunConfig::default();
        let inputs = CostBuildInputs {
            tile: TileRegion::new(0, 0, 2, 2),
            mag: &mag,
            wrapped_phase: &wrapped,
            power: None,
            correlation: None,
            unwrapped_estimate: None,
            arc_weights: None,
        };
        let out = build_cost_arrays(&inputs, &params).unwrap();
        assert!(matches!(out.costs, CostArrayData::Topo(_)));
        assert_eq!(out.mst_costs.len(), 3);
        assert!(out.topo_diagnostics.is_some());
    }

    #[test]
    fn nested_round_trip_preserves_row_col_layout() {
        let values = vec![vec![1, 2], vec![3], vec![4]];
        let (row_data, col_data) = nested_to_row_col(&values, 2, 2).unwrap();
        let round = row_col_to_nested(&row_data, &col_data, 2, 2);
        assert_eq!(round, values);
    }
}
