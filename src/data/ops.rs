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
        let expected = vec![3.0, 4.0, 2.25, 2.25, 2.75, 1.5];
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
}
