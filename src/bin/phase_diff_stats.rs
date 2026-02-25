//! Compute direct and modulo-2pi error statistics between two phase rasters.
//!
//! Usage:
//! `cargo run --bin phase_diff_stats -- -W 600 -H 600 c.img rust.img`

use clap::Parser;
use snaphu_rs::phase_compare::{
    diff_mod_k, error_stats, error_stats_from_values, k_histogram, read_f32_raster,
};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Compare two phase rasters with raw and modulo-2pi metrics")]
struct Args {
    #[arg(short = 'W', long)]
    width: usize,
    #[arg(short = 'H', long)]
    height: usize,
    #[arg(long, default_value_t = 12)]
    hist_top: usize,
    a: PathBuf,
    b: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let a = read_f32_raster(&args.a, args.width, args.height)?;
    let b = read_f32_raster(&args.b, args.width, args.height)?;
    let raw = error_stats(&a, &b);
    let (_diff, mod_res, k) = diff_mod_k(&a, &b);
    let mod_stats = error_stats_from_values(&mod_res);

    println!("pixels: {}", a.len());
    println!(
        "raw   mae={:.9} rmse={:.9} max_abs={:.9}",
        raw.mae, raw.rmse, raw.max_abs
    );
    println!(
        "mod2pi mae={:.9} rmse={:.9} max_abs={:.9}",
        mod_stats.mae, mod_stats.rmse, mod_stats.max_abs
    );

    let k0 = k.iter().filter(|&&v| v == 0).count();
    println!("k==0 fraction: {:.6}", k0 as f64 / k.len() as f64);

    let hist = k_histogram(&k);
    let mut pairs = hist.into_iter().collect::<Vec<_>>();
    pairs.sort_by(|lhs, rhs| rhs.1.cmp(&lhs.1));
    println!("top {} cycle bins (k = round((b-a)/2pi)):", args.hist_top);
    for (i, (kv, ct)) in pairs.into_iter().take(args.hist_top).enumerate() {
        println!(
            "  {:2}. k={:>4} count={} frac={:.6}",
            i + 1,
            kv,
            ct,
            ct as f64 / k.len() as f64
        );
    }

    Ok(())
}
