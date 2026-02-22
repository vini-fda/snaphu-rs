#![allow(dead_code)]

//! Flow-based unwrapping modes.

use crate::config::{InputFiles, OutputFiles, RunConfig};
use crate::unwrapping::tiles::set_tile_init_outfile;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileJob {
    pub tilerow: usize,
    pub tilecol: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwrapIterationPlan {
    pub index: usize,
    pub read_from_tile_init: bool,
    pub write_tile_init: bool,
    pub tiles: Vec<TileJob>,
    pub assemble_tiles: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwrapPlan {
    pub iterations: Vec<UnwrapIterationPlan>,
    pub tile_init_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnwrapError {
    InvalidImageDims,
    InvalidTileGrid,
    InvalidOutfile,
}

/// Build the tile/iteration execution plan for unwrapping.
///
/// This is the typed Rust equivalent of C `Unwrap()`.
pub fn unwrap(
    infiles: &InputFiles,
    outfiles: &OutputFiles,
    params: &RunConfig,
    linelen: usize,
    nlines: usize,
) -> Result<UnwrapPlan, UnwrapError> {
    if linelen == 0 || nlines == 0 {
        return Err(UnwrapError::InvalidImageDims);
    }
    if params.ntilerow == 0 || params.ntilecol == 0 {
        return Err(UnwrapError::InvalidTileGrid);
    }
    if outfiles.outfile.is_empty() {
        return Err(UnwrapError::InvalidOutfile);
    }

    let _ = infiles;
    let noptiter = if params.onetilereopt { 2 } else { 1 };
    let mut tile_init_file = None;
    let mut iterations = Vec::with_capacity(noptiter);

    for optiter in 0..noptiter {
        let mut iter_params = params.clone();
        let mut read_from_tile_init = false;
        let mut write_tile_init = false;

        if optiter == 0 && noptiter > 1 {
            let init_path =
                set_tile_init_outfile(Path::new(&outfiles.outfile), params.parent_pid as u32)
                    .map_err(|_| UnwrapError::InvalidOutfile)?;
            tile_init_file = Some(init_path.to_string_lossy().to_string());
            write_tile_init = true;
        } else if optiter == 1 {
            read_from_tile_init = true;
            iter_params.unwrapped = true;
            iter_params.ntilerow = 1;
            iter_params.ntilecol = 1;
            iter_params.rowovrlp = 0;
            iter_params.colovrlp = 0;
        }

        let mut tiles = Vec::new();
        if iter_params.ntilerow == 1 && iter_params.ntilecol == 1 {
            tiles.push(TileJob {
                tilerow: 0,
                tilecol: 0,
            });
        } else if !iter_params.assemble_only {
            for tilerow in 0..iter_params.ntilerow {
                for tilecol in 0..iter_params.ntilecol {
                    tiles.push(TileJob { tilerow, tilecol });
                }
            }
        }

        iterations.push(UnwrapIterationPlan {
            index: optiter,
            read_from_tile_init,
            write_tile_init,
            tiles,
            assemble_tiles: iter_params.ntilerow != 1 || iter_params.ntilecol != 1,
        });
    }

    Ok(UnwrapPlan {
        iterations,
        tile_init_file,
    })
}

pub fn compute_flow() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwrap_single_tile_has_one_iteration_one_tile() {
        let infiles = InputFiles::default();
        let outfiles = OutputFiles::default();
        let params = RunConfig::default();

        let plan = unwrap(&infiles, &outfiles, &params, 32, 32).unwrap();
        assert_eq!(plan.iterations.len(), 1);
        assert_eq!(plan.iterations[0].tiles.len(), 1);
        assert!(!plan.iterations[0].assemble_tiles);
    }

    #[test]
    fn unwrap_with_one_tile_reopt_has_two_iterations() {
        let infiles = InputFiles::default();
        let outfiles = OutputFiles::default();
        let params = RunConfig {
            onetilereopt: true,
            ntilerow: 2,
            ntilecol: 2,
            ..RunConfig::default()
        };

        let plan = unwrap(&infiles, &outfiles, &params, 64, 64).unwrap();
        assert_eq!(plan.iterations.len(), 2);
        assert!(plan.iterations[0].write_tile_init);
        assert!(plan.iterations[1].read_from_tile_init);
    }
}
