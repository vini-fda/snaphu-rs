mod colormap;

use clap::Parser;
use colormap::cmap;
use image::{ImageBuffer, Rgb};
use rerun::external::arrow::buffer::{Buffer, ScalarBuffer};
use rerun::{RecordingStream, RecordingStreamBuilder, Tensor, TensorBuffer, TensorData};
use std::error::Error;
use std::f32::consts::TAU;
use std::fs::File;
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Parser)]
#[command(about = "Generate a synthetic interferogram and optionally unwrap it with snaphu")] 
struct Args {
    /// Number of columns in the generated interferogram
    #[arg(long, default_value_t = 1024)]
    width: usize,
    /// Number of rows in the generated interferogram
    #[arg(long, default_value_t = 512)]
    height: usize,
    /// Mix between azimuth and range slopes (0 = azimuth, 1 = range)
    #[arg(long, default_value_t = 0.1)]
    tilt_weight: f32,
    /// Number of wraps to add to the synthetic phase ramp
    #[arg(long, default_value_t = 5.0)]
    wraps: f32,
    /// Directory where intermediate files will be written
    #[arg(long, default_value = "target/examples/complex_writer")]
    out_dir: PathBuf,
    /// Path to the snaphu binary (falls back to snaphu on $PATH)
    #[arg(long, value_hint = clap::ValueHint::ExecutablePath, default_value = "snaphu")]
    snaphu_bin: PathBuf,
    /// Skip invoking snaphu even if a binary is available
    #[arg(long)]
    skip_snaphu: bool,
    /// Additional CLI arguments forwarded to snaphu (repeatable)
    #[arg(long = "snaphu-arg")]
    snaphu_args: Vec<String>,
    /// Connect to a running Rerun viewer and stream tensors
    #[arg(long)]
    rerun: bool,
    /// Optional label for the Rerun recording session
    #[arg(long, default_value = "complex_writer")]
    rerun_session: String,
    /// Emit a PNG visualization of the wrapped interferogram
    #[arg(long)]
    write_png: bool,
}

pub fn read_float_file(path: impl AsRef<Path>, width: usize, height: usize) -> io::Result<Vec<f32>> {
    let mut file = File::open(path.as_ref())?;

    // Validate file size
    let expected_size = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(std::mem::size_of::<f32>()))
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "size overflow"))?;

    let actual_size = file.metadata()?.len() as usize;
    if actual_size != expected_size {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "file size mismatch: expected {} bytes, got {}",
                expected_size, actual_size
            ),
        ));
    }

    // Read entire file
    let mut buf = vec![0u8; actual_size];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buf)?;

    // Convert bytes → f32
    let mut data = Vec::with_capacity(width * height);
    for chunk in buf.chunks_exact(4) {
        data.push(f32::from_ne_bytes(chunk.try_into().unwrap()));
    }

    Ok(data)
}

pub fn read_complex_file(
    path: impl AsRef<Path>,
    width: usize,
    height: usize,
) -> io::Result<Vec<(f32, f32)>> {
    let mut file = File::open(path.as_ref())?;

    // Validate file size
    let expected_size = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(2 * std::mem::size_of::<f32>()))
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "size overflow"))?;

    let actual_size = file.metadata()?.len() as usize;
    if actual_size != expected_size {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "file size mismatch: expected {} bytes, got {}",
                expected_size, actual_size
            ),
        ));
    }

    // Read entire file
    let mut buf = vec![0u8; actual_size];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buf)?;

    // Convert bytes → f32
    let mut data = Vec::with_capacity(width * height);
    for chunk in buf.chunks_exact(8) {
        let re = f32::from_ne_bytes(chunk[0..4].try_into().unwrap());
        let im = f32::from_ne_bytes(chunk[4..8].try_into().unwrap());
        data.push((re, im));
    }

    Ok(data)
}

pub fn read_alt_line_file(
    path: impl AsRef<Path>,
    width: usize,
    height: usize,
) -> io::Result<(Vec<f32>, Vec<f32>)> {
    let mut file = File::open(path.as_ref())?;

    // Validate file size
    let expected_size = width
        .checked_mul(height)
        .and_then(|v| v.checked_mul(2 * std::mem::size_of::<f32>()))
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "size overflow"))?;

    let actual_size = file.metadata()?.len() as usize;
    if actual_size != expected_size {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "file size mismatch: expected {} bytes, got {}",
                expected_size, actual_size
            ),
        ));
    }

    // Read entire file
    let mut buf = vec![0u8; actual_size];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut buf)?;

    // Convert bytes → f32
    let mut data_0 = Vec::with_capacity(width * height);
    let mut data_1 = Vec::with_capacity(width * height);
    for (i, chunk) in buf.chunks_exact(4).enumerate() {
        let sample = f32::from_ne_bytes(chunk.try_into().unwrap());
        if (i / width) % 2 == 0 {
            data_0.push(sample);
        } else {
            data_1.push(sample);
        }
    }

    Ok((data_0, data_1))
}

pub fn write_complex_file<P, F>(path: P, width: usize, height: usize, f: F) -> io::Result<()>
where
    P: AsRef<Path>,
    F: Fn(usize, usize) -> (f32, f32),
{
    let file = File::create(path.as_ref())?;
    let mut writer = BufWriter::new(file);

    for i in 0..height {
        for j in 0..width {
            let (re, im) = f(i, j);
            writer.write_all(&re.to_ne_bytes())?;
            writer.write_all(&im.to_ne_bytes())?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn write_png_file<P, F, C>(
    path: P,
    width: usize,
    height: usize,
    f: F,
    cmap: C,
) -> io::Result<()>
where
    P: AsRef<Path>,
    F: Fn(usize, usize) -> (f32, f32),
    C: Fn(f32, f32) -> [u8; 3],
{
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width as u32, height as u32);

    for i in 0..height {
        for j in 0..width {
            let (re, im) = f(i, j);
            let rgb = cmap(re, im);

            let pixel = Rgb(rgb);

            img.put_pixel(j as u32, i as u32, pixel);
        }
    }
    img.save(path.as_ref())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

pub fn log_rerun_img_from_phase(
    name: &str,
    samples: &[f32],
    width: usize,
    height: usize,
    rr: &RecordingStream,
) {
    let n = samples.len();
    assert_eq!(n, width * height);
    let f32_buffer = ScalarBuffer::<f32>::new(Buffer::from(samples.to_vec()), 0, n);
    let shape = ScalarBuffer::<u64>::new(Buffer::from(vec![height as u64, width as u64]), 0, 2);
    let tensor_data = TensorData {
        shape,
        names: Some(vec!["height".into(), "width".into()]),
        buffer: TensorBuffer::F32(f32_buffer),
    };
    let tensor = Tensor::new(tensor_data);
    if let Err(err) = rr.log_static(name, &tensor) {
        eprintln!("Failed to log {name} to Rerun: {err}");
    }
}

struct PhaseField {
    width: usize,
    height: usize,
    wraps: f32,
    tilt_weight: f32,
}

impl PhaseField {
    fn new(width: usize, height: usize, wraps: f32, tilt_weight: f32) -> Self {
        Self {
            width,
            height,
            wraps,
            tilt_weight,
        }
    }

    fn phase(&self, row: usize, col: usize) -> f32 {
        let dy = 1.0 / self.height as f32;
        let dx = 1.0 / self.width as f32;
        let tilt = self.tilt_weight.clamp(0.0, 1.0);
        self.wraps * TAU * (col as f32 * dx * tilt + row as f32 * dy * (1.0 - tilt))
    }

    fn sample_complex(&self, row: usize, col: usize) -> (f32, f32) {
        let phi = self.phase(row, col);
        (phi.cos(), phi.sin())
    }

    fn original_phase(&self) -> Vec<f32> {
        let mut buf = Vec::with_capacity(self.width * self.height);
        for row in 0..self.height {
            for col in 0..self.width {
                buf.push(self.phase(row, col));
            }
        }
        buf
    }
}

fn run_snaphu(
    binary: &Path,
    input_path: &Path,
    width: usize,
    output_path: &Path,
    extra_args: &[String],
) -> io::Result<std::process::Output> {
    let mut command = Command::new(binary);
    if extra_args.is_empty() {
        command.arg("-s");
    } else {
        command.args(extra_args);
    }
    command
        .arg(input_path)
        .arg(width.to_string())
        .arg("-o")
        .arg(output_path);
    command.output()
}

fn run_snaphu_and_read_products(
    binary: &Path,
    width: usize,
    height: usize,
    input_path: &Path,
    output_path: &Path,
    extra_args: &[String],
) -> io::Result<(Vec<f32>, Vec<f32>)> {
    let output = run_snaphu(binary, input_path, width, output_path, extra_args)?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "snaphu exited with {:?}:\nstdout: {}\nstderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    if !output.stdout.is_empty() {
        println!("snaphu stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprintln!("snaphu stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    }

    read_alt_line_file(output_path, width, height)
}

fn connect_rerun(session: &str) -> Option<RecordingStream> {
    match RecordingStreamBuilder::new(session).connect_grpc() {
        Ok(rr) => Some(rr),
        Err(err) => {
            eprintln!("Failed to connect to Rerun viewer: {err}");
            None
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    std::fs::create_dir_all(&args.out_dir)?;

    let phase_field = PhaseField::new(args.width, args.height, args.wraps, args.tilt_weight);
    let wrapped_path = args.out_dir.join("wrapped_phase.bin");
    let snaphu_out_path = args.out_dir.join("snaphu.out");
    let png_path = args.out_dir.join("wrapped_phase.png");

    write_complex_file(&wrapped_path, args.width, args.height, |row, col| {
        phase_field.sample_complex(row, col)
    })?;
    println!("Saved wrapped interferogram to {}", wrapped_path.display());

    if args.write_png {
        write_png_file(&png_path, args.width, args.height, |row, col| {
            phase_field.sample_complex(row, col)
        }, cmap)?;
        println!("Saved PNG visualization to {}", png_path.display());
    }

    let rerun_stream = if args.rerun {
        connect_rerun(&args.rerun_session)
    } else {
        None
    };

    let original_phase = phase_field.original_phase();
    let wrapped_phase: Vec<f32> = original_phase
        .iter()
        .map(|phi| phi.rem_euclid(TAU))
        .collect();

    if let Some(rr) = rerun_stream.as_ref() {
        for (name, samples) in [
            ("original_phase", &original_phase),
            ("wrapped_phase", &wrapped_phase),
        ] {
            log_rerun_img_from_phase(name, samples, args.width, args.height, rr);
        }
    }

    if args.skip_snaphu {
        println!("Skipping snaphu execution (pass --skip-snaphu=false to run it).");
    } else {
        let (magnitudes, unwrapped_phase) = run_snaphu_and_read_products(
            &args.snaphu_bin,
            args.width,
            args.height,
            &wrapped_path,
            &snaphu_out_path,
            &args.snaphu_args,
        )?;
        println!("Saved snaphu output to {}", snaphu_out_path.display());

        if let Some(rr) = rerun_stream.as_ref() {
            for (name, samples) in [
                ("unwrapped_phase", &unwrapped_phase),
                ("magnitudes", &magnitudes),
            ] {
                log_rerun_img_from_phase(name, samples, args.width, args.height, rr);
            }
        }
    }

    println!(
        "Done. Inspect generated files under {} or stream them with Rerun.",
        args.out_dir.display()
    );

    Ok(())
}
