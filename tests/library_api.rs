//! The CLI must be a thin wrapper over the library API: same data in, same
//! numbers out. These tests pin that equivalence.

use snaphu_rs::data::raster::Raster;
use snaphu_rs::io::phase_format::{read_phase_raster, write_phase_file};
use snaphu_rs::{CostMode, RunConfig, UnwrapInputs, arc_count, run_snaphu, run_snaphu_inplace};
use std::error::Error;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(name: &str) -> Result<(PathBuf, TempDirGuard), Box<dyn Error>> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("{name}_{}_{}", std::process::id(), stamp));
    fs::create_dir_all(&dir)?;
    Ok((dir.clone(), TempDirGuard(dir)))
}

/// A wrapped ramp with a sinusoidal wobble, so the solver has real work to do.
fn wrapped_scene(rows: usize, cols: usize) -> Raster<f32> {
    let mut data = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        for col in 0..cols {
            let phase = 0.35 * col as f32
                + 0.21 * row as f32
                + 2.0 * (row as f32 / 9.0).sin() * (col as f32 / 7.0).cos();
            data.push(phase - TAU * (phase / TAU).floor());
        }
    }
    Raster::new(cols, rows, data)
}

fn cli_unwrap(
    scene: &Raster<f32>,
    dir: &std::path::Path,
    extra: &[&str],
) -> Result<Raster<f32>, Box<dyn Error>> {
    let input = dir.join("wrapped.phase");
    let output = dir.join("unwrapped.phase");
    write_phase_file(scene, &input)?;

    let mut args = vec![
        "snaphu".to_string(),
        "-s".to_string(),
        "-C".to_string(),
        "INFILEFORMAT FLOAT_DATA_PHASE_FORMAT".to_string(),
        "-C".to_string(),
        "OUTFILEFORMAT FLOAT_DATA_PHASE_FORMAT".to_string(),
    ];
    args.extend(extra.iter().map(|s| s.to_string()));
    args.push(input.to_string_lossy().to_string());
    args.push("-o".to_string());
    args.push(output.to_string_lossy().to_string());
    snaphu_rs::run_cli(args)?;

    Ok(read_phase_raster(&output)?)
}

fn smooth_config() -> RunConfig {
    RunConfig {
        cost_mode: CostMode::Smooth,
        ..RunConfig::default()
    }
}

#[test]
fn library_api_matches_the_cli_single_tile() -> Result<(), Box<dyn Error>> {
    let (dir, _guard) = temp_dir("snaphu_rs_api_single")?;
    let scene = wrapped_scene(40, 32);

    let from_cli = cli_unwrap(&scene, &dir, &[])?;
    let from_api = run_snaphu(&UnwrapInputs::new(&scene), &smooth_config())?;

    assert_eq!(
        from_api.unwrapped_phase.data, from_cli.data,
        "library and CLI must agree bit for bit"
    );
    assert_eq!(from_api.flows.unwrap().len(), arc_count(40, 32));
    Ok(())
}

#[test]
fn library_api_matches_the_cli_multi_tile() -> Result<(), Box<dyn Error>> {
    let (dir, _guard) = temp_dir("snaphu_rs_api_multi")?;
    let scene = wrapped_scene(64, 64);

    let from_cli = cli_unwrap(&scene, &dir, &["--tile", "2", "2", "0", "0"])?;
    let config = RunConfig {
        ntilerow: 2,
        ntilecol: 2,
        ..smooth_config()
    };
    let from_api = run_snaphu(&UnwrapInputs::new(&scene), &config)?;

    assert_eq!(from_api.unwrapped_phase.data, from_cli.data);
    // Tile assembly owns the flows, so they are not reported back.
    assert!(from_api.flows.is_none());
    assert!(from_api.diagnostics.is_none());
    Ok(())
}

#[test]
fn inplace_writes_caller_buffers_without_allocating_outputs() -> Result<(), Box<dyn Error>> {
    let scene = wrapped_scene(24, 20);
    let config = smooth_config();
    let expected = run_snaphu(&UnwrapInputs::new(&scene), &config)?;

    // Buffers the caller owns and can reuse across runs.
    let mut phase = vec![0.0f32; 24 * 20];
    let mut flows = vec![0i16; arc_count(24, 20)];
    let phase_ptr = phase.as_ptr();

    let report = run_snaphu_inplace(
        &UnwrapInputs::new(&scene),
        &config,
        &mut phase,
        Some(&mut flows),
        None,
        None,
    )?;

    assert_eq!((report.rows, report.cols), (24, 20));
    assert!(report.flows_written);
    assert!(!report.magnitude_written);
    assert_eq!(phase, expected.unwrapped_phase.data);
    assert_eq!(flows, expected.flows.unwrap());
    assert!(
        std::ptr::eq(phase_ptr, phase.as_ptr()),
        "buffer was replaced"
    );
    Ok(())
}

#[test]
fn pre_wrapped_inputs_skip_the_defensive_copy() -> Result<(), Box<dyn Error>> {
    let scene = wrapped_scene(16, 16); // already in [0, 2pi)
    let config = smooth_config();

    let wrapping = run_snaphu(&UnwrapInputs::new(&scene), &config)?;
    let mut inputs = UnwrapInputs::new(&scene);
    inputs.wrap_input = false;
    let borrowed = run_snaphu(&inputs, &config)?;

    assert_eq!(borrowed.unwrapped_phase.data, wrapping.unwrapped_phase.data);
    Ok(())
}
