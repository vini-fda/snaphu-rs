//! Log raw float32 phase rasters (wrapped or unwrapped) to a running Rerun
//! viewer as 2-D tensors.
//!
//! Usage:
//!
//! ```bash
//! # Start the Rerun viewer first
//! rerun
//!
//! # Then log one or more phase files (width and height are required)
//! cargo run --features rerun --example phase_viewer -- -W 24639 -H 4187 wrapped.img unwrapped.img
//! ```
//!
//! Each file is logged as a separate entity whose name is derived from the
//! filename stem.  If either dimension exceeds the GPU texture limit
//! (`--max-dim`, default 16384) the raster is cropped to `[0..max_dim]` on
//! the offending axis before upload.

use clap::Parser;
use rerun::external::arrow::buffer::{Buffer, ScalarBuffer};
use rerun::{RecordingStreamBuilder, Tensor, TensorBuffer, TensorData};
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;

/// Maximum texture dimension supported by most GPUs (D3D11, Metal, WebGPU).
/// Rerun maps tensors to GPU textures, so we crop to fit within this
/// limit to avoid "texture larger than max" errors.
const MAX_TEXTURE_DIM: usize = 16384;

#[derive(Debug, Parser)]
#[command(about = "Log raw float phase rasters to Rerun as tensors")]
struct Args {
    /// Float32 raster files to log (entity name derived from filename)
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Image width in pixels (samples per row)
    #[arg(short = 'W', long)]
    width: usize,
    /// Image height in pixels (number of rows)
    #[arg(short = 'H', long)]
    height: usize,
    /// Rerun recording session label
    #[arg(long, default_value = "phase_viewer")]
    session: String,
    /// Max texture dimension (default 8192; capped to avoid GPU limits)
    #[arg(long, default_value_t = MAX_TEXTURE_DIM)]
    max_dim: usize,
}

fn read_float_file(path: &PathBuf, width: usize, height: usize) -> io::Result<Vec<f32>> {
    let mut file = File::open(path)?;
    let expected = width * height * std::mem::size_of::<f32>();
    let actual = file.metadata()?.len() as usize;
    if actual != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{}: expected {} bytes ({}x{}x4), got {}",
                path.display(),
                expected,
                width,
                height,
                actual
            ),
        ));
    }
    let mut buf = vec![0u8; actual];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buf)?;
    Ok(buf
        .chunks_exact(4)
        .map(|c| f32::from_ne_bytes(c.try_into().unwrap()))
        .collect())
}

/// Crop a raster to at most `max_w` x `max_h`, keeping the top-left corner.
fn crop(
    data: &[f32],
    width: usize,
    height: usize,
    max_w: usize,
    max_h: usize,
) -> (Vec<f32>, usize, usize) {
    let cw = width.min(max_w);
    let ch = height.min(max_h);
    let mut out = Vec::with_capacity(cw * ch);
    for r in 0..ch {
        out.extend_from_slice(&data[r * width..r * width + cw]);
    }
    (out, cw, ch)
}

fn log_tensor(name: &str, data: &[f32], width: usize, height: usize, rr: &rerun::RecordingStream) {
    let n = data.len();
    let f32_buf = ScalarBuffer::<f32>::new(Buffer::from(data.to_vec()), 0, n);
    let shape = ScalarBuffer::<u64>::new(Buffer::from(vec![height as u64, width as u64]), 0, 2);
    let tensor_data = TensorData {
        shape,
        names: Some(vec!["height".into(), "width".into()]),
        buffer: TensorBuffer::F32(f32_buf),
    };
    let tensor = Tensor::new(tensor_data);
    if let Err(err) = rr.log_static(name, &tensor) {
        eprintln!("Failed to log {name}: {err}");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rr = RecordingStreamBuilder::new(args.session.as_str()).connect_grpc()?;
    eprintln!("Connected to Rerun viewer");

    let needs_crop = args.width > args.max_dim || args.height > args.max_dim;
    if needs_crop {
        eprintln!(
            "Cropping to {}x{} (from {}x{}) to fit GPU max texture dim",
            args.width.min(args.max_dim),
            args.height.min(args.max_dim),
            args.width,
            args.height,
        );
    }

    for path in &args.files {
        let entity = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());
        eprintln!("Reading {} ...", path.display());
        let data = read_float_file(path, args.width, args.height)?;
        let (log_data, log_w, log_h) = if needs_crop {
            crop(&data, args.width, args.height, args.max_dim, args.max_dim)
        } else {
            (data, args.width, args.height)
        };
        eprintln!("Logging as '{}' ({}x{})", entity, log_w, log_h);
        log_tensor(&entity, &log_data, log_w, log_h, &rr);
    }

    eprintln!("Done — all tensors logged.");
    Ok(())
}
