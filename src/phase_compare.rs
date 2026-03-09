//! Shared utilities for comparing two float32 phase rasters.
//!
//! The helper functions in this module are used by small diagnostic CLIs under
//! `src/bin/` to analyze C-vs-Rust phase outputs.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub const TWO_PI: f64 = std::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct ErrorStats {
    pub mae: f64,
    pub rmse: f64,
    pub max_abs: f64,
}

#[derive(Debug, Clone)]
pub struct KMapSummary {
    pub row_medians: Vec<f64>,
    pub col_medians: Vec<f64>,
    pub mean_row_variance: f64,
    pub mean_col_variance: f64,
    pub global_mean: f64,
    pub global_std: f64,
}

#[derive(Debug, Clone)]
pub struct GradientComparison {
    pub dx_raw: ErrorStats,
    pub dy_raw: ErrorStats,
    pub dx_wrapped: ErrorStats,
    pub dy_wrapped: ErrorStats,
}

#[derive(Debug, Clone)]
pub struct TransformScore {
    pub name: String,
    pub raw_rmse: f64,
    pub mod_rmse: f64,
    pub mod_mae: f64,
    pub mod_max_abs: f64,
}

pub fn expected_bytes(width: usize, height: usize) -> io::Result<usize> {
    width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "raster dimensions overflow"))
}

pub fn read_f32_raster(path: &Path, width: usize, height: usize) -> io::Result<Vec<f32>> {
    let mut file = File::open(path)?;
    let expected = expected_bytes(width, height)?;
    let actual = file.metadata()?.len() as usize;
    if expected != actual {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{}: expected {} bytes ({}x{}x4), got {}",
                path.display(),
                expected,
                width,
                height,
                actual
            ),
        ));
    }
    let mut buf = vec![0u8; expected];
    file.read_exact(&mut buf)?;
    Ok(bytes_to_f32_native(&buf))
}

pub fn read_f32_raster_with_bytes(
    path: &Path,
    width: usize,
    height: usize,
) -> io::Result<(Vec<f32>, Vec<u8>)> {
    let mut file = File::open(path)?;
    let expected = expected_bytes(width, height)?;
    let actual = file.metadata()?.len() as usize;
    if expected != actual {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{}: expected {} bytes ({}x{}x4), got {}",
                path.display(),
                expected,
                width,
                height,
                actual
            ),
        ));
    }
    let mut buf = vec![0u8; expected];
    file.read_exact(&mut buf)?;
    let data = bytes_to_f32_native(&buf);
    Ok((data, buf))
}

pub fn write_f32_raster(path: &Path, data: &[f32]) -> io::Result<()> {
    let mut file = File::create(path)?;
    let mut raw = Vec::with_capacity(data.len() * 4);
    for &v in data {
        raw.extend_from_slice(&v.to_ne_bytes());
    }
    file.write_all(&raw)?;
    Ok(())
}

pub fn bytes_to_f32_native(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_ne_bytes(c.try_into().unwrap_or([0; 4])))
        .collect()
}

pub fn bytes_to_f32_swapped(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_ne_bytes([c[3], c[2], c[1], c[0]]))
        .collect()
}

pub fn wrap_to_pi(v: f64) -> f64 {
    v - (v / TWO_PI).round() * TWO_PI
}

pub fn diff_mod_k(a: &[f32], b: &[f32]) -> (Vec<f32>, Vec<f32>, Vec<i32>) {
    let mut diff = Vec::with_capacity(a.len());
    let mut mod_res = Vec::with_capacity(a.len());
    let mut k = Vec::with_capacity(a.len());
    for (&av, &bv) in a.iter().zip(b) {
        let d = f64::from(bv) - f64::from(av);
        let cycles = (d / TWO_PI).round() as i32;
        let m = d - f64::from(cycles) * TWO_PI;
        diff.push(d as f32);
        mod_res.push(m as f32);
        k.push(cycles);
    }
    (diff, mod_res, k)
}

pub fn error_stats_from_values(values: &[f32]) -> ErrorStats {
    let n = values.len().max(1) as f64;
    let mut sum_abs = 0.0f64;
    let mut sum_sq = 0.0f64;
    let mut max_abs = 0.0f64;
    for &v in values {
        let x = f64::from(v);
        let ax = x.abs();
        sum_abs += ax;
        sum_sq += x * x;
        if ax > max_abs {
            max_abs = ax;
        }
    }
    ErrorStats {
        mae: sum_abs / n,
        rmse: (sum_sq / n).sqrt(),
        max_abs,
    }
}

pub fn error_stats(a: &[f32], b: &[f32]) -> ErrorStats {
    let mut diff = Vec::with_capacity(a.len());
    for (&av, &bv) in a.iter().zip(b) {
        diff.push(bv - av);
    }
    error_stats_from_values(&diff)
}

pub fn wrapped_error_stats(a: &[f32], b: &[f32]) -> ErrorStats {
    let mut wrapped = Vec::with_capacity(a.len());
    for (&av, &bv) in a.iter().zip(b) {
        let d = f64::from(bv) - f64::from(av);
        wrapped.push(wrap_to_pi(d) as f32);
    }
    error_stats_from_values(&wrapped)
}

pub fn k_histogram(k: &[i32]) -> BTreeMap<i32, usize> {
    let mut hist = BTreeMap::new();
    for &v in k {
        *hist.entry(v).or_insert(0) += 1;
    }
    hist
}

pub fn summarize_k_map(k: &[i32], width: usize, height: usize) -> KMapSummary {
    let mut row_medians = Vec::with_capacity(height);
    let mut col_medians = Vec::with_capacity(width);

    let mut row_vars = Vec::with_capacity(height);
    let mut col_vars = Vec::with_capacity(width);

    for r in 0..height {
        let row = &k[r * width..(r + 1) * width];
        row_medians.push(median_i32(row));
        row_vars.push(variance_i32(row));
    }
    for c in 0..width {
        let mut col = Vec::with_capacity(height);
        for r in 0..height {
            col.push(k[r * width + c]);
        }
        col_medians.push(median_i32(&col));
        col_vars.push(variance_i32(&col));
    }

    let mean_row_variance = mean_f64(&row_vars);
    let mean_col_variance = mean_f64(&col_vars);

    let global_mean = mean_i32(k);
    let global_std = variance_i32(k).sqrt();

    KMapSummary {
        row_medians,
        col_medians,
        mean_row_variance,
        mean_col_variance,
        global_mean,
        global_std,
    }
}

pub fn gradient_compare(a: &[f32], b: &[f32], width: usize, height: usize) -> GradientComparison {
    let (adx, ady) = gradients(a, width, height);
    let (bdx, bdy) = gradients(b, width, height);

    let dx_raw = error_stats(&adx, &bdx);
    let dy_raw = error_stats(&ady, &bdy);
    let dx_wrapped = wrapped_error_stats(&adx, &bdx);
    let dy_wrapped = wrapped_error_stats(&ady, &bdy);

    GradientComparison {
        dx_raw,
        dy_raw,
        dx_wrapped,
        dy_wrapped,
    }
}

pub fn transform_candidates(
    b_native: &[f32],
    b_swapped: &[f32],
    width: usize,
    height: usize,
) -> Vec<(String, Vec<f32>)> {
    let mut out = vec![
        ("identity".to_string(), b_native.to_vec()),
        ("flip_x".to_string(), flip_x(b_native, width, height)),
        ("flip_y".to_string(), flip_y(b_native, width, height)),
        (
            "flip_xy".to_string(),
            flip_y(&flip_x(b_native, width, height), width, height),
        ),
        (
            "col_major_reinterpret".to_string(),
            col_major_reinterpret(b_native, width, height),
        ),
        ("identity_endian_swap".to_string(), b_swapped.to_vec()),
    ];

    if width == height {
        out.push((
            "transpose_square".to_string(),
            transpose_square(b_native, width),
        ));
        out.push((
            "transpose_square_endian_swap".to_string(),
            transpose_square(b_swapped, width),
        ));
    }
    out
}

pub fn score_transforms(a: &[f32], candidates: Vec<(String, Vec<f32>)>) -> Vec<TransformScore> {
    let mut scores = Vec::with_capacity(candidates.len());
    for (name, data) in candidates {
        if data.len() != a.len() {
            continue;
        }
        let raw = error_stats(a, &data);
        let wrapped = wrapped_error_stats(a, &data);
        scores.push(TransformScore {
            name,
            raw_rmse: raw.rmse,
            mod_rmse: wrapped.rmse,
            mod_mae: wrapped.mae,
            mod_max_abs: wrapped.max_abs,
        });
    }
    scores.sort_by(|lhs, rhs| {
        lhs.mod_rmse
            .partial_cmp(&rhs.mod_rmse)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                lhs.raw_rmse
                    .partial_cmp(&rhs.raw_rmse)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    scores
}

pub fn min_max(values: &[f32]) -> (f32, f32) {
    let mut min_v = f32::INFINITY;
    let mut max_v = f32::NEG_INFINITY;
    for &v in values {
        if v < min_v {
            min_v = v;
        }
        if v > max_v {
            max_v = v;
        }
    }
    if min_v.is_infinite() || max_v.is_infinite() {
        (0.0, 0.0)
    } else {
        (min_v, max_v)
    }
}

pub fn linear_scale_u8(values: &[f32], min_v: f32, max_v: f32) -> Vec<u8> {
    if max_v <= min_v {
        return vec![127u8; values.len()];
    }
    let span = max_v - min_v;
    values
        .iter()
        .map(|&v| {
            let t = ((v - min_v) / span).clamp(0.0, 1.0);
            (255.0 * t) as u8
        })
        .collect()
}

fn gradients(data: &[f32], width: usize, height: usize) -> (Vec<f32>, Vec<f32>) {
    let mut dx = Vec::with_capacity(height.saturating_mul(width.saturating_sub(1)));
    let mut dy = Vec::with_capacity(height.saturating_sub(1).saturating_mul(width));

    if width >= 2 {
        for r in 0..height {
            let row_base = r * width;
            for c in 0..(width - 1) {
                dx.push(data[row_base + c + 1] - data[row_base + c]);
            }
        }
    }
    if height >= 2 {
        for r in 0..(height - 1) {
            let row_base = r * width;
            for c in 0..width {
                dy.push(data[row_base + width + c] - data[row_base + c]);
            }
        }
    }
    (dx, dy)
}

fn flip_x(data: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; data.len()];
    for r in 0..height {
        for c in 0..width {
            out[r * width + c] = data[r * width + (width - 1 - c)];
        }
    }
    out
}

fn flip_y(data: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; data.len()];
    for r in 0..height {
        let src_r = height - 1 - r;
        let dst = &mut out[r * width..(r + 1) * width];
        let src = &data[src_r * width..(src_r + 1) * width];
        dst.copy_from_slice(src);
    }
    out
}

fn transpose_square(data: &[f32], width: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; data.len()];
    for r in 0..width {
        for c in 0..width {
            out[r * width + c] = data[c * width + r];
        }
    }
    out
}

fn col_major_reinterpret(data: &[f32], width: usize, height: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; data.len()];
    for r in 0..height {
        for c in 0..width {
            out[r * width + c] = data[c * height + r];
        }
    }
    out
}

fn median_i32(values: &[i32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    let mid = v.len() / 2;
    if v.len() % 2 == 1 {
        f64::from(v[mid])
    } else {
        (f64::from(v[mid - 1]) + f64::from(v[mid])) / 2.0
    }
}

fn mean_i32(values: &[i32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: i64 = values.iter().map(|&v| i64::from(v)).sum();
    sum as f64 / values.len() as f64
}

fn variance_i32(values: &[i32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = mean_i32(values);
    let mut acc = 0.0f64;
    for &v in values {
        let dv = f64::from(v) - mean;
        acc += dv * dv;
    }
    acc / values.len() as f64
}

fn mean_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}
