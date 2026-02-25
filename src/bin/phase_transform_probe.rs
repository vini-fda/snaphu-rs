//! Probe likely layout/endianness/flip transforms and rank best matches.
//!
//! Usage:
//! `cargo run --bin phase_transform_probe -- -W 600 -H 600 c.img rust.img`

use clap::Parser;
use snaphu_rs::phase_compare::{
    bytes_to_f32_swapped, read_f32_raster, read_f32_raster_with_bytes, score_transforms,
    transform_candidates,
};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Rank candidate transforms that best align raster B to raster A")]
struct Args {
    #[arg(short = 'W', long)]
    width: usize,
    #[arg(short = 'H', long)]
    height: usize,
    #[arg(long, default_value_t = 10)]
    top: usize,
    a: PathBuf,
    b: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let a = read_f32_raster(&args.a, args.width, args.height)?;
    let (b_native, b_bytes) = read_f32_raster_with_bytes(&args.b, args.width, args.height)?;
    let b_swapped = bytes_to_f32_swapped(&b_bytes);

    let cands = transform_candidates(&b_native, &b_swapped, args.width, args.height);
    let scores = score_transforms(&a, cands);

    println!(
        "top {} transform matches (sorted by mod2pi rmse):",
        args.top
    );
    for (i, s) in scores.iter().take(args.top).enumerate() {
        println!(
            "  {:2}. {:28} mod_rmse={:.9} mod_mae={:.9} mod_max={:.9} raw_rmse={:.9}",
            i + 1,
            s.name,
            s.mod_rmse,
            s.mod_mae,
            s.mod_max_abs,
            s.raw_rmse
        );
    }

    Ok(())
}
