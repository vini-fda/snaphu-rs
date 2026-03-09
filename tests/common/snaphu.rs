//! Utilities for SNAPHU C vs snaphu-rs integration tests.
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process::Command;

pub fn is_snaphu_available() -> bool {
    Command::new("snaphu").arg("--help").output().is_ok()
}

/// Wraps phase to the [-pi, pi) interval.
pub fn wrap_to_pi(phase: f32) -> f32 {
    ((phase + PI).rem_euclid(TAU)) - PI
}

pub fn generate_wrapped_linear_phase(
    width: usize,
    height: usize,
    alpha: f32,
    beta: f32,
) -> Vec<f32> {
    let mut wrapped = Vec::with_capacity(width * height);
    for row in 0..height {
        for col in 0..width {
            let phase = alpha * col as f32 + beta * row as f32;
            wrapped.push(wrap_to_pi(phase));
        }
    }
    wrapped
}

pub fn write_f32_raster(path: &Path, values: &[f32]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for &value in values {
        writer.write_all(&value.to_ne_bytes())?;
    }
    writer.flush()?;
    Ok(())
}

fn run_and_check(command: &mut Command, what: &str) -> io::Result<()> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "{what} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

pub fn run_snaphu_c_smooth_float(
    input_path: &Path,
    width: usize,
    output_path: &Path,
) -> io::Result<()> {
    let mut command = Command::new("snaphu");
    command
        .arg("-s")
        .arg("-C")
        .arg("INFILEFORMAT FLOAT_DATA")
        .arg("-C")
        .arg("OUTFILEFORMAT FLOAT_DATA")
        .arg(input_path)
        .arg(width.to_string())
        .arg("-o")
        .arg(output_path);
    run_and_check(&mut command, "snaphu C")
}

pub fn run_snaphu_rs_smooth_float(
    input_path: &Path,
    width: usize,
    output_path: &Path,
) -> io::Result<()> {
    let args = [
        "snaphu".to_string(),
        "-s".to_string(),
        "-C".to_string(),
        "INFILEFORMAT FLOAT_DATA".to_string(),
        "-C".to_string(),
        "OUTFILEFORMAT FLOAT_DATA".to_string(),
        input_path.to_string_lossy().to_string(),
        width.to_string(),
        "-o".to_string(),
        output_path.to_string_lossy().to_string(),
    ];
    snaphu_rs::run_cli(args)
}
