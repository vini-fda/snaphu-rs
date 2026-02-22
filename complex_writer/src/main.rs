use complex_writer::colormap::cubehelix_colormap;
use image::{ImageBuffer, Rgb};
use rerun::external::arrow::buffer::{Buffer, ScalarBuffer};
use rerun::{Color, Image, RecordingStream};
use std::f32::consts::{PI, TAU};
use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Rem;
use std::path::Path;

// TODO: improve this so it is not hardcoded.
const SNAPHU_PATH: &'static str =
    "/Users/vinifreitas/Programming/experiment_snaphu/snaphu-v2.0.7/bin/snaphu";

pub fn run_snaphu<I, S>(args: I) -> std::io::Result<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    // Optional: validate binary exists
    if !Path::new(SNAPHU_PATH).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("snaphu binary not found at {}", SNAPHU_PATH),
        ));
    }

    std::process::Command::new(SNAPHU_PATH)
        .args(args.into_iter().map(|s| s.as_ref().to_string()))
        .output()
}

pub fn read_float_file(path: &str, width: usize, height: usize) -> std::io::Result<Vec<f32>> {
    let mut file = File::open(path)?;

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
    path: &str,
    width: usize,
    height: usize,
) -> std::io::Result<Vec<(f32, f32)>> {
    let mut file = File::open(path)?;

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
    path: &str,
    width: usize,
    height: usize,
) -> std::io::Result<(Vec<f32>, Vec<f32>)> {
    let mut file = File::open(path)?;

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

pub fn write_complex_file<F>(path: &str, width: usize, height: usize, f: F) -> std::io::Result<()>
where
    F: Fn(usize, usize) -> (f32, f32),
{
    let file = File::create(path)?;
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

pub fn write_png_file<F, C>(
    path: &str,
    width: usize,
    height: usize,
    f: F,
    cmap: C,
) -> std::io::Result<()>
where
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

    img.save(Path::new(path))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
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
    let tensor_data = rerun::TensorData {
        shape,
        names: Some(vec!["height".into(), "width".into()]),
        buffer: rerun::TensorBuffer::F32(f32_buffer),
    };
    let tensor = rerun::Tensor::new(tensor_data);
    rr.log_static(name, &tensor)
        .expect("Could not log phase tensor");
}

// TODO: use this unfinished script as inspiration to create wrapped interferograms for SNAPHU
// It is especially useful for human visualization through [Rerun](https://rerun.io/)
fn main() -> std::io::Result<()> {
    let width = 1024;
    let height = 512;
    let phase = |i: usize, j: usize| {
        let dy = 1.0 / height as f32;
        let dx = 1.0 / width as f32;
        let t = 0.1;
        5.0 * TAU * (j as f32 * dx * t + i as f32 * dy * (1.0 - t))
    };
    let re_im = |i: usize, j: usize| {
        let phi = phase(i, j);
        (phi.cos(), phi.sin())
    };

    write_complex_file("wrappedfile.bin", width, height, re_im)?;

    // run_snaphu([
    //     "-s",
    //     "wrappedfile.bin",
    //     "1024",
    //     "-o",
    //     "/Users/vinifreitas/Programming/experiment_snaphu/complex_writer/snaphu.out",
    // ])?;

    let (magnitudes, unwrapped_phase) = read_alt_line_file("snaphu.out", width, height)?;

    let mut original_phase = Vec::with_capacity(width * height);
    for i in 0..height {
        for j in 0..width {
            original_phase.push(phase(i, j));
        }
    }
    let wrapped_phase: Vec<f32> = original_phase.iter().map(|x| x.rem(TAU)).collect();
    let rr = rerun::RecordingStreamBuilder::new("plot_phase")
        .connect_grpc()
        .expect("Could not connect to local Rerun instance.");
    log_rerun_img_from_phase("original_phase", &original_phase, width, height, &rr);
    log_rerun_img_from_phase("wrapped_phase", &wrapped_phase, width, height, &rr);
    log_rerun_img_from_phase("unwrapped_phase", &unwrapped_phase, width, height, &rr);
    log_rerun_img_from_phase("magnitudes", &magnitudes, width, height, &rr);

    // write_png_file("wrappedfile.png", width, height, phase, cmap)?;
    Ok(())
}
