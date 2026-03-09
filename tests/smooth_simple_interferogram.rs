mod common;

use common::snaphu::{
    generate_wrapped_linear_phase, is_snaphu_available, run_snaphu_c_smooth_float,
    run_snaphu_rs_smooth_float, write_f32_raster,
};
use snaphu_rs::phase_compare::{gradient_compare, read_f32_raster, wrapped_error_stats};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn unique_suffix() -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}_{}", std::process::id(), stamp)
}

#[test]
fn test_linear_phase() -> Result<(), Box<dyn Error>> {
    if !is_snaphu_available() {
        eprintln!("skipping: 'snaphu' binary not found on PATH");
        return Ok(());
    }

    let width = 96usize;
    let height = 64usize;
    let alpha = 0.35f32;
    let beta = 0.21f32;
    let wrapped = generate_wrapped_linear_phase(width, height, alpha, beta);

    let tmp_dir = std::env::temp_dir().join(format!("snaphu_rs_linear_phase_{}", unique_suffix()));
    fs::create_dir_all(&tmp_dir)?;
    let _tmp_guard = TempDirGuard(tmp_dir.clone());

    let input_path = tmp_dir.join("wrapped_phase.f32");
    let c_out_path = tmp_dir.join("unwrapped_c.f32");
    let rs_out_path = tmp_dir.join("unwrapped_rs.f32");

    write_f32_raster(&input_path, &wrapped)?;
    run_snaphu_c_smooth_float(&input_path, width, &c_out_path)?;
    run_snaphu_rs_smooth_float(&input_path, width, &rs_out_path)?;

    let c_unwrapped = read_f32_raster(&c_out_path, width, height)?;
    let rs_unwrapped = read_f32_raster(&rs_out_path, width, height)?;

    let wrapped_stats = wrapped_error_stats(&c_unwrapped, &rs_unwrapped);
    let grad_stats = gradient_compare(&c_unwrapped, &rs_unwrapped, width, height);

    assert!(
        wrapped_stats.rmse < 1.0e-2,
        "wrapped RMSE too high: {} (mae={}, max_abs={})",
        wrapped_stats.rmse,
        wrapped_stats.mae,
        wrapped_stats.max_abs
    );
    assert!(
        wrapped_stats.max_abs < 5.0e-2,
        "wrapped max abs too high: {}",
        wrapped_stats.max_abs
    );
    assert!(
        grad_stats.dx_raw.rmse < 1.0e-2,
        "dx raw RMSE too high: {}",
        grad_stats.dx_raw.rmse
    );
    assert!(
        grad_stats.dy_raw.rmse < 1.0e-2,
        "dy raw RMSE too high: {}",
        grad_stats.dy_raw.rmse
    );

    Ok(())
}
