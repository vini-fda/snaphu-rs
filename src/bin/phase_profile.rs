//! Dump row/column phase profiles to CSV for pointwise inspection.
//!
//! Output columns:
//! `axis,index,position,a,b,diff,mod_diff,k`
//!
//! Usage:
//! `cargo run --bin phase_profile -- -W 600 -H 600 c.img rust.img --row 300 --col 300 --out profile.csv`

use clap::Parser;
use snaphu_rs::phase_compare::{TWO_PI, read_f32_raster};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Write row/column line profiles comparing two phase rasters")]
struct Args {
    #[arg(short = 'W', long)]
    width: usize,
    #[arg(short = 'H', long)]
    height: usize,
    #[arg(long = "row")]
    rows: Vec<usize>,
    #[arg(long = "col")]
    cols: Vec<usize>,
    #[arg(long)]
    out: Option<PathBuf>,
    a: PathBuf,
    b: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let a = read_f32_raster(&args.a, args.width, args.height)?;
    let b = read_f32_raster(&args.b, args.width, args.height)?;

    let mut rows = args.rows;
    let mut cols = args.cols;
    if rows.is_empty() && cols.is_empty() {
        rows.push(args.height / 2);
        cols.push(args.width / 2);
    }
    rows.retain(|&r| r < args.height);
    cols.retain(|&c| c < args.width);

    let mut output: Box<dyn Write> = match args.out {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(std::io::stdout()),
    };
    writeln!(output, "axis,index,position,a,b,diff,mod_diff,k")?;

    for &r in &rows {
        for c in 0..args.width {
            let idx = r * args.width + c;
            let av = f64::from(a[idx]);
            let bv = f64::from(b[idx]);
            let d = bv - av;
            let k = (d / TWO_PI).round() as i32;
            let mod_d = d - f64::from(k) * TWO_PI;
            writeln!(output, "row,{r},{c},{av:.9},{bv:.9},{d:.9},{mod_d:.9},{k}")?;
        }
    }

    for &c in &cols {
        for r in 0..args.height {
            let idx = r * args.width + c;
            let av = f64::from(a[idx]);
            let bv = f64::from(b[idx]);
            let d = bv - av;
            let k = (d / TWO_PI).round() as i32;
            let mod_d = d - f64::from(k) * TWO_PI;
            writeln!(output, "col,{c},{r},{av:.9},{bv:.9},{d:.9},{mod_d:.9},{k}")?;
        }
    }

    Ok(())
}
