//! Build and summarize the integer-cycle map k = round((b-a)/2pi).
//!
//! Optional outputs:
//! - `--write-k` writes k as float32 raster (for plotting/viewers).
//! - `--write-row-median-csv` and `--write-col-median-csv` dump medians.
//!
//! Usage:
//! `cargo run --bin phase_k_map -- -W 600 -H 600 c.img rust.img --write-k k.img`

use clap::Parser;
use snaphu_rs::phase_compare::{diff_mod_k, read_f32_raster, summarize_k_map, write_f32_raster};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Summarize integer-cycle offsets between two phase rasters")]
struct Args {
    #[arg(short = 'W', long)]
    width: usize,
    #[arg(short = 'H', long)]
    height: usize,
    #[arg(long)]
    write_k: Option<PathBuf>,
    #[arg(long)]
    write_row_median_csv: Option<PathBuf>,
    #[arg(long)]
    write_col_median_csv: Option<PathBuf>,
    a: PathBuf,
    b: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let a = read_f32_raster(&args.a, args.width, args.height)?;
    let b = read_f32_raster(&args.b, args.width, args.height)?;
    let (_diff, _mod_res, k) = diff_mod_k(&a, &b);

    let summary = summarize_k_map(&k, args.width, args.height);
    let row_min = summary
        .row_medians
        .iter()
        .fold(f64::INFINITY, |acc, &v| acc.min(v));
    let row_max = summary
        .row_medians
        .iter()
        .fold(f64::NEG_INFINITY, |acc, &v| acc.max(v));
    let col_min = summary
        .col_medians
        .iter()
        .fold(f64::INFINITY, |acc, &v| acc.min(v));
    let col_max = summary
        .col_medians
        .iter()
        .fold(f64::NEG_INFINITY, |acc, &v| acc.max(v));

    println!("pixels: {}", k.len());
    println!(
        "k global mean={:.6} std={:.6}",
        summary.global_mean, summary.global_std
    );
    println!(
        "k row variance mean={:.6}, col variance mean={:.6}",
        summary.mean_row_variance, summary.mean_col_variance
    );
    println!("row median range: [{:.3}, {:.3}]", row_min, row_max);
    println!("col median range: [{:.3}, {:.3}]", col_min, col_max);

    if let Some(path) = args.write_k.as_ref() {
        let k_f32 = k.iter().map(|&v| v as f32).collect::<Vec<_>>();
        write_f32_raster(path, &k_f32)?;
        println!("wrote k-map float raster: {}", path.display());
    }

    if let Some(path) = args.write_row_median_csv.as_ref() {
        let mut f = File::create(path)?;
        writeln!(f, "row,median_k")?;
        for (r, &m) in summary.row_medians.iter().enumerate() {
            writeln!(f, "{r},{m}")?;
        }
        println!("wrote row medians: {}", path.display());
    }

    if let Some(path) = args.write_col_median_csv.as_ref() {
        let mut f = File::create(path)?;
        writeln!(f, "col,median_k")?;
        for (c, &m) in summary.col_medians.iter().enumerate() {
            writeln!(f, "{c},{m}")?;
        }
        println!("wrote col medians: {}", path.display());
    }

    Ok(())
}
