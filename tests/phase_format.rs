//! End-to-end coverage for the snaphu-rs `.phase` input/output format.
//!
//! The fixture `testcases/phase_format/original_phase_example.phase` holds an
//! 8x8 raster of *unwrapped* phase. The tests wrap it, unwrap it again through
//! the CLI, and check that the `.phase` path behaves exactly like the plain
//! `FLOAT_DATA` path.

#[path = "common/snaphu.rs"]
mod snaphu;

use snaphu::{wrap_to_pi, write_f32_raster};
use snaphu_rs::data::raster::Raster;
use snaphu_rs::io::phase_format::{PHASE_HEADER_LEN, read_phase_header, read_phase_raster};
use std::error::Error;
use std::f32::consts::TAU;
use std::fs;
use std::path::{Path, PathBuf};
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

fn example_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testcases")
        .join("phase_format")
        .join("original_phase_example.phase")
}

fn run_smooth(
    input: &Path,
    width: Option<usize>,
    output: &Path,
    format: &str,
) -> Result<(), Box<dyn Error>> {
    let mut args = vec![
        "snaphu".to_string(),
        "-s".to_string(),
        "-C".to_string(),
        format!("INFILEFORMAT {format}"),
        "-C".to_string(),
        format!("OUTFILEFORMAT {format}"),
        input.to_string_lossy().to_string(),
    ];
    if let Some(width) = width {
        args.push(width.to_string());
    }
    args.push("-o".to_string());
    args.push(output.to_string_lossy().to_string());
    snaphu_rs::run_cli(args)?;
    Ok(())
}

#[test]
fn example_file_matches_the_documented_layout() -> Result<(), Box<dyn Error>> {
    let path = example_path();
    assert_eq!(
        fs::metadata(&path)?.len(),
        (PHASE_HEADER_LEN + 8 * 8 * 4) as u64
    );

    let dims = read_phase_header(&path)?;
    assert_eq!((dims.nrows, dims.ncols), (8, 8));

    let raster = read_phase_raster(&path)?;
    assert_eq!((raster.height, raster.width), (8, 8));
    assert_eq!(raster.data.len(), 64);
    // First and last samples, row-major.
    assert!((raster.data[0] - (-2.356_194_5)).abs() < 1.0e-5);
    assert!((raster.data[63] - 10.585_398).abs() < 1.0e-5);
    // The fixture is unwrapped phase, so it leaves the [-pi, pi) interval.
    assert!(raster.data.iter().any(|v| *v > std::f32::consts::PI));
    Ok(())
}

#[test]
fn phase_format_round_trips_through_the_cli() -> Result<(), Box<dyn Error>> {
    let (dir, _guard) = temp_dir("snaphu_rs_phase_cli")?;
    let original = read_phase_raster(&example_path())?;
    let wrapped: Vec<f32> = original.data.iter().copied().map(wrap_to_pi).collect();

    let input = dir.join("wrapped.phase");
    let output = dir.join("unwrapped.phase");
    snaphu_rs::io::phase_format::write_phase_file(
        &Raster::new(original.width, original.height, wrapped.clone()),
        &input,
    )?;

    run_smooth(
        &input,
        Some(original.width),
        &output,
        "FLOAT_DATA_PHASE_FORMAT",
    )?;

    let unwrapped = read_phase_raster(&output)?;
    assert_eq!((unwrapped.height, unwrapped.width), (8, 8));

    // Whatever the solver decides, unwrapping may only add integer cycles.
    for (index, (out, inp)) in unwrapped.data.iter().zip(wrapped.iter()).enumerate() {
        let cycles = (out - inp) / TAU;
        assert!(
            (cycles - cycles.round()).abs() < 1.0e-3,
            "sample {index}: {out} is not {inp} plus a whole number of cycles"
        );
    }
    Ok(())
}

#[test]
fn phase_format_matches_the_plain_float_path() -> Result<(), Box<dyn Error>> {
    let (dir, _guard) = temp_dir("snaphu_rs_phase_vs_float")?;
    let original = read_phase_raster(&example_path())?;
    let wrapped: Vec<f32> = original.data.iter().copied().map(wrap_to_pi).collect();

    let float_input = dir.join("wrapped.f32");
    let float_output = dir.join("unwrapped.f32");
    write_f32_raster(&float_input, &wrapped)?;
    run_smooth(
        &float_input,
        Some(original.width),
        &float_output,
        "FLOAT_DATA",
    )?;

    let phase_input = dir.join("wrapped.phase");
    let phase_output = dir.join("unwrapped.phase");
    snaphu_rs::io::phase_format::write_phase_file(
        &Raster::new(original.width, original.height, wrapped),
        &phase_input,
    )?;
    run_smooth(
        &phase_input,
        Some(original.width),
        &phase_output,
        "FLOAT_DATA_PHASE_FORMAT",
    )?;

    let from_float =
        snaphu_rs::phase_compare::read_f32_raster(&float_output, original.width, original.height)?;
    let from_phase = read_phase_raster(&phase_output)?;
    assert_eq!(
        from_phase.data, from_float,
        "the container must not change the unwrapped values"
    );
    Ok(())
}

#[test]
fn width_argument_is_optional_for_phase_input() -> Result<(), Box<dyn Error>> {
    let (dir, _guard) = temp_dir("snaphu_rs_phase_no_width")?;
    let original = read_phase_raster(&example_path())?;
    let wrapped: Vec<f32> = original.data.iter().copied().map(wrap_to_pi).collect();

    let input = dir.join("wrapped.phase");
    snaphu_rs::io::phase_format::write_phase_file(
        &Raster::new(original.width, original.height, wrapped),
        &input,
    )?;

    let with_width = dir.join("with_width.phase");
    let without_width = dir.join("without_width.phase");
    run_smooth(
        &input,
        Some(original.width),
        &with_width,
        "FLOAT_DATA_PHASE_FORMAT",
    )?;
    run_smooth(&input, None, &without_width, "FLOAT_DATA_PHASE_FORMAT")?;

    assert_eq!(
        read_phase_raster(&without_width)?,
        read_phase_raster(&with_width)?
    );
    Ok(())
}

#[test]
fn wrong_width_argument_is_rejected_for_phase_input() -> Result<(), Box<dyn Error>> {
    let (dir, _guard) = temp_dir("snaphu_rs_phase_bad_width")?;
    let original = read_phase_raster(&example_path())?;
    let wrapped: Vec<f32> = original.data.iter().copied().map(wrap_to_pi).collect();

    let input = dir.join("wrapped.phase");
    snaphu_rs::io::phase_format::write_phase_file(
        &Raster::new(original.width, original.height, wrapped),
        &input,
    )?;

    let err = run_smooth(
        &input,
        Some(original.width + 1),
        &dir.join("out.phase"),
        "FLOAT_DATA_PHASE_FORMAT",
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("samples per line"),
        "unexpected error: {err}"
    );
    Ok(())
}
