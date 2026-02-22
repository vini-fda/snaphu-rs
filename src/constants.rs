#![allow(dead_code)]

//! Numeric constants mirrored from the original SNAPHU macros.

/// Equivalent to the `LARGESHORT` macro used throughout the C sources.
pub const LARGE_SHORT: i16 = 32_000;

/// Clipping bounds for `LARGESHORT` when converted to wider integers.
pub const LARGE_SHORT_I32: i32 = LARGE_SHORT as i32;

/// Used as an effectively "infinite" floating-point value in the original code.
pub const LARGE_FLOAT: f64 = 1.0e35;

/// Maximum number of iterations used by the numerical solvers.
pub const MAX_ITERATIONS: usize = 5_000;

/// Upper bound for the dz-rho lookup table search.
pub const BIGGEST_DZ_RHO_MAX: f64 = 10_000.0;

/// Square root of 1/2, reused when converting between angles.
pub const SQRT_HALF: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// π constant mirrored from the original C globals.
pub const PI: f64 = std::f64::consts::PI;

/// Two-π constant reused when wrapping phase values.
pub const TWO_PI: f64 = std::f64::consts::TAU;

/// Two-π as `f32`, avoiding repeated casts in tight loops.
pub const TWO_PI_F32: f32 = std::f32::consts::TAU;

/// Group value indicating a node whose surrounding pixels are all masked (zero
/// magnitude). Mirrors the C `MASKED` macro (`-5`).
pub const MASKED: i32 = -5;

/// Number of directional arms used by the `Despeckle` filter.
pub const NARMS: usize = 8;

/// Length of each arm (in pixels) for the `Despeckle` filter.
pub const ARMLEN: usize = 5;
