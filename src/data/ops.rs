#![allow(dead_code)]

//! Standalone numeric helpers translated from the SNAPHU leaf utilities.

use crate::constants::{ARMLEN, LARGE_SHORT, LARGE_SHORT_I32, NARMS, PI, TWO_PI};

/// Adds the values of two 2-D arrays (stored in row-major order) elementwise.
///
/// The function mutates `accumulator` in-place and expects both arrays to
/// contain exactly `rows * cols` samples laid out consecutively.
pub fn add_2d_float_arrays(accumulator: &mut [f32], addend: &[f32], rows: usize, cols: usize) {
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating array lengths");
    assert_eq!(accumulator.len(), total, "accumulator length mismatch");
    assert_eq!(addend.len(), total, "addend length mismatch");

    for (dst, src) in accumulator.iter_mut().zip(addend.iter()) {
        *dst += *src;
    }
}

/// Convolves a padded 2-D array with a `krow` x `kcol` boxcar window.
///
/// `padded` must contain the mirror-padded raster laid out in row-major order
/// with dimensions `(rows + krow - 1) x (cols + kcol - 1)`.
pub fn box_car_average(
    output: &mut [f32],
    padded: &[f32],
    rows: usize,
    cols: usize,
    krow: usize,
    kcol: usize,
) {
    assert!(krow > 0 && kcol > 0, "kernel dimensions must be positive");
    if rows == 0 || cols == 0 {
        assert!(
            output.is_empty(),
            "output must be empty for zero-sized grids"
        );
        return;
    }

    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating output length");
    assert_eq!(output.len(), total, "output length mismatch");

    let pad_rows = rows + krow - 1;
    let pad_cols = cols + kcol - 1;
    let expected_pad_len = pad_rows
        .checked_mul(pad_cols)
        .expect("padded dimensions overflow");
    assert_eq!(
        padded.len(),
        expected_pad_len,
        "padded array length mismatch"
    );

    let mut row_offset = 0;
    for row in 0..rows {
        let mut window = 0.0f64;
        for local_r in 0..krow {
            let pad_row = row + local_r;
            let base = pad_row * pad_cols;
            for col in 0..kcol {
                window += padded[base + col] as f64;
            }
        }
        output[row_offset] = window as f32;

        for col in 1..cols {
            for local_r in 0..krow {
                let pad_row = row + local_r;
                let base = pad_row * pad_cols;
                window -= padded[base + col - 1] as f64;
                window += padded[base + col + kcol - 1] as f64;
            }
            output[row_offset + col] = window as f32;
        }

        row_offset += cols;
    }

    let norm = (krow * kcol) as f32;
    for value in output.iter_mut() {
        *value /= norm;
    }
}

/// Returns the clipped average of two `sigsq` values, honoring the sentinel
/// `LARGE_SHORT` value used to encode "no cost" arcs.
pub fn avg_sig_sq(a: i16, b: i16) -> i16 {
    if a == LARGE_SHORT || b == LARGE_SHORT {
        return LARGE_SHORT;
    }

    let sum = a as i32 + b as i32;
    let avg = if sum >= 0 { (sum + 1) / 2 } else { sum / 2 };
    l_clip(avg, -LARGE_SHORT_I32, LARGE_SHORT_I32) as i16
}

/// Clips `value` to the inclusive `[min, max]` range.
pub fn l_clip(value: i32, min: i32, max: i32) -> i32 {
    assert!(min <= max, "invalid clip range");
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Returns `true` if the 2-D array contains only finite samples.
pub fn valid_data_array(data: &[f32], rows: usize, cols: usize) -> bool {
    validate_len(data.len(), rows, cols);
    data.iter().all(|v| v.is_finite())
}

/// Returns `true` if the 2-D array contains only non-negative samples.
pub fn non_neg_data_array(data: &[f32], rows: usize, cols: usize) -> bool {
    validate_len(data.len(), rows, cols);
    data.iter().all(|v| *v >= 0.0)
}

/// Equivalent of the C `LRound` helper (round-to-nearest with ties -> even).
pub fn l_round(value: f64) -> i64 {
    value.round_ties_even() as i64
}

/// Returns the minimum of two `i64` values (`LMin` from the C sources).
pub fn l_min(a: i64, b: i64) -> i64 {
    a.min(b)
}

/// Mirror pads a 2-D array by `pad_rows`/`pad_cols` while reflecting edges.
///
/// Returns `None` if the requested padding would exceed the array dimensions,
/// signaling the caller to keep using the original raster (mirrors the C
/// implementation that returned the original pointer in that case).
pub fn mirror_pad<T>(
    src: &[T],
    rows: usize,
    cols: usize,
    pad_rows: usize,
    pad_cols: usize,
) -> Option<Vec<T>>
where
    T: Copy + Default,
{
    mirror_pad_with_fill(src, rows, cols, pad_rows, pad_cols, T::default())
}

/// Mirror pads an array using an explicit fill value for the padded region.
pub fn mirror_pad_with_fill<T>(
    src: &[T],
    rows: usize,
    cols: usize,
    pad_rows: usize,
    pad_cols: usize,
    fill: T,
) -> Option<Vec<T>>
where
    T: Copy,
{
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating source length");
    assert_eq!(src.len(), total, "source length mismatch");
    if pad_rows > rows || pad_cols > cols {
        return None;
    }

    let padded_rows = rows + 2 * pad_rows;
    let padded_cols = cols + 2 * pad_cols;
    let mut dst = vec![fill; padded_rows * padded_cols];
    if rows == 0 || cols == 0 {
        return Some(dst);
    }

    for row in 0..rows {
        let dst_offset = (row + pad_rows) * padded_cols + pad_cols;
        let src_offset = row * cols;
        dst[dst_offset..dst_offset + cols].copy_from_slice(&src[src_offset..src_offset + cols]);
    }

    if pad_rows == 0 && pad_cols == 0 {
        return Some(dst);
    }

    let pr = pad_rows;
    let pc = pad_cols;
    let pc2 = 2 * pc;
    let pr2 = 2 * pr;

    for row in 0..pr {
        for col in 0..pc {
            let dst_idx = row * padded_cols + col;
            let src_idx = (pr2 - row) * padded_cols + (pc2 - col);
            dst[dst_idx] = dst[src_idx];

            let dst_idx = row * padded_cols + (cols + pc + col);
            let src_idx = (pr2 - row) * padded_cols + (cols + pc - 2 - col);
            dst[dst_idx] = dst[src_idx];

            let dst_idx = (rows + pr + row) * padded_cols + col;
            let src_idx = (rows + pr - 2 - row) * padded_cols + (pc2 - col);
            dst[dst_idx] = dst[src_idx];

            let dst_idx = (rows + pr + row) * padded_cols + (cols + pc + col);
            let src_idx = (rows + pr - 2 - row) * padded_cols + (cols + pc - 2 - col);
            dst[dst_idx] = dst[src_idx];
        }
    }

    for row in pr..(rows + pr) {
        for col in 0..pc {
            let dst_idx = row * padded_cols + col;
            let src_idx = row * padded_cols + (pc2 - col);
            dst[dst_idx] = dst[src_idx];

            let dst_idx = row * padded_cols + (cols + pc + col);
            let src_idx = row * padded_cols + (cols + pc - 2 - col);
            dst[dst_idx] = dst[src_idx];
        }
    }

    for col in pc..(cols + pc) {
        for row in 0..pr {
            let dst_idx = row * padded_cols + col;
            let src_idx = (pr2 - row) * padded_cols + col;
            dst[dst_idx] = dst[src_idx];

            let dst_idx = (rows + pr + row) * padded_cols + col;
            let src_idx = (rows + pr - 2 - row) * padded_cols + col;
            dst[dst_idx] = dst[src_idx];
        }
    }

    Some(dst)
}

fn validate_len(actual: usize, rows: usize, cols: usize) {
    let expected = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating array length");
    assert_eq!(actual, expected, "array length mismatch");
}

/// 1-D linear interpolation that clamps to the array bounds.
pub fn lin_interp_1d(arr: &[f32], index: f64) -> f32 {
    assert!(!arr.is_empty(), "interpolation array must not be empty");
    let int_part = index.floor();
    if int_part < 0.0 {
        return arr[0];
    }
    let upper_bound = (arr.len() - 1) as f64;
    if int_part >= upper_bound {
        return arr[arr.len() - 1];
    }

    let base = int_part as usize;
    let frac = (index - int_part) as f32;
    let weight0 = 1.0_f32 - frac;
    (weight0 * arr[base] + frac * arr[base + 1]) / 2.0
}

/// 2-D linear interpolation using row-major storage.
pub fn lin_interp_2d(arr: &[f32], rows: usize, cols: usize, row_idx: f64, col_idx: f64) -> f32 {
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating array length");
    assert_eq!(arr.len(), total, "array length mismatch");
    assert!(rows >= 1 && cols >= 1, "interpolation array must have data");

    let get_row = |row: usize| -> f32 {
        let offset = row * cols;
        lin_interp_1d(&arr[offset..offset + cols], col_idx)
    };

    let row_floor = row_idx.floor();
    if row_floor < 0.0 {
        return get_row(0);
    }
    let max_row = (rows - 1) as f64;
    if row_floor >= max_row {
        return get_row(rows - 1);
    }

    let base = row_floor as usize;
    let frac = (row_idx - row_floor) as f32;
    let lower = get_row(base);
    let upper = get_row(base + 1);
    ((1.0_f32 - frac) * lower + frac * upper) / 2.0
}

/// Subtracts the estimated unwrapped phase from the wrapped field and re-wraps
/// each sample into the `[0, 2π)` range.
pub fn flatten_wrapped_phase(
    wrapped_phase: &mut [f32],
    unwrapped_estimate: &[f32],
    rows: usize,
    cols: usize,
) {
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating array lengths");
    assert_eq!(wrapped_phase.len(), total, "wrapped phase length mismatch");
    assert_eq!(
        unwrapped_estimate.len(),
        total,
        "unwrapped estimate length mismatch",
    );

    for (wrapped, estimate) in wrapped_phase.iter_mut().zip(unwrapped_estimate.iter()) {
        let flattened = (*wrapped as f64 - *estimate as f64).rem_euclid(TWO_PI);
        *wrapped = flattened as f32;
    }
}

/// Wraps every sample back into the `[0, 2π)` interval.
pub fn wrap_phase(field: &mut [f32], rows: usize, cols: usize) {
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating array length");
    assert_eq!(field.len(), total, "phase field length mismatch");
    for value in field.iter_mut() {
        *value = (*value as f64).rem_euclid(TWO_PI) as f32;
    }
}

/// Computes the difference between `f1` and `f2` wrapped into `(-π, π]`.
pub fn mod_diff(f1: f64, f2: f64) -> f64 {
    let mut diff = f1 - f2;
    if diff > PI {
        diff -= TWO_PI;
    } else if diff <= -PI {
        diff += TWO_PI;
    }
    diff
}

/// Fills every element of a 2-D `i16` array with `value`.
///
/// Equivalent of the C `Set2DShortArray`.
pub fn set_2d_short_array(arr: &mut [i16], rows: usize, cols: usize, value: i16) {
    validate_len(arr.len(), rows, cols);
    arr.fill(value);
}

/// Returns the maximum absolute value across a "row-col" flow/cost array.
///
/// The array uses SNAPHU's standard grid-network layout:
///   - rows `[0, nrow-1)`: row arcs, each with `ncol` entries
///   - rows `[nrow-1, 2*nrow-1)`: col arcs, each with `ncol-1` entries
///
/// Equivalent of the C `Short2DRowColAbsMax`.
pub fn short_2d_row_col_abs_max(arr: &[i16], nrow: usize, ncol: usize) -> i64 {
    assert!(nrow >= 1 && ncol >= 1, "dimensions must be at least 1");
    let row_arc_count = (nrow - 1) * ncol;
    let col_arc_count = nrow * (ncol - 1);
    assert_eq!(
        arr.len(),
        row_arc_count + col_arc_count,
        "array length does not match row-col layout"
    );

    let mut max_val: i64 = 0;

    // Row arcs: first (nrow-1) rows, ncol columns each
    for &v in &arr[..row_arc_count] {
        let abs = (v as i64).abs();
        if abs > max_val {
            max_val = abs;
        }
    }

    // Col arcs: next nrow rows, (ncol-1) columns each
    for &v in &arr[row_arc_count..] {
        let abs = (v as i64).abs();
        if abs > max_val {
            max_val = abs;
        }
    }

    max_val
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveMeanError {
    KernelTooLarge,
}

/// Divides intensity by a local sliding-window average.
///
/// This is the idiomatic Rust equivalent of the C `RemoveMean()` helper.
/// Even kernel dimensions are promoted to the next odd size, then mirror
/// padding + boxcar averaging are used to compute local means.
pub fn remove_mean(
    ei: &mut [f32],
    nrow: usize,
    ncol: usize,
    mut krowei: usize,
    mut kcolei: usize,
) -> Result<(), RemoveMeanError> {
    validate_len(ei.len(), nrow, ncol);

    if krowei % 2 == 0 {
        krowei += 1;
    }
    if kcolei % 2 == 0 {
        kcolei += 1;
    }

    let pad_rows = (krowei - 1) / 2;
    let pad_cols = (kcolei - 1) / 2;
    let Some(padded) = mirror_pad(ei, nrow, ncol, pad_rows, pad_cols) else {
        return Err(RemoveMeanError::KernelTooLarge);
    };

    let mut avg = vec![0.0f32; nrow * ncol];
    box_car_average(&mut avg, &padded, nrow, ncol, krowei, kcolei);
    for (value, local_avg) in ei.iter_mut().zip(avg.iter()) {
        *value /= *local_avg;
    }
    Ok(())
}

/// Compute wrapped residue for one grid node.
///
/// Equivalent to the C `NodeResidue()` function. The residue is built from
/// the wrapped phase differences around the 2x2 plaquette whose top-left
/// pixel is `(row, col)`.
pub fn node_residue(wphase: &[f32], nrow: usize, ncol: usize, row: usize, col: usize) -> i32 {
    validate_len(wphase.len(), nrow, ncol);
    assert!(row + 1 < nrow, "row out of bounds for node residue");
    assert!(col + 1 < ncol, "col out of bounds for node residue");
    let idx = |r: usize, c: usize| r * ncol + c;
    let residue = (mod_diff(
        wphase[idx(row, col + 1)] as f64,
        wphase[idx(row, col)] as f64,
    ) + mod_diff(
        wphase[idx(row + 1, col + 1)] as f64,
        wphase[idx(row, col + 1)] as f64,
    ) + mod_diff(
        wphase[idx(row + 1, col)] as f64,
        wphase[idx(row + 1, col + 1)] as f64,
    ) + mod_diff(
        wphase[idx(row, col)] as f64,
        wphase[idx(row + 1, col)] as f64,
    )) / TWO_PI;
    l_round(residue) as i32
}

/// Calculate row/column flow arrays from an unwrapped phase raster.
///
/// Equivalent to the C `CalcFlow()` function. Output follows SNAPHU's
/// row-col layout:
/// - first `(nrow-1) * ncol` values are row arcs
/// - then `nrow * (ncol-1)` values are col arcs
pub fn calc_flow(phase: &[f32], nrow: usize, ncol: usize) -> Vec<i16> {
    validate_len(phase.len(), nrow, ncol);
    let row_arc_count = (nrow - 1) * ncol;
    let col_arc_count = nrow * (ncol - 1);
    let mut flows = vec![0i16; row_arc_count + col_arc_count];
    let idx = |r: usize, c: usize| r * ncol + c;

    for row in 0..(nrow - 1) {
        for col in 0..ncol {
            let flow =
                l_round((phase[idx(row, col)] as f64 - phase[idx(row + 1, col)] as f64) / TWO_PI);
            flows[row * ncol + col] = i16::try_from(flow).expect("row flow exceeds i16 range");
        }
    }

    let col_base = row_arc_count;
    for row in 0..nrow {
        for col in 0..(ncol - 1) {
            let flow =
                l_round((phase[idx(row, col + 1)] as f64 - phase[idx(row, col)] as f64) / TWO_PI);
            flows[col_base + row * (ncol - 1) + col] =
                i16::try_from(flow).expect("col flow exceeds i16 range");
        }
    }

    flows
}

/// Adaptive geometric (directional) despeckle filter for magnitude data.
///
/// Filters using 8 directional arms of length `ARMLEN` around each pixel.
/// For each nonzero pixel the arm pair with the highest anisotropy ratio
/// is selected, preserving linear features while smoothing noise. Zero
/// pixels are preserved as-is (to honour mask information in the input).
///
/// Returns the filtered output as a new flat row-major `Vec<f32>`.
///
/// Equivalent of the C `Despeckle`.
pub fn despeckle(mag: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    validate_len(mag.len(), rows, cols);

    let mut output = vec![0.0f32; rows * cols];

    // Mirror-pad the magnitude raster by ARMLEN in both directions.
    let padded = mirror_pad(mag, rows, cols, ARMLEN, ARMLEN)
        .expect("Despeckling box size too large for input array size");
    let padded_cols = cols + 2 * ARMLEN;

    // Diagonal arm geometry tables (from the C source).
    let jmin: [usize; 5] = [2, 2, 0, 1, 2];
    let jmax: [usize; 5] = [2, 3, 4, 3, 2];

    // Arm indices: C=0, T=1, B=2, R=3, L=4, TR=5, BL=6, TL=7, BR=8
    const C: usize = 0;
    const T: usize = 1;
    const B: usize = 2;
    const R: usize = 3;
    const L: usize = 4;
    const TR: usize = 5;
    const BL: usize = 6;
    const TL: usize = 7;
    const BR: usize = 8;

    for row in 0..rows {
        let i_row = row + ARMLEN;
        for col in 0..cols {
            let i_col = col + ARMLEN;

            // Preserve zeros (mask info).
            if padded[i_row * padded_cols + i_col] == 0.0 {
                output[row * cols + col] = 0.0;
                continue;
            }

            let mut w = [0.0f64; NARMS + 1];

            // Center 3×3 block
            for di in -1i32..=1 {
                for dj in -1i32..=1 {
                    let r = (i_row as i32 + di) as usize;
                    let c = (i_col as i32 + dj) as usize;
                    w[C] += padded[r * padded_cols + c] as f64;
                }
            }

            // Four straight arms (T, B, L, R)
            for di in -1i32..=1 {
                for j in 2..(ARMLEN as i32 + 1) {
                    let r = i_row as i32;
                    let c = i_col as i32;
                    w[T] += padded[((r - j) as usize) * padded_cols + (c + di) as usize] as f64;
                    w[B] += padded[((r + j) as usize) * padded_cols + (c + di) as usize] as f64;
                    w[L] += padded[((r + di) as usize) * padded_cols + (c - j) as usize] as f64;
                    w[R] += padded[((r + di) as usize) * padded_cols + (c + j) as usize] as f64;
                }
            }

            // Four diagonal arms (TR, BR, BL, TL)
            for i in 0..5usize {
                for j in jmin[i]..=jmax[i] {
                    let r = i_row;
                    let c = i_col;
                    w[TR] += padded[(r - i) * padded_cols + (c + j)] as f64;
                    w[BR] += padded[(r + i) * padded_cols + (c + j)] as f64;
                    w[BL] += padded[(r + i) * padded_cols + (c - j)] as f64;
                    w[TL] += padded[(r - i) * padded_cols + (c - j)] as f64;
                }
            }

            // Full diamond weight
            let mut wfull = w[C] + w[T] + w[R] + w[B] + w[L];
            for i in 2i32..5 {
                for j in 2i32..(7 - i) {
                    let r = i_row as i32;
                    let c = i_col as i32;
                    wfull += padded[(r + i) as usize * padded_cols + (c + j) as usize] as f64;
                    wfull += padded[(r - i) as usize * padded_cols + (c + j) as usize] as f64;
                    wfull += padded[(r + i) as usize * padded_cols + (c - j) as usize] as f64;
                    wfull += padded[(r - i) as usize * padded_cols + (c - j) as usize] as f64;
                }
            }

            // Select arm pair with highest anisotropy ratio
            let mut ratio_max = 1.0f64;
            // k iterates over opposite arm pairs: (T,B), (L,R), (TR,BL), (TL,BR)
            let mut k = 1;
            while k <= NARMS {
                let wstick = w[C] + w[k] + w[k + 1];
                let complement = wfull - wstick;
                if complement > 0.0 {
                    let mut ratio = wstick / complement;
                    if ratio < 1.0 {
                        ratio = 1.0 / ratio;
                    }
                    if ratio > ratio_max {
                        ratio_max = ratio;
                        output[row * cols + col] = wstick as f32;
                    }
                }
                k += 2;
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::TWO_PI_F32;

    #[test]
    fn add_2d_arrays_adds_elementwise() {
        let mut lhs = vec![1.0, 2.0, 3.0, -1.0, 0.0, 5.0];
        let rhs = vec![0.5, -1.0, 2.0, 3.5, 4.0, -2.5];
        add_2d_float_arrays(&mut lhs, &rhs, 2, 3);
        assert_eq!(lhs, vec![1.5, 1.0, 5.0, 2.5, 4.0, 2.5]);
    }

    #[test]
    fn box_car_average_matches_reference_window() {
        // 2x3 array, kernel 2x2, padded to 3x4.
        let padded = vec![1.0, 2.0, 3.0, 0.0, 4.0, 5.0, 6.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut output = vec![0.0; 6];
        box_car_average(&mut output, &padded, 2, 3, 2, 2);
        let expected = [3.0, 4.0, 2.25, 2.25, 2.75, 1.5];
        for (got, want) in output.iter().zip(expected.iter()) {
            assert!((got - want).abs() < 1e-5, "{} vs {}", got, want);
        }
    }

    #[test]
    fn avg_sig_sq_clips_and_handles_sentinels() {
        assert_eq!(avg_sig_sq(100, 102), 101);
        assert_eq!(avg_sig_sq(-5, -6), -5);
        assert_eq!(avg_sig_sq(LARGE_SHORT, 5), LARGE_SHORT);
        assert_eq!(
            avg_sig_sq(LARGE_SHORT - 1, LARGE_SHORT - 1),
            LARGE_SHORT - 1
        );
        // Force clipping beyond the sentinel range.
        assert_eq!(avg_sig_sq(i16::MAX, i16::MAX), LARGE_SHORT);
    }

    #[test]
    fn l_clip_bounds_values() {
        assert_eq!(l_clip(5, 0, 10), 5);
        assert_eq!(l_clip(-1, 0, 10), 0);
        assert_eq!(l_clip(15, 0, 10), 10);
    }

    #[test]
    fn valid_data_array_checks_finiteness() {
        let data = vec![0.0f32, 1.0, 2.0, -3.5];
        assert!(valid_data_array(&data, 2, 2));
        let mut data = data;
        data[2] = f32::INFINITY;
        assert!(!valid_data_array(&data, 2, 2));
    }

    #[test]
    fn non_neg_data_array_checks_sign() {
        let data = vec![0.0f32, 1.0, 2.0, 3.0];
        assert!(non_neg_data_array(&data, 2, 2));
        let mut data = data;
        data[3] = -0.1;
        assert!(!non_neg_data_array(&data, 2, 2));
    }

    #[test]
    fn l_round_matches_rint_semantics() {
        assert_eq!(l_round(1.2), 1);
        assert_eq!(l_round(1.8), 2);
        assert_eq!(l_round(-1.2), -1);
        assert_eq!(l_round(-1.8), -2);
        // Halfway cases round to even, mirroring C's `rint` helper.
        assert_eq!(l_round(1.5), 2);
        assert_eq!(l_round(2.5), 2);
        assert_eq!(l_round(-1.5), -2);
        assert_eq!(l_round(-2.5), -2);
    }

    #[test]
    fn l_min_returns_smallest_value() {
        assert_eq!(l_min(5, 10), 5);
        assert_eq!(l_min(-3, -7), -7);
        assert_eq!(l_min(0, 0), 0);
    }

    #[test]
    fn mirror_pad_reflects_edges() {
        let data: Vec<i32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let rows = 3;
        let cols = 4;
        let pad_rows = 1;
        let pad_cols = 1;
        let padded_cols = cols + 2 * pad_cols;
        let padded = mirror_pad(&data, rows, cols, pad_rows, pad_cols).unwrap();

        // Center block matches original data.
        for r in 0..rows {
            for c in 0..cols {
                let idx = (r + pad_rows) * padded_cols + (c + pad_cols);
                assert_eq!(padded[idx], data[r * cols + c]);
            }
        }

        // Sample a few mirrored positions, matching the original index math.
        assert_eq!(padded[0], data[cols + 1]);
        assert_eq!(padded[5], data[cols + 2]);
        assert_eq!(padded[4 * padded_cols], data[cols + 1]);
        assert_eq!(padded[4 * padded_cols + 5], data[cols + 2]);
        assert_eq!(padded[padded_cols], data[1]);
        assert_eq!(padded[padded_cols + 5], data[2]);
    }

    #[test]
    fn mirror_pad_rejects_large_padding() {
        let data = vec![0.0f32; 4];
        assert!(mirror_pad(&data, 2, 2, 3, 1).is_none());
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Pixel(u8);

    #[test]
    fn mirror_pad_with_fill_supports_non_default_types() {
        let src = vec![Pixel(1), Pixel(2), Pixel(3), Pixel(4)];
        let padded = mirror_pad_with_fill(&src, 2, 2, 1, 1, Pixel(0)).unwrap();
        let padded_cols = 4;
        assert_eq!(padded.len(), 16);
        assert_eq!(padded[padded_cols + 1], Pixel(1));
        assert_eq!(padded[padded_cols + 2], Pixel(2));
        assert_eq!(padded[2 * padded_cols + 1], Pixel(3));
        assert_eq!(padded[2 * padded_cols + 2], Pixel(4));
        // Corners mirror the interior values.
        assert_eq!(padded[0], Pixel(4));
        assert_eq!(padded[3], Pixel(3));
        assert_eq!(padded[12], Pixel(2));
        assert_eq!(padded[15], Pixel(1));
    }

    #[test]
    fn lin_interp_1d_matches_reference() {
        let arr = [0.0, 10.0, 20.0, 30.0];
        assert_eq!(lin_interp_1d(&arr, -1.0), 0.0);
        assert_eq!(lin_interp_1d(&arr, 10.0), 30.0);
        let mid = lin_interp_1d(&arr, 1.5);
        assert!((mid - 7.5).abs() < 1e-6);
    }

    #[test]
    fn lin_interp_2d_matches_reference() {
        let arr: Vec<f32> = vec![0.0, 1.0, 2.0, 10.0, 11.0, 12.0, 20.0, 21.0, 22.0];
        let val = lin_interp_2d(&arr, 3, 3, 0.5, 1.5);
        // Expect averaging both dimensions (with /2 scaling from legacy code).
        let expected_row0 = lin_interp_1d(&arr[0..3], 1.5);
        let expected = ((1.0 - 0.5) * expected_row0 + 0.5 * lin_interp_1d(&arr[3..6], 1.5)) / 2.0;
        assert!((val - expected).abs() < 1e-6);
        assert_eq!(lin_interp_2d(&arr, 3, 3, -1.0, 0.0), 0.0);
        assert_eq!(lin_interp_2d(&arr, 3, 3, 10.0, 2.0), 22.0);
    }

    #[test]
    fn flatten_wrapped_phase_rewraps_relative_to_estimate() {
        let mut wrapped = vec![0.0f32, 3.0, -1.0, 9.0];
        let unwrapped = vec![1.0f32, 1.5, -5.0, 4.0];
        flatten_wrapped_phase(&mut wrapped, &unwrapped, 2, 2);
        let expected = [
            TWO_PI_F32 - 1.0,
            1.5,
            4.0,
            (9.0f32 - 4.0f32).rem_euclid(TWO_PI_F32),
        ];
        for (got, want) in wrapped.iter().zip(expected.iter()) {
            let diff = (*got - *want).abs();
            assert!(diff < 1e-5, "{} vs {}", got, want);
        }
    }

    #[test]
    fn wrap_phase_normalizes_values() {
        let mut field = vec![-3.0 * std::f32::consts::PI, 0.0, 5.0 * std::f32::consts::PI];
        wrap_phase(&mut field, 3, 1);
        assert!(field[0] >= 0.0 && field[0] < TWO_PI_F32);
        assert_eq!(field[1], 0.0);
        assert!((field[2] - std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn mod_diff_matches_expected_range() {
        let diff = mod_diff(3.0 * std::f64::consts::PI, 0.0);
        assert!((diff - std::f64::consts::PI).abs() < 1e-12);
        let diff = mod_diff(-3.5 * std::f64::consts::PI, 0.0);
        assert!((diff + 1.5 * std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn set_2d_short_array_fills_all_elements() {
        let mut arr = vec![0i16; 12];
        set_2d_short_array(&mut arr, 3, 4, 42);
        assert!(arr.iter().all(|&v| v == 42));
    }

    #[test]
    fn set_2d_short_array_handles_negative_fill() {
        let mut arr = vec![1i16; 6];
        set_2d_short_array(&mut arr, 2, 3, -7);
        assert!(arr.iter().all(|&v| v == -7));
    }

    #[test]
    fn short_2d_row_col_abs_max_finds_maximum() {
        // 3 rows, 3 cols → row arcs: 2*3=6, col arcs: 3*2=6, total=12
        let mut arr = vec![0i16; 12];
        arr[3] = -50; // row arc region
        arr[9] = 42; // col arc region
        assert_eq!(short_2d_row_col_abs_max(&arr, 3, 3), 50);
    }

    #[test]
    fn short_2d_row_col_abs_max_all_zeros() {
        // 2 rows, 3 cols → row arcs: 1*3=3, col arcs: 2*2=4, total=7
        let arr = vec![0i16; 7];
        assert_eq!(short_2d_row_col_abs_max(&arr, 2, 3), 0);
    }

    #[test]
    fn despeckle_preserves_zeros() {
        // A raster of all zeros should produce all zeros.
        let mag = vec![0.0f32; 36];
        let result = despeckle(&mag, 6, 6);
        assert!(result.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn despeckle_filters_nonzero_pixels() {
        // A uniform nonzero raster should produce nonzero outputs.
        let rows = 12;
        let cols = 12;
        let mag = vec![1.0f32; rows * cols];
        let result = despeckle(&mag, rows, cols);
        // All interior pixels should get a nonzero filtered value.
        for row in 1..rows - 1 {
            for col in 1..cols - 1 {
                assert!(
                    result[row * cols + col] > 0.0,
                    "expected nonzero at ({}, {})",
                    row,
                    col
                );
            }
        }
    }

    #[test]
    fn remove_mean_divides_by_local_average() {
        let mut ei = vec![2.0f32, 4.0, 6.0, 8.0];
        remove_mean(&mut ei, 2, 2, 3, 3).unwrap();
        assert!(ei.iter().all(|v| v.is_finite()));
        // All entries are scaled, not left untouched.
        assert_ne!(ei, vec![2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn remove_mean_rejects_oversized_kernel() {
        let mut ei = vec![1.0f32; 4];
        let err = remove_mean(&mut ei, 2, 2, 9, 3).unwrap_err();
        assert_eq!(err, RemoveMeanError::KernelTooLarge);
    }

    #[test]
    fn node_residue_of_constant_phase_is_zero() {
        let phase = vec![1.0f32; 9]; // 3x3
        assert_eq!(node_residue(&phase, 3, 3, 0, 0), 0);
        assert_eq!(node_residue(&phase, 3, 3, 1, 1), 0);
    }

    #[test]
    fn calc_flow_zero_for_constant_phase() {
        let phase = vec![5.0f32; 12]; // 3x4
        let flow = calc_flow(&phase, 3, 4);
        assert!(flow.iter().all(|&v| v == 0));
    }

    #[test]
    fn calc_flow_matches_simple_gradient() {
        // 2x3 field:
        // [0, 2π, 4π]
        // [2π, 4π, 6π]
        // Row flows: [ -1, -1, -1 ]
        // Col flows: [ 1, 1, 1, 1 ]
        let p = std::f32::consts::TAU;
        let phase = vec![0.0, p, 2.0 * p, p, 2.0 * p, 3.0 * p];
        let flow = calc_flow(&phase, 2, 3);
        assert_eq!(flow, vec![-1, -1, -1, 1, 1, 1, 1]);
    }
}
