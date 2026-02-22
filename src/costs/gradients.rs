#![allow(dead_code)]

//! Wrapped-phase gradient builders used by the statistical cost terms.

use crate::constants::TWO_PI;
use crate::data::ops::{box_car_average, mirror_pad};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientDirection {
    Range,
    Azimuth,
}

#[derive(Debug, Clone)]
pub struct WrappedGradientField {
    direction: GradientDirection,
    rows: usize,
    cols: usize,
    gradients: Vec<f32>,
    averages: Vec<f32>,
}

impl WrappedGradientField {
    pub fn direction(&self) -> GradientDirection {
        self.direction
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn gradients(&self) -> &[f32] {
        &self.gradients
    }

    pub fn averages(&self) -> &[f32] {
        &self.averages
    }

    fn empty(direction: GradientDirection, rows: usize, cols: usize) -> Self {
        Self {
            direction,
            rows,
            cols,
            gradients: Vec::new(),
            averages: Vec::new(),
        }
    }

    fn new(
        direction: GradientDirection,
        rows: usize,
        cols: usize,
        gradients: Vec<f32>,
        averages: Vec<f32>,
    ) -> Self {
        Self {
            direction,
            rows,
            cols,
            gradients,
            averages,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrappedGradientError {
    LengthMismatch {
        expected: usize,
        actual: usize,
    },
    InvalidKernel {
        kernel_rows: usize,
        kernel_cols: usize,
    },
    KernelTooLarge {
        rows: usize,
        cols: usize,
        kernel_rows: usize,
        kernel_cols: usize,
    },
}

impl fmt::Display for WrappedGradientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrappedGradientError::LengthMismatch { expected, actual } => {
                write!(
                    f,
                    "gradient input length mismatch: expected {expected}, got {actual}"
                )
            }
            WrappedGradientError::InvalidKernel {
                kernel_rows,
                kernel_cols,
            } => write!(
                f,
                "kernel dimensions must be positive odd numbers (got {kernel_rows}x{kernel_cols})",
            ),
            WrappedGradientError::KernelTooLarge {
                rows,
                cols,
                kernel_rows,
                kernel_cols,
            } => write!(
                f,
                "kernel {kernel_rows}x{kernel_cols} too large for gradient grid {rows}x{cols}",
            ),
        }
    }
}

impl Error for WrappedGradientError {}

pub fn compute_range_gradients(
    wrapped_phase: &[f32],
    rows: usize,
    cols: usize,
    kernel_perpendicular: usize,
    kernel_parallel: usize,
) -> Result<WrappedGradientField, WrappedGradientError> {
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating wrapped-phase length");
    if wrapped_phase.len() != total {
        return Err(WrappedGradientError::LengthMismatch {
            expected: total,
            actual: wrapped_phase.len(),
        });
    }

    let grad_rows = rows;
    let grad_cols = cols.saturating_sub(1);
    if grad_rows == 0 || grad_cols == 0 {
        return Ok(WrappedGradientField::empty(
            GradientDirection::Range,
            grad_rows,
            grad_cols,
        ));
    }

    validate_kernel(kernel_perpendicular, kernel_parallel)?;

    let mut gradients = vec![0.0f32; grad_rows * grad_cols];
    for row in 0..rows {
        let src_row = row * cols;
        let dst_row = row * grad_cols;
        for col in 0..grad_cols {
            let diff = (wrapped_phase[src_row + col + 1] as f64
                - wrapped_phase[src_row + col] as f64)
                / TWO_PI;
            gradients[dst_row + col] = wrap_to_short_cycle(diff);
        }
    }

    let averages = average_gradients(
        &gradients,
        grad_rows,
        grad_cols,
        kernel_perpendicular,
        kernel_parallel,
    )?;

    Ok(WrappedGradientField::new(
        GradientDirection::Range,
        grad_rows,
        grad_cols,
        gradients,
        averages,
    ))
}

pub fn compute_azimuth_gradients(
    wrapped_phase: &[f32],
    rows: usize,
    cols: usize,
    kernel_parallel: usize,
    kernel_perpendicular: usize,
) -> Result<WrappedGradientField, WrappedGradientError> {
    let total = rows
        .checked_mul(cols)
        .expect("rows*cols would overflow when validating wrapped-phase length");
    if wrapped_phase.len() != total {
        return Err(WrappedGradientError::LengthMismatch {
            expected: total,
            actual: wrapped_phase.len(),
        });
    }

    let grad_rows = rows.saturating_sub(1);
    let grad_cols = cols;
    if grad_rows == 0 || grad_cols == 0 {
        return Ok(WrappedGradientField::empty(
            GradientDirection::Azimuth,
            grad_rows,
            grad_cols,
        ));
    }

    validate_kernel(kernel_parallel, kernel_perpendicular)?;

    let mut gradients = vec![0.0f32; grad_rows * grad_cols];
    for row in 0..grad_rows {
        let top = row * cols;
        let bottom = top + cols;
        let dst_row = row * grad_cols;
        for col in 0..grad_cols {
            let diff =
                (wrapped_phase[top + col] as f64 - wrapped_phase[bottom + col] as f64) / TWO_PI;
            gradients[dst_row + col] = wrap_to_short_cycle(diff);
        }
    }

    let averages = average_gradients(
        &gradients,
        grad_rows,
        grad_cols,
        kernel_parallel,
        kernel_perpendicular,
    )?;

    Ok(WrappedGradientField::new(
        GradientDirection::Azimuth,
        grad_rows,
        grad_cols,
        gradients,
        averages,
    ))
}

fn validate_kernel(kernel_rows: usize, kernel_cols: usize) -> Result<(), WrappedGradientError> {
    if kernel_rows == 0
        || kernel_cols == 0
        || kernel_rows.is_multiple_of(2)
        || kernel_cols.is_multiple_of(2)
    {
        return Err(WrappedGradientError::InvalidKernel {
            kernel_rows,
            kernel_cols,
        });
    }
    Ok(())
}

fn average_gradients(
    gradients: &[f32],
    rows: usize,
    cols: usize,
    kernel_rows: usize,
    kernel_cols: usize,
) -> Result<Vec<f32>, WrappedGradientError> {
    if gradients.is_empty() {
        return Ok(Vec::new());
    }
    let pad_rows = (kernel_rows - 1) / 2;
    let pad_cols = (kernel_cols - 1) / 2;
    let padded = mirror_pad(gradients, rows, cols, pad_rows, pad_cols).ok_or(
        WrappedGradientError::KernelTooLarge {
            rows,
            cols,
            kernel_rows,
            kernel_cols,
        },
    )?;
    let mut averages = vec![0.0f32; gradients.len()];
    box_car_average(&mut averages, &padded, rows, cols, kernel_rows, kernel_cols);
    Ok(averages)
}

fn wrap_to_short_cycle(mut value: f64) -> f32 {
    if value >= 0.5 {
        value -= 1.0;
    } else if value < -0.5 {
        value += 1.0;
    }
    value as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI as PI_F32;

    #[test]
    fn range_gradients_match_expected_cycles() {
        let wrapped = vec![0.0, PI_F32, -PI_F32, PI_F32, 0.0, PI_F32];
        let grads = compute_range_gradients(&wrapped, 2, 3, 3, 3).unwrap();
        assert_eq!(grads.direction(), GradientDirection::Range);
        assert_eq!(grads.rows(), 2);
        assert_eq!(grads.cols(), 2);
        let got = grads.gradients();
        assert!((got[0] + 0.5).abs() < 1e-6);
        assert!(got[1].abs() < 1e-6);
        assert!((got[2] - 0.5).abs() < 1e-6);
        assert!((got[3] + 0.5).abs() < 1e-6);
    }

    #[test]
    fn azimuth_gradients_match_expected_cycles() {
        let wrapped = vec![0.0, PI_F32, 0.0, -PI_F32];
        let grads = compute_azimuth_gradients(&wrapped, 2, 2, 3, 3).unwrap();
        assert_eq!(grads.direction(), GradientDirection::Azimuth);
        assert_eq!(grads.rows(), 1);
        assert_eq!(grads.cols(), 2);
        assert_eq!(grads.gradients().len(), 2);
    }

    #[test]
    fn detects_invalid_kernel() {
        let wrapped = vec![0.0, PI_F32, 0.0, -PI_F32];
        let err = compute_range_gradients(&wrapped, 2, 2, 2, 3).unwrap_err();
        assert!(matches!(err, WrappedGradientError::InvalidKernel { .. }));
    }

    #[test]
    fn detects_kernel_too_large() {
        let wrapped = vec![0.0, PI_F32, 0.0, -PI_F32];
        let err = compute_range_gradients(&wrapped, 2, 2, 3, 5).unwrap_err();
        assert!(matches!(err, WrappedGradientError::KernelTooLarge { .. }));
    }
}
