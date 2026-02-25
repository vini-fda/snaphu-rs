//! Emit debug PNGs for phase comparison:
//! - `A`, `B` with shared range
//! - `A-B`
//! - `(A-B) mod 2pi`
//! - integer cycle map `k = round((B-A)/2pi)`
//!
//! Usage:
//! `cargo run --bin phase_png_debug -- -W 600 -H 600 c.img rust.img --prefix debug_600`

use clap::Parser;
use image::{ImageBuffer, Luma};
use snaphu_rs::phase_compare::{diff_mod_k, linear_scale_u8, min_max, read_f32_raster};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Write PNG debug products for two phase rasters")]
struct Args {
    #[arg(short = 'W', long)]
    width: usize,
    #[arg(short = 'H', long)]
    height: usize,
    #[arg(long, default_value = "phase_debug")]
    prefix: String,
    a: PathBuf,
    b: PathBuf,
}

fn save_gray_png(
    path: &PathBuf,
    data: &[f32],
    width: usize,
    height: usize,
    min_v: f32,
    max_v: f32,
) -> Result<(), Box<dyn Error>> {
    let pixels = linear_scale_u8(data, min_v, max_v);
    let img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_vec(u32::try_from(width)?, u32::try_from(height)?, pixels)
            .ok_or("failed to build image buffer")?;
    img.save(path)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let a = read_f32_raster(&args.a, args.width, args.height)?;
    let b = read_f32_raster(&args.b, args.width, args.height)?;
    let (diff, mod_res, k) = diff_mod_k(&a, &b);
    let k_f32 = k.iter().map(|&v| v as f32).collect::<Vec<_>>();

    let (a_min, a_max) = min_max(&a);
    let (b_min, b_max) = min_max(&b);
    let ab_min = a_min.min(b_min);
    let ab_max = a_max.max(b_max);

    let diff_max_abs = diff
        .iter()
        .fold(0.0f32, |acc, &v| acc.max(v.abs()))
        .max(1e-6);
    let (k_min, k_max) = min_max(&k_f32);

    let prefix = PathBuf::from(args.prefix);
    let a_png = prefix.with_extension("a.png");
    let b_png = prefix.with_extension("b.png");
    let d_png = prefix.with_extension("diff.png");
    let m_png = prefix.with_extension("diff_mod2pi.png");
    let k_png = prefix.with_extension("kmap.png");

    save_gray_png(&a_png, &a, args.width, args.height, ab_min, ab_max)?;
    save_gray_png(&b_png, &b, args.width, args.height, ab_min, ab_max)?;
    save_gray_png(
        &d_png,
        &diff,
        args.width,
        args.height,
        -diff_max_abs,
        diff_max_abs,
    )?;
    save_gray_png(
        &m_png,
        &mod_res,
        args.width,
        args.height,
        -std::f32::consts::PI,
        std::f32::consts::PI,
    )?;
    save_gray_png(&k_png, &k_f32, args.width, args.height, k_min, k_max)?;

    println!("wrote {}", a_png.display());
    println!("wrote {}", b_png.display());
    println!("wrote {}", d_png.display());
    println!("wrote {}", m_png.display());
    println!("wrote {}", k_png.display());
    Ok(())
}
