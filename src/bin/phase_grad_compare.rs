//! Compare phase gradients (`dx`, `dy`) between two rasters.
//!
//! This is robust to global additive offsets and highlights local structure
//! mismatches.
//!
//! Usage:
//! `cargo run --bin phase_grad_compare -- -W 600 -H 600 c.img rust.img`

use clap::Parser;
use snaphu_rs::phase_compare::{gradient_compare, read_f32_raster};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Compare raw and wrapped phase gradients between two rasters")]
struct Args {
    #[arg(short = 'W', long)]
    width: usize,
    #[arg(short = 'H', long)]
    height: usize,
    a: PathBuf,
    b: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let a = read_f32_raster(&args.a, args.width, args.height)?;
    let b = read_f32_raster(&args.b, args.width, args.height)?;

    let g = gradient_compare(&a, &b, args.width, args.height);
    println!(
        "dx raw      mae={:.9} rmse={:.9} max_abs={:.9}",
        g.dx_raw.mae, g.dx_raw.rmse, g.dx_raw.max_abs
    );
    println!(
        "dx wrapped  mae={:.9} rmse={:.9} max_abs={:.9}",
        g.dx_wrapped.mae, g.dx_wrapped.rmse, g.dx_wrapped.max_abs
    );
    println!(
        "dy raw      mae={:.9} rmse={:.9} max_abs={:.9}",
        g.dy_raw.mae, g.dy_raw.rmse, g.dy_raw.max_abs
    );
    println!(
        "dy wrapped  mae={:.9} rmse={:.9} max_abs={:.9}",
        g.dy_wrapped.mae, g.dy_wrapped.rmse, g.dy_wrapped.max_abs
    );
    Ok(())
}
