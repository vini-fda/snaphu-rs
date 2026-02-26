//! Manual Rerun visualization tests for real Kumamoto datasets.
//!
//! Run with:
//!   rerun
//!   cargo test --test rerun_kumamoto -- --ignored --nocapture

#[path = "common/colormap.rs"]
mod colormap;
#[path = "common/kumamoto.rs"]
mod kumamoto;
#[path = "common/snaphu.rs"]
mod snaphu;

use colormap::cubehelix_colormap_unwrapped;
use kumamoto::KumamotoOutputs;
use rerun::external::arrow::buffer::{Buffer, ScalarBuffer};
use rerun::{
    ChannelDatatype, ColorModel, Image, RecordingStream, RecordingStreamBuilder, Tensor,
    TensorBuffer, TensorData,
};
use std::error::Error;
use std::io;

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

fn load_case_outputs() -> Result<KumamotoOutputs, Box<dyn Error>> {
    let case = kumamoto::cases()[1];
    if let Err(reason) = kumamoto::can_run_case(case) {
        return Err(io::Error::other(format!("cannot run {}: {reason}", case.name)).into());
    }
    kumamoto::run_case(case)
}

#[test]
#[ignore = "manual visualization test: requires Rerun viewer and local testcases"]
fn rerun_kumamoto_as_tensor() -> Result<(), Box<dyn Error>> {
    let out = load_case_outputs()?;
    let rr = connect_rerun("rerun_kumamoto_tensor")?;
    let width = out.case.width;
    let height = out.case.height;

    log_phase_tensor(&rr, "kumamoto/wrapped_tensor", &out.wrapped, width, height)?;
    log_phase_tensor(
        &rr,
        "kumamoto/unwrapped_c_tensor",
        &out.c_unwrapped,
        width,
        height,
    )?;
    log_phase_tensor(
        &rr,
        "kumamoto/unwrapped_snaphu_rs_tensor",
        &out.rs_unwrapped,
        width,
        height,
    )?;
    Ok(())
}

#[test]
#[ignore = "manual visualization test: requires Rerun viewer and local testcases"]
fn rerun_kumamoto_as_image() -> Result<(), Box<dyn Error>> {
    let out = load_case_outputs()?;
    let rr = connect_rerun("rerun_kumamoto_image")?;
    let width = out.case.width;
    let height = out.case.height;

    let wrapped_image = rr_phase(&out.wrapped, width, height);
    let c_image = rr_phase(&out.c_unwrapped, width, height);
    let rs_image = rr_phase(&out.rs_unwrapped, width, height);
    rr.log_static("kumamoto/wrapped_image", &wrapped_image)?;
    rr.log_static("kumamoto/unwrapped_c_image", &c_image)?;
    rr.log_static("kumamoto/unwrapped_snaphu_rs_image", &rs_image)?;
    Ok(())
}
