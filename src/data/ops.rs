#![allow(dead_code)]

//! Standalone numeric helpers translated from the SNAPHU leaf utilities.

use crate::constants::{LARGE_SHORT, LARGE_SHORT_I32};

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
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating source length");
    assert_eq!(src.len(), total, "source length mismatch");
    if rows == 0 || cols == 0 {
        return Some(vec![
            T::default();
            (rows + 2 * pad_rows) * (cols + 2 * pad_cols)
        ]);
    }
    if pad_rows >= rows || pad_cols >= cols {
        return None;
    }

    let padded_rows = rows + 2 * pad_rows;
    let padded_cols = cols + 2 * pad_cols;
    let mut dst = vec![T::default(); padded_rows * padded_cols];

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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(mirror_pad(&data, 2, 2, 2, 1).is_none());
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
}
