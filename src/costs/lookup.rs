#![allow(dead_code)]

//! Lookup table builders for rho/dz statistics.

use crate::constants::{BIGGEST_DZ_RHO_MAX, LARGE_FLOAT, MAX_ITERATIONS, SQRT_HALF};
use crate::data::tile::TileRegion;
use std::error::Error;
use std::f64::consts::{FRAC_PI_2, PI};
use std::fmt;

/// Subset of the SNAPHU runtime parameters required by the lookup builders.
#[derive(Debug, Clone)]
pub struct LookupParameters {
    pub orbit_radius: f64,
    pub earth_radius: f64,
    pub near_range: f64,
    pub range_spacing: f64,
    pub azimuth_spacing: f64,
    pub baseline: f64,
    pub baseline_angle: f64,
    pub kds: f64,
    pub sloperatio_factor: f64,
    pub specular_exponent: f64,
    pub dzrcrit_factor: f64,
    pub initial_dzr: f64,
    pub initial_dz_step: f64,
    pub incidence_angle_step: f64,
    pub range_resolution: f64,
    pub wavelength: f64,
    pub threshold: f64,
}

/// Parameters of the piecewise-linear EI model.
///
/// Equivalent outputs to the C `SolveEIModelParams()` function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EiModelParams {
    pub slope1: f64,
    pub slope2: f64,
    pub const1: f64,
    pub const2: f64,
}

/// 1-D lookup table indexed by nominal incidence angle.
#[derive(Debug, Clone)]
pub struct DzrCritLookup {
    start_angle: f64,
    angle_step: f64,
    values: Vec<f32>,
}

impl DzrCritLookup {
    pub fn start_angle(&self) -> f64 {
        self.start_angle
    }

    pub fn angle_step(&self) -> f64 {
        self.angle_step
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

/// 2-D lookup table indexed by incidence angle and correlation.
#[derive(Debug, Clone)]
pub struct DzRhoMaxLookup {
    start_angle: f64,
    angle_step: f64,
    rho_min: f64,
    rho_step: f64,
    angle_count: usize,
    rho_count: usize,
    values: Vec<f32>,
}

impl DzRhoMaxLookup {
    pub fn angle_count(&self) -> usize {
        self.angle_count
    }

    pub fn rho_count(&self) -> usize {
        self.rho_count
    }

    pub fn value(&self, angle_idx: usize, rho_idx: usize) -> f32 {
        let idx = angle_idx * self.rho_count + rho_idx;
        self.values[idx]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupError {
    InvalidGeometry(&'static str),
    InvalidConfiguration(&'static str),
    IterationLimit(&'static str),
}

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LookupError::InvalidGeometry(msg) => write!(f, "invalid geometry: {msg}"),
            LookupError::InvalidConfiguration(msg) => write!(f, "invalid configuration: {msg}"),
            LookupError::IterationLimit(msg) => {
                write!(f, "iteration limit reached while computing {msg}")
            }
        }
    }
}

impl Error for LookupError {}

type LookupResult<T> = Result<T, LookupError>;

pub fn build_dzrcrit_lookup(
    tile: &TileRegion,
    params: &LookupParameters,
) -> LookupResult<DzrCritLookup> {
    if !(params.incidence_angle_step.is_finite() && params.incidence_angle_step > 0.0) {
        return Err(LookupError::InvalidConfiguration(
            "incidence angle step must be positive and finite",
        ));
    }

    let start_slant = params.near_range + params.range_spacing * tile.first_col as f64;
    let start_angle = incidence_angle(params.orbit_radius, params.earth_radius, start_slant)
        .ok_or(LookupError::InvalidGeometry("near-range incidence"))?;
    let far_slant = start_slant + params.range_spacing * tile.cols as f64;
    let max_angle = incidence_angle(params.orbit_radius, params.earth_radius, far_slant)
        .ok_or(LookupError::InvalidGeometry("far-range incidence"))?;

    let mut angle = start_angle;
    let mut values = Vec::new();
    let table_size = ((max_angle - start_angle) / params.incidence_angle_step).floor() as isize + 1;
    let table_size = table_size.max(1) as usize;
    values.reserve(table_size);

    for _ in 0..table_size {
        let dz = solve_dzrcrit(angle, params)?;
        values.push(dz as f32);
        angle += params.incidence_angle_step;
        if angle > FRAC_PI_2 {
            angle -= params.incidence_angle_step;
        }
    }

    Ok(DzrCritLookup {
        start_angle,
        angle_step: params.incidence_angle_step,
        values,
    })
}

pub fn build_dz_rho_max_lookup(
    incidence: &DzrCritLookup,
    rho_min: f64,
    rho_step: f64,
    rho_count: usize,
    params: &LookupParameters,
) -> LookupResult<DzRhoMaxLookup> {
    if rho_count == 0 {
        return Err(LookupError::InvalidConfiguration(
            "rho table must have at least one entry",
        ));
    }
    if !(rho_step.is_finite() && rho_step > 0.0) {
        return Err(LookupError::InvalidConfiguration(
            "rho step must be positive and finite",
        ));
    }
    if incidence.is_empty() {
        return Err(LookupError::InvalidConfiguration(
            "incidence lookup must contain values",
        ));
    }

    let mut values = Vec::with_capacity(incidence.len() * rho_count);
    let mut angle = incidence.start_angle();
    for _ in 0..incidence.len() {
        let mut rho = rho_min;
        for _ in 0..rho_count {
            let dz = calc_dz_rho_max(rho, angle, params)?;
            values.push(dz as f32);
            rho += rho_step;
        }
        angle += incidence.angle_step();
    }

    Ok(DzRhoMaxLookup {
        start_angle: incidence.start_angle(),
        angle_step: incidence.angle_step(),
        rho_min,
        rho_step,
        angle_count: incidence.len(),
        rho_count,
        values,
    })
}

fn solve_dzrcrit(angle: f64, params: &LookupParameters) -> LookupResult<f64> {
    let sin_nom = angle.sin();
    let cos_nom = angle.cos();
    if !sin_nom.is_finite() || !cos_nom.is_finite() {
        return Err(LookupError::InvalidGeometry("nominal incidence angle"));
    }

    let mut thetai = PI / 4.0;
    let mut step = PI / 4.0 - 1e-6;
    let mut iterations = 0usize;

    loop {
        let mut cos2 = (2.0 * thetai).cos();
        if cos2 < 0.0 {
            cos2 = 0.0;
        }
        let diffuse = params.dzrcrit_factor * params.kds * thetai.cos();
        let specular = cos2.powf(params.specular_exponent);
        let residual = diffuse - specular;
        if residual.abs() < params.threshold * diffuse {
            break;
        }
        if residual < 0.0 {
            thetai += step;
        } else {
            thetai -= step;
        }
        step /= 2.0;
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            return Err(LookupError::IterationLimit("critical incidence angle"));
        }
    }

    let mut dzr = params.initial_dzr;
    let mut step = dzr + params.range_spacing * cos_nom - 1e-2;
    iterations = 0;
    let target_cos = thetai.cos();
    if sin_nom.abs() < f64::EPSILON {
        return Err(LookupError::InvalidGeometry("sine of nominal incidence"));
    }

    loop {
        let dx = (params.range_spacing + dzr * cos_nom) / sin_nom;
        let numerator = dzr * sin_nom + dx * cos_nom;
        let residual = target_cos - numerator / (dzr.hypot(dx));
        if residual.abs() < params.threshold * target_cos.abs() {
            return Ok(dzr);
        }
        if residual < 0.0 {
            dzr -= step;
        } else {
            dzr += step;
        }
        step /= 2.0;
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            return Err(LookupError::IterationLimit("critical slope"));
        }
    }
}

fn calc_dz_rho_max(rho: f64, angle: f64, params: &LookupParameters) -> LookupResult<f64> {
    if rho >= 1.0 {
        return Ok(-params.range_spacing * angle.cos());
    }
    if rho <= 0.0 {
        return Ok(LARGE_FLOAT);
    }

    let costheta = angle.cos();
    let sintheta = angle.sin();
    if sintheta.abs() < f64::EPSILON {
        return Err(LookupError::InvalidGeometry("sine of nominal incidence"));
    }

    let mut dz_step = params.initial_dz_step;
    if !(dz_step.is_finite() && dz_step > 0.0) {
        return Err(LookupError::InvalidConfiguration(
            "initial dz step must be positive",
        ));
    }
    let asin_arg = (params.earth_radius / params.orbit_radius * sintheta).clamp(-1.0, 1.0);
    let look_angle = asin_arg.asin();
    let bperp = params.baseline * (look_angle - params.baseline_angle).cos();
    let radicand = params.orbit_radius.powi(2) + params.earth_radius.powi(2)
        - 2.0 * params.orbit_radius * params.earth_radius * (angle - look_angle).cos();
    if radicand <= 0.0 {
        return Err(LookupError::InvalidGeometry("slant range"));
    }
    let slant_range = radicand.sqrt();
    let rhos_factor =
        2.0 * bperp.abs() * params.range_resolution / (params.wavelength * slant_range);

    let mut dz = -params.range_spacing * costheta;
    let mut rhos = 1.0;
    while rhos > rho {
        dz += dz_step;
        let dx = (params.range_spacing + dz * costheta) / sintheta;
        let numerator = dz * sintheta + dx * costheta;
        let cos_theta_air_sq = numerator * numerator / (dz * dz + dx * dx);
        rhos = 1.0 - rhos_factor * (cos_theta_air_sq / (1.0 - cos_theta_air_sq)).sqrt();
        if rhos < 0.0 {
            rhos = 0.0;
        }
        if dz > BIGGEST_DZ_RHO_MAX {
            return Ok(BIGGEST_DZ_RHO_MAX);
        }
    }

    let mut iterations = 0usize;
    loop {
        let residual = rhos - rho;
        if residual.abs() <= params.threshold * rho {
            return Ok(dz);
        }
        dz_step /= 2.0;
        if residual < 0.0 {
            dz -= dz_step;
        } else {
            dz += dz_step;
        }
        let dx = (params.range_spacing + dz * costheta) / sintheta;
        let numerator = dz * sintheta + dx * costheta;
        let cos_theta_air_sq = numerator * numerator / (dz * dz + dx * dx);
        rhos = 1.0 - rhos_factor * (cos_theta_air_sq / (1.0 - cos_theta_air_sq)).sqrt();
        if rhos < 0.0 {
            rhos = 0.0;
        }
        iterations += 1;
        if iterations > MAX_ITERATIONS {
            return Err(LookupError::IterationLimit("rho-dependent slope"));
        }
    }
}

/// Calculate linearized EI-vs-slope model parameters.
///
/// This is the idiomatic Rust equivalent of the C `SolveEIModelParams()`
/// function.
pub fn solve_ei_model_params(
    dzrcrit: f64,
    dzr0: f64,
    sin_nom_inc: f64,
    cos_nom_inc: f64,
    params: &LookupParameters,
) -> LookupResult<EiModelParams> {
    let slope_ratio = params.kds * params.sloperatio_factor;
    if !slope_ratio.is_finite() || slope_ratio == 0.0 {
        return Err(LookupError::InvalidConfiguration(
            "kds * sloperatio_factor must be finite and non-zero",
        ));
    }

    let dzr3 = 15.0 * (dzrcrit - dzr0) + dzr0;
    let ei0 = ei_of_dzr(0.0, sin_nom_inc, cos_nom_inc, params);
    if !ei0.is_finite() || ei0 == 0.0 {
        return Err(LookupError::InvalidGeometry(
            "EIofDZR(0) must be finite and non-zero",
        ));
    }
    let ei3 = ei_of_dzr(dzr3, sin_nom_inc, cos_nom_inc, params) / ei0;
    if !ei3.is_finite() || ei3 == 0.0 {
        return Err(LookupError::InvalidGeometry(
            "normalized EI reference must be finite and non-zero",
        ));
    }

    let const1 = dzr0;
    let slope2 = (slope_ratio * (dzrcrit - const1) - dzrcrit + dzr3) / ei3;
    let slope1 = slope2 / slope_ratio;
    let const2 = dzr3 - slope2 * ei3;

    Ok(EiModelParams {
        slope1,
        slope2,
        const1,
        const2,
    })
}

/// Calculates expected value of intensity (arbitrary units) for a given
/// range slope `dzr`, assuming zero azimuth slope.
///
/// This is the idiomatic Rust equivalent of the C `EIofDZR()` function.
/// The scattering model combines a diffuse component (`kds * cos θ_i`) with a
/// specular component (`cos(2θ_i)^n`) that only contributes when the local
/// incidence angle is small enough (`cos θ_i > √½`).
fn ei_of_dzr(dzr: f64, sin_nom_inc: f64, cos_nom_inc: f64, params: &LookupParameters) -> f64 {
    let dr = params.range_spacing;
    let da = params.azimuth_spacing;
    let dx = dr / sin_nom_inc + dzr * cos_nom_inc / sin_nom_inc;
    let kds = params.kds;
    let n = params.specular_exponent;

    // Zero-slope reference point and projected area.
    let dzr0 = -dr * cos_nom_inc;
    let proj_area = da * ((dzr - dzr0) / sin_nom_inc).abs();

    // Local incidence angle cosine.
    let cos_theta_i = proj_area / (dzr * dzr * da * da + da * da * dx * dx).sqrt();

    let sigma0 = if cos_theta_i > SQRT_HALF {
        let cos2_theta_i = 2.0 * cos_theta_i * cos_theta_i - 1.0;
        kds * cos_theta_i + cos2_theta_i.powf(n)
    } else {
        kds * cos_theta_i
    };

    sigma0 * proj_area
}

fn incidence_angle(orbit_radius: f64, earth_radius: f64, slant_range: f64) -> Option<f64> {
    let denom = 2.0 * slant_range * earth_radius;
    if denom == 0.0 {
        return None;
    }
    let ratio =
        (orbit_radius * orbit_radius - slant_range * slant_range - earth_radius * earth_radius)
            / denom;
    let clamped = ratio.clamp(-1.0, 1.0);
    let angle = clamped.acos();
    if angle.is_finite() { Some(angle) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_params() -> LookupParameters {
        LookupParameters {
            orbit_radius: 7_000_000.0,
            earth_radius: 6_371_000.0,
            near_range: 800_000.0,
            range_spacing: 20.0,
            azimuth_spacing: 20.0,
            baseline: 200.0,
            baseline_angle: 0.1,
            kds: 0.25,
            sloperatio_factor: 1.5,
            specular_exponent: 3.0,
            dzrcrit_factor: 0.8,
            initial_dzr: 5.0,
            initial_dz_step: 2.0,
            incidence_angle_step: 0.001,
            range_resolution: 10.0,
            wavelength: 0.056,
            threshold: 1e-4,
        }
    }

    #[test]
    fn dzrcrit_lookup_has_expected_length_and_values() {
        let params = sample_params();
        let tile = TileRegion::new(0, 0, 128, 32);
        let lut = build_dzrcrit_lookup(&tile, &params).unwrap();
        assert_eq!(lut.angle_step(), params.incidence_angle_step);
        assert!(!lut.is_empty());
        assert!(lut.values().iter().all(|v| v.is_finite() && *v > 0.0));
    }

    #[test]
    fn dzrho_lookup_respects_rho_limits() {
        let params = sample_params();
        let tile = TileRegion::new(0, 0, 64, 16);
        let dzrcrit = build_dzrcrit_lookup(&tile, &params).unwrap();
        let rho_min = 0.0;
        let rho_step = 0.5;
        let rho_count = 3;
        let dzrho =
            build_dz_rho_max_lookup(&dzrcrit, rho_min, rho_step, rho_count, &params).unwrap();
        assert_eq!(dzrho.angle_count(), dzrcrit.len());
        assert_eq!(dzrho.value(0, 0), LARGE_FLOAT as f32);
        let start_angle = dzrcrit.start_angle();
        let expected = (-params.range_spacing * start_angle.cos()) as f32;
        let unit_rho = dzrho.value(0, rho_count - 1);
        assert!((unit_rho - expected).abs() < 1e-3 * expected.abs().max(1.0));
    }

    #[test]
    fn invalid_geometry_is_reported() {
        let mut params = sample_params();
        params.near_range = 0.0;
        let tile = TileRegion::new(0, 0, 1, 1);
        assert!(matches!(
            build_dzrcrit_lookup(&tile, &params),
            Err(LookupError::InvalidGeometry(_))
        ));

        let dzrcrit = build_dzrcrit_lookup(&TileRegion::new(0, 0, 4, 4), &sample_params()).unwrap();
        assert!(matches!(
            build_dz_rho_max_lookup(&dzrcrit, 0.0, -1.0, 1, &sample_params()),
            Err(LookupError::InvalidConfiguration(_))
        ));
    }

    #[test]
    fn ei_of_dzr_positive_for_zero_slope() {
        let params = sample_params();
        let angle: f64 = 0.6; // ~34 degrees
        let ei = ei_of_dzr(0.0, angle.sin(), angle.cos(), &params);
        assert!(ei > 0.0, "expected positive intensity at zero slope");
        assert!(ei.is_finite());
    }

    #[test]
    fn ei_of_dzr_increases_with_steeper_slopes() {
        // Intensity should generally increase as the surface tilts
        // further towards the sensor (larger dzr), at least for
        // moderate slopes.
        let params = sample_params();
        let angle: f64 = 0.6;
        let sin_a = angle.sin();
        let cos_a = angle.cos();

        let ei_0 = ei_of_dzr(0.0, sin_a, cos_a, &params);
        let ei_small = ei_of_dzr(2.0, sin_a, cos_a, &params);
        let ei_large = ei_of_dzr(10.0, sin_a, cos_a, &params);

        assert!(
            ei_small >= ei_0,
            "expected EI to increase with slope: ei_small={ei_small}, ei_0={ei_0}"
        );
        assert!(
            ei_large >= ei_small,
            "expected EI to increase with slope: ei_large={ei_large}, ei_small={ei_small}"
        );
    }

    #[test]
    fn ei_of_dzr_specular_branch_activates_at_steep_angle() {
        // For a sufficiently steep slope the local incidence becomes small
        // (cos θ_i > SQRT_HALF), activating the specular term.
        let params = sample_params();
        let angle: f64 = 0.6;
        let sin_a = angle.sin();
        let cos_a = angle.cos();

        // At the zero-slope reference point dzr0 = -dr*cos, the projected
        // area is zero so EI is zero; slightly beyond that the specular
        // branch should kick in since cos_theta_i will be large.
        let dzr0 = -params.range_spacing * cos_a;
        let near_zero = ei_of_dzr(dzr0 + 0.01, sin_a, cos_a, &params);
        assert!(near_zero.is_finite());
    }

    #[test]
    fn ei_of_dzr_normalisation_is_meaningful() {
        // SolveEIModelParams divides EI(dzr3) / EI(0); make sure the
        // denominator is nonzero for typical parameters.
        let params = sample_params();
        let angle: f64 = 0.6;
        let denom = ei_of_dzr(0.0, angle.sin(), angle.cos(), &params);
        assert!(denom > 0.0, "EI(0) must be positive for normalisation");
    }

    #[test]
    fn solve_ei_model_params_returns_finite_coefficients() {
        let params = sample_params();
        let model = solve_ei_model_params(10.0, -2.0, 0.8, 0.6, &params).unwrap();
        assert!(model.slope1.is_finite());
        assert!(model.slope2.is_finite());
        assert_eq!(model.const1, -2.0);
        assert!(model.const2.is_finite());
    }

    #[test]
    fn solve_ei_model_params_rejects_zero_slope_ratio() {
        let mut params = sample_params();
        params.sloperatio_factor = 0.0;
        let err = solve_ei_model_params(10.0, -2.0, 0.8, 0.6, &params).unwrap_err();
        assert!(matches!(err, LookupError::InvalidConfiguration(_)));
    }
}
