//! Module to create human-understandable colormaps for test data visualization with plots
use std::f32::consts::TAU;

/// Cubehelix cycle colormap.
///
/// The input is assumed to be unwrapped phase in radians, so any valid fp32 value.
///
/// The output is an array with the [red, green, blue] channels from 0 to 255 as bytes (u8).
///
/// Based on the BEAM Color Palette Definition
pub fn cubehelix_colormap_unwrapped(uw_phase: f32) -> [u8; 3] {
    let remainder = uw_phase.rem_euclid(TAU);
    let normalized_phase = remainder / TAU;
    cubehelix_colormap_f32(normalized_phase).map(to_byte)
}

/// Scales the floating point normalized phase value (assumed to be in the range [0, 1])
/// to the range [0, 255] then convert to a byte.
///
/// Tho avoid underflow or overflow, this function clamps the value.
#[inline(always)]
fn to_byte(normalized_phase: f32) -> u8 {
    (255.0 * normalized_phase).clamp(0.0, 255.0) as u8
}

/// Cubehelix cycle colormap in normalized [0, 1] RGB.
fn cubehelix_colormap_f32(x: f32) -> [f32; 3] {
    // Color points from the palette (normalized to [0,1])
    let colors: [[f32; 3]; 8] = [
        [110.0 / 255.0, 60.0 / 255.0, 170.0 / 255.0], // color0
        [210.0 / 255.0, 60.0 / 255.0, 160.0 / 255.0], // color1
        [1.0, 110.0 / 255.0, 70.0 / 255.0],           // color2
        [200.0 / 255.0, 200.0 / 255.0, 50.0 / 255.0], // color3
        [80.0 / 255.0, 245.0 / 255.0, 100.0 / 255.0], // color4
        [25.0 / 255.0, 200.0 / 255.0, 180.0 / 255.0], // color5
        [60.0 / 255.0, 130.0 / 255.0, 220.0 / 255.0], // color6
        [100.0 / 255.0, 70.0 / 255.0, 190.0 / 255.0], // color7
    ];

    // Wrap input to [0, 1]
    let x = x.rem_euclid(1.0);

    // Scale x to [0, 7] to match our 8 color points
    let x_scaled = x * 7.0;

    // Get the two colors to interpolate between
    let idx = x_scaled.floor() as usize;
    let t = x_scaled - idx as f32;

    // Handle the wrap-around case
    let (c1, c2) = if idx == 7 {
        (colors[7], colors[0])
    } else {
        (colors[idx], colors[idx + 1])
    };

    // Cubic interpolation
    let t2 = t * t;
    let t3 = t2 * t;

    // Cubic interpolation coefficients
    let a = -0.5 * c1[0] + 1.5 * c2[0] - 1.5 * c1[0] + 0.5 * c2[0];
    let b = c1[0] - 2.5 * c2[0] + 2.0 * c1[0] - 0.5 * c2[0];
    let c = -0.5 * c1[0] + 0.5 * c2[0];
    let d = c1[0];

    let red = a * t3 + b * t2 + c * t + d;

    let a = -0.5 * c1[1] + 1.5 * c2[1] - 1.5 * c1[1] + 0.5 * c2[1];
    let b = c1[1] - 2.5 * c2[1] + 2.0 * c1[1] - 0.5 * c2[1];
    let c = -0.5 * c1[1] + 0.5 * c2[1];
    let d = c1[1];

    let green = a * t3 + b * t2 + c * t + d;

    let a = -0.5 * c1[2] + 1.5 * c2[2] - 1.5 * c1[2] + 0.5 * c2[2];
    let b = c1[2] - 2.5 * c2[2] + 2.0 * c1[2] - 0.5 * c2[2];
    let c = -0.5 * c1[2] + 0.5 * c2[2];
    let d = c1[2];

    let blue = a * t3 + b * t2 + c * t + d;

    [red, green, blue]
}
