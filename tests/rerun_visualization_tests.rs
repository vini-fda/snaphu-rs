//! Manual visualization tests.
//!
//! Run with:
//!   rerun
//!   cargo test --release --features rerun --test rerun_visualization_tests -- --ignored --nocapture

#![cfg(feature = "rerun")]

#[path = "common/colormap.rs"]
mod colormap;
#[path = "common/snaphu.rs"]
mod snaphu;

use colormap::cubehelix_colormap_unwrapped;
use rerun::external::arrow::buffer::{Buffer, ScalarBuffer};
use rerun::{
    ChannelDatatype, ColorModel, Image, RecordingStream, RecordingStreamBuilder, Tensor,
    TensorBuffer, TensorData,
};
use snaphu::{
    generate_wrapped_linear_phase, is_snaphu_available, run_snaphu_c_smooth_float,
    run_snaphu_rs_smooth_float, write_f32_raster,
};
use snaphu_rs::phase_compare::read_f32_raster;
use std::error::Error;
use std::fs;
use std::io;
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

/// Stores the unwrapped phases from the C and Rust SNAPHU outputs,
/// as well as the original wrapped phase input
struct Phases {
    wrapped: Vec<f32>,
    c_unwrapped: Vec<f32>,
    rs_unwrapped: Vec<f32>,
}

fn connect_rerun(session: &str) -> Result<RecordingStream, Box<dyn Error>> {
    RecordingStreamBuilder::new(session)
        .connect_grpc()
        .map_err(|err| {
            io::Error::other(format!(
            "failed to connect to Rerun viewer: {err}. Start Rerun first with `rerun` and retry",
        ))
        .into()
        })
}

/// Computes the unwrapped phases from the C and Rust SNAPHU outputs,
/// as well as the original wrapped phase input
fn compute_unwrapped_triple(width: usize, height: usize) -> Result<Phases, Box<dyn Error>> {
    let wrapped = generate_wrapped_linear_phase(width, height, 0.12, 0.08);

    let tmp_dir = std::env::temp_dir().join(format!("snaphu_rs_rerun_vis_{}", unique_suffix()));
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
    Ok(Phases {
        wrapped,
        c_unwrapped,
        rs_unwrapped,
    })
}

fn log_phase_tensor(
    rr: &RecordingStream,
    entity_path: &str,
    phase: &[f32],
    width: usize,
    height: usize,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(phase.len(), width * height);
    let n = phase.len();
    let f32_buffer = ScalarBuffer::<f32>::new(Buffer::from(phase.to_vec()), 0, n);
    let shape = ScalarBuffer::<u64>::new(Buffer::from(vec![height as u64, width as u64]), 0, 2);
    let tensor_data = TensorData {
        shape,
        names: Some(vec!["height".into(), "width".into()]),
        buffer: TensorBuffer::F32(f32_buffer),
    };
    let tensor = Tensor::new(tensor_data);
    rr.log_static(entity_path, &tensor)?;
    Ok(())
}

/// Log phase data as a Rerun Image with a specific colormap.
fn rr_phase(phase: &[f32], width: usize, height: usize) -> Image {
    assert_eq!(phase.len(), width * height);
    let mut rgb_vector = Vec::with_capacity(phase.len() * 3);
    for &p in phase {
        let rgb = cubehelix_colormap_unwrapped(p);
        rgb_vector.extend_from_slice(&rgb);
    }

    Image::from_color_model_and_bytes(
        rgb_vector,
        [width as u32, height as u32],
        ColorModel::RGB,
        ChannelDatatype::U8,
    )
}

#[test]
#[ignore = "manual visualization test: requires Rerun viewer; skip on CI"]
fn rerun_log_phase_as_tensor() -> Result<(), Box<dyn Error>> {
    if !is_snaphu_available() {
        eprintln!("skipping manual Rerun test: 'snaphu' binary not found on PATH");
        return Ok(());
    }

    let width = 320usize;
    let height = 200usize;
    let phases = compute_unwrapped_triple(width, height)?;
    let rr = connect_rerun("rerun_visualization_tensor")?;

    log_phase_tensor(&rr, "phase/wrapped_tensor", &phases.wrapped, width, height)?;
    log_phase_tensor(
        &rr,
        "phase/unwrapped_c_tensor",
        &phases.c_unwrapped,
        width,
        height,
    )?;
    log_phase_tensor(
        &rr,
        "phase/unwrapped_snaphu_rs_tensor",
        &phases.rs_unwrapped,
        width,
        height,
    )?;
    Ok(())
}

#[test]
#[ignore = "manual visualization test: requires Rerun viewer; skip on CI"]
fn rerun_log_phase_as_image() -> Result<(), Box<dyn Error>> {
    if !is_snaphu_available() {
        eprintln!("skipping manual Rerun test: 'snaphu' binary not found on PATH");
        return Ok(());
    }

    let width = 320usize;
    let height = 200usize;
    let phases = compute_unwrapped_triple(width, height)?;
    let rr = connect_rerun("rerun_visualization_image")?;

    let wrapped_image = rr_phase(&phases.wrapped, width, height);
    let c_image = rr_phase(&phases.c_unwrapped, width, height);
    let rs_image = rr_phase(&phases.rs_unwrapped, width, height);
    rr.log_static("phase/wrapped_image", &wrapped_image)?;
    rr.log_static("phase/unwrapped_c_image", &c_image)?;
    rr.log_static("phase/unwrapped_snaphu_rs_image", &rs_image)?;
    Ok(())
}
