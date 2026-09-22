//! Top-level library API: unwrap in-memory rasters without touching the filesystem.
//!
//! The whole unwrapper is one function of data and parameters:
//!
//! ```no_run
//! use snaphu_rs::{CostMode, RunConfig, UnwrapInputs, run_snaphu};
//! use snaphu_rs::data::raster::Raster;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let (width, height) = (64usize, 48usize);
//! # let my_phase = vec![0.0f32; width * height];
//! let wrapped = Raster::new(width, height, my_phase);
//! let config = RunConfig {
//!     cost_mode: CostMode::Smooth,
//!     ..RunConfig::default()
//! };
//!
//! let out = run_snaphu(&UnwrapInputs::new(&wrapped), &config)?;
//! let unwrapped: &[f32] = out.unwrapped_phase.as_slice();
//! # let _ = unwrapped;
//! # Ok(())
//! # }
//! ```
//!
//! [`run_snaphu`] allocates its outputs. [`run_snaphu_inplace`] is the same
//! computation writing into caller-owned buffers, for callers who want to
//! control (or reuse) the output allocations; `run_snaphu` is a thin wrapper
//! around it.
//!
//! # Memory
//!
//! Inputs are borrowed. The only copies this layer makes are the ones the
//! unwrapper needs to own: the wrapped phase (because it is wrapped into
//! `[0, 2π)` in place — set [`UnwrapInputs::wrap_input`] to `false` to skip
//! both the copy and the wrapping if your data is already in that interval),
//! and the magnitude (synthesized, or masked). Power, correlation, the coarse
//! estimate and the arc weights are passed through by reference.
//!
//! # Arc arrays
//!
//! Flows and arc weights use SNAPHU's packed row/column layout: `(rows-1) *
//! cols` row arcs first, then `rows * (cols-1)` column arcs, each block
//! row-major. [`arc_count`] gives the total length.

use crate::config::RunConfig;
use crate::costs::BuildCostArraysResult;
use crate::data::ops::{integrate_phase, non_neg_data_array, valid_data_array, wrap_phase};
use crate::data::raster::Raster;
use crate::data::tile::TileRegion;
use crate::io::reader::RowColTile;
use crate::unwrapping::flow::{UnwrapTileError, UnwrapTileParams, unwrap_tile};
use std::fmt;

/// Number of row arcs in a `rows x cols` grid: `(rows - 1) * cols`.
pub fn row_arc_count(rows: usize, cols: usize) -> usize {
    rows.saturating_sub(1) * cols
}

/// Number of column arcs in a `rows x cols` grid: `rows * (cols - 1)`.
pub fn col_arc_count(rows: usize, cols: usize) -> usize {
    rows * cols.saturating_sub(1)
}

/// Total length of a packed arc array for a `rows x cols` grid.
pub fn arc_count(rows: usize, cols: usize) -> usize {
    row_arc_count(rows, cols) + col_arc_count(rows, cols)
}

/// Widths of the arc "rows" in the jagged representation used internally.
fn arc_row_widths(rows: usize, cols: usize) -> Vec<usize> {
    (0..(2 * rows - 1))
        .map(|row| if row < rows - 1 { cols } else { cols - 1 })
        .collect()
}

fn unpack_arcs(flat: &[i16], rows: usize, cols: usize) -> Result<Vec<Vec<i16>>, SnaphuError> {
    let widths = arc_row_widths(rows, cols);
    let expected = widths.iter().sum::<usize>();
    if flat.len() != expected {
        return Err(SnaphuError::InputLen {
            what: "initial_flows",
            expected,
            got: flat.len(),
        });
    }
    let mut out = Vec::with_capacity(widths.len());
    let mut offset = 0usize;
    for width in widths {
        out.push(flat[offset..offset + width].to_vec());
        offset += width;
    }
    Ok(out)
}

fn pack_arcs_into(
    jagged: &[Vec<i16>],
    rows: usize,
    cols: usize,
    dst: &mut [i16],
) -> Result<(), SnaphuError> {
    let expected = arc_count(rows, cols);
    if dst.len() != expected {
        return Err(SnaphuError::OutputLen {
            what: "flows",
            expected,
            got: dst.len(),
        });
    }
    let mut offset = 0usize;
    for row in jagged {
        dst[offset..offset + row.len()].copy_from_slice(row);
        offset += row.len();
    }
    debug_assert_eq!(offset, expected);
    Ok(())
}

/// Everything that can go wrong in [`run_snaphu`] / [`run_snaphu_inplace`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnaphuError {
    /// The grid is smaller than the 2x2 minimum the unwrapper needs.
    TooSmall { rows: usize, cols: usize },
    /// A raster's `width * height` disagrees with its data length.
    MalformedRaster { what: &'static str },
    /// An optional raster does not match the wrapped-phase shape.
    ShapeMismatch {
        what: &'static str,
        expected: (usize, usize),
        got: (usize, usize),
    },
    /// An input slice has the wrong length for the grid.
    InputLen {
        what: &'static str,
        expected: usize,
        got: usize,
    },
    /// An output buffer has the wrong length for the grid.
    OutputLen {
        what: &'static str,
        expected: usize,
        got: usize,
    },
    /// NaN or infinity in an input raster.
    NonFinite { what: &'static str },
    /// Negative magnitude sample.
    NegativeMagnitude,
    /// Connected components were requested without `RunConfig::regrow_conn_comps`.
    ConnCompsNotRequested,
    /// The requested output is not available in the configured mode.
    Unsupported(&'static str),
    /// The single-tile unwrapper failed.
    Unwrap(UnwrapTileError),
    /// The multi-tile driver failed.
    MultiTile(String),
}

impl fmt::Display for SnaphuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnaphuError::TooSmall { rows, cols } => {
                write!(f, "grid is {rows}x{cols}; at least 2x2 is required")
            }
            SnaphuError::MalformedRaster { what } => {
                write!(f, "{what}: data length does not match width*height")
            }
            SnaphuError::ShapeMismatch {
                what,
                expected,
                got,
            } => write!(
                f,
                "{what} is {}x{} but {}x{} was expected",
                got.0, got.1, expected.0, expected.1
            ),
            SnaphuError::InputLen {
                what,
                expected,
                got,
            } => write!(f, "{what} has length {got}; {expected} expected"),
            SnaphuError::OutputLen {
                what,
                expected,
                got,
            } => write!(
                f,
                "output buffer '{what}' has length {got}; {expected} expected"
            ),
            SnaphuError::NonFinite { what } => write!(f, "NaN or infinity found in {what}"),
            SnaphuError::NegativeMagnitude => write!(f, "negative sample in magnitude data"),
            SnaphuError::ConnCompsNotRequested => write!(
                f,
                "connected components requested, but RunConfig::regrow_conn_comps is false"
            ),
            SnaphuError::Unsupported(what) => write!(f, "unsupported in this mode: {what}"),
            SnaphuError::Unwrap(err) => write!(f, "unwrapping failed: {err:?}"),
            SnaphuError::MultiTile(err) => write!(f, "multi-tile unwrapping failed: {err}"),
        }
    }
}

impl std::error::Error for SnaphuError {}

impl From<UnwrapTileError> for SnaphuError {
    fn from(value: UnwrapTileError) -> Self {
        SnaphuError::Unwrap(value)
    }
}

/// The data half of the unwrapping problem: everything that varies per scene.
///
/// The parameter half is [`RunConfig`].
///
/// All pixel rasters are row-major and must have the same shape as
/// [`wrapped_phase`](Self::wrapped_phase). Arc arrays use the packed layout
/// described in the [module docs](self).
#[derive(Debug, Clone)]
pub struct UnwrapInputs<'a> {
    /// Wrapped interferogram phase, in radians. Required.
    pub wrapped_phase: &'a Raster<f32>,
    /// Interferogram magnitude. Defaults to 1.0 everywhere when absent, which
    /// is what the single-band file formats do.
    pub magnitude: Option<&'a Raster<f32>>,
    /// Brightness/power, already squared if it was amplitude.
    pub power: Option<&'a Raster<f32>>,
    /// Correlation (coherence) in `[0, 1]`.
    pub correlation: Option<&'a Raster<f32>>,
    /// Coarse unwrapped-phase estimate.
    pub unwrapped_estimate: Option<&'a Raster<f32>>,
    /// Scalar arc weights, packed row arcs then column arcs.
    pub arc_weights: Option<&'a RowColTile<i16>>,
    /// Initial flows to refine instead of initializing from residues, packed
    /// row arcs then column arcs. For already-unwrapped input, derive these
    /// with [`crate::data::ops::extract_flow`].
    pub initial_flows: Option<&'a [i16]>,
    /// Byte mask, `rows * cols`. Magnitude is zeroed wherever this is 0.
    pub mask: Option<&'a [i8]>,
    /// Which tiles to process in multi-tile mode, `ntilerow * ntilecol`.
    pub tile_mask: Option<&'a [i8]>,
    /// Wrap [`wrapped_phase`](Self::wrapped_phase) into `[0, 2π)` on entry.
    ///
    /// `true` matches the CLI and accepts any real input. Set it to `false`
    /// only when the samples are already in `[0, 2π)`; that skips a full
    /// `rows * cols` copy.
    pub wrap_input: bool,
}

impl<'a> UnwrapInputs<'a> {
    /// Inputs consisting of nothing but wrapped phase.
    pub fn new(wrapped_phase: &'a Raster<f32>) -> Self {
        Self {
            wrapped_phase,
            magnitude: None,
            power: None,
            correlation: None,
            unwrapped_estimate: None,
            arc_weights: None,
            initial_flows: None,
            mask: None,
            tile_mask: None,
            wrap_input: true,
        }
    }

    /// Grid shape as `(rows, cols)`.
    pub fn shape(&self) -> (usize, usize) {
        (self.wrapped_phase.height, self.wrapped_phase.width)
    }

    fn validate(&self) -> Result<(usize, usize), SnaphuError> {
        let (rows, cols) = self.shape();
        if rows < 2 || cols < 2 {
            return Err(SnaphuError::TooSmall { rows, cols });
        }

        let check = |what: &'static str, raster: Option<&Raster<f32>>| -> Result<(), SnaphuError> {
            let Some(raster) = raster else {
                return Ok(());
            };
            if !raster.has_valid_shape() {
                return Err(SnaphuError::MalformedRaster { what });
            }
            if raster.height != rows || raster.width != cols {
                return Err(SnaphuError::ShapeMismatch {
                    what,
                    expected: (rows, cols),
                    got: (raster.height, raster.width),
                });
            }
            if !valid_data_array(&raster.data, raster.height, raster.width) {
                return Err(SnaphuError::NonFinite { what });
            }
            Ok(())
        };

        check("wrapped_phase", Some(self.wrapped_phase))?;
        check("magnitude", self.magnitude)?;
        check("power", self.power)?;
        check("correlation", self.correlation)?;
        check("unwrapped_estimate", self.unwrapped_estimate)?;

        if let Some(mag) = self.magnitude
            && !non_neg_data_array(&mag.data, mag.height, mag.width)
        {
            return Err(SnaphuError::NegativeMagnitude);
        }

        if let Some(mask) = self.mask
            && mask.len() != rows * cols
        {
            return Err(SnaphuError::InputLen {
                what: "mask",
                expected: rows * cols,
                got: mask.len(),
            });
        }

        if let Some(flows) = self.initial_flows {
            let expected = arc_count(rows, cols);
            if flows.len() != expected {
                return Err(SnaphuError::InputLen {
                    what: "initial_flows",
                    expected,
                    got: flows.len(),
                });
            }
        }

        if let Some(weights) = self.arc_weights {
            let expected_rows = row_arc_count(rows, cols);
            let expected_cols = col_arc_count(rows, cols);
            if weights.row_arcs.len() != expected_rows {
                return Err(SnaphuError::InputLen {
                    what: "arc_weights.row_arcs",
                    expected: expected_rows,
                    got: weights.row_arcs.len(),
                });
            }
            if weights.col_arcs.len() != expected_cols {
                return Err(SnaphuError::InputLen {
                    what: "arc_weights.col_arcs",
                    expected: expected_cols,
                    got: weights.col_arcs.len(),
                });
            }
        }

        Ok((rows, cols))
    }
}

/// What a run produced, for callers that let the library allocate.
#[derive(Debug, Clone, PartialEq)]
pub struct UnwrapOutputs {
    /// Unwrapped phase, row-major, `rows x cols`.
    pub unwrapped_phase: Raster<f32>,
    /// The magnitude actually used (supplied, synthesized, or masked; in
    /// multi-tile mode, the assembled magnitude).
    pub magnitude: Raster<f32>,
    /// Solved flows in packed arc layout. `None` in multi-tile mode.
    pub flows: Option<Vec<i16>>,
    /// Connected-component labels, when `RunConfig::regrow_conn_comps` is set.
    pub conn_comps: Option<Raster<u32>>,
    /// Cost arrays and topo diagnostics. `None` in multi-tile mode.
    pub diagnostics: Option<BuildCostArraysResult>,
}

/// Summary returned by [`run_snaphu_inplace`].
#[derive(Debug, Clone, PartialEq)]
pub struct UnwrapReport {
    pub rows: usize,
    pub cols: usize,
    pub multi_tile: bool,
    pub flows_written: bool,
    pub conn_comps_written: bool,
    pub magnitude_written: bool,
    /// Cost arrays and topo diagnostics, moved out of the solver. `None` in
    /// multi-tile mode.
    pub diagnostics: Option<BuildCostArraysResult>,
}

/// Unwrap `inputs` under `config`, allocating the outputs.
///
/// A thin wrapper over [`run_snaphu_inplace`].
pub fn run_snaphu(
    inputs: &UnwrapInputs<'_>,
    config: &RunConfig,
) -> Result<UnwrapOutputs, SnaphuError> {
    let (rows, cols) = inputs.validate()?;
    let multi_tile = config.ntilerow != 1 || config.ntilecol != 1;

    let mut unwrapped_phase = vec![0.0f32; rows * cols];
    let mut magnitude = vec![0.0f32; rows * cols];
    let mut flows = (!multi_tile).then(|| vec![0i16; arc_count(rows, cols)]);
    let mut conn_comps = config.regrow_conn_comps.then(|| vec![0u32; rows * cols]);

    let report = run_snaphu_inplace(
        inputs,
        config,
        &mut unwrapped_phase,
        flows.as_deref_mut(),
        conn_comps.as_deref_mut(),
        Some(&mut magnitude),
    )?;

    Ok(UnwrapOutputs {
        unwrapped_phase: Raster::new(cols, rows, unwrapped_phase),
        magnitude: Raster::new(cols, rows, magnitude),
        flows: report.flows_written.then_some(flows).flatten(),
        conn_comps: report
            .conn_comps_written
            .then_some(conn_comps)
            .flatten()
            .map(|data| Raster::new(cols, rows, data)),
        diagnostics: report.diagnostics,
    })
}

/// Unwrap `inputs` under `config`, writing into caller-owned buffers.
///
/// Every output buffer must be exactly the right length: `rows * cols` for
/// `unwrapped_phase`, `magnitude` and `conn_comps`, and [`arc_count`] for
/// `flows`. Optional buffers can be `None` when not wanted; asking for one the
/// configuration cannot produce is an error rather than a silent no-op.
pub fn run_snaphu_inplace(
    inputs: &UnwrapInputs<'_>,
    config: &RunConfig,
    unwrapped_phase: &mut [f32],
    flows: Option<&mut [i16]>,
    conn_comps: Option<&mut [u32]>,
    magnitude: Option<&mut [f32]>,
) -> Result<UnwrapReport, SnaphuError> {
    let (rows, cols) = inputs.validate()?;
    let pixels = rows * cols;
    if unwrapped_phase.len() != pixels {
        return Err(SnaphuError::OutputLen {
            what: "unwrapped_phase",
            expected: pixels,
            got: unwrapped_phase.len(),
        });
    }
    if let Some(buf) = magnitude.as_deref()
        && buf.len() != pixels
    {
        return Err(SnaphuError::OutputLen {
            what: "magnitude",
            expected: pixels,
            got: buf.len(),
        });
    }
    if let Some(buf) = conn_comps.as_deref() {
        if !config.regrow_conn_comps {
            return Err(SnaphuError::ConnCompsNotRequested);
        }
        if buf.len() != pixels {
            return Err(SnaphuError::OutputLen {
                what: "conn_comps",
                expected: pixels,
                got: buf.len(),
            });
        }
    }

    // Magnitude: supplied, or synthesized as unity like the single-band formats.
    let mut mag = match inputs.magnitude {
        Some(mag) => mag.clone(),
        None => Raster::new(cols, rows, vec![1.0f32; pixels]),
    };
    if let Some(mask) = inputs.mask {
        for (sample, &flag) in mag.data.iter_mut().zip(mask) {
            if flag == 0 {
                *sample = 0.0;
            }
        }
    }

    // Wrapped phase: copied only when this layer has to wrap it.
    let wrapped_owned = inputs.wrap_input.then(|| {
        let mut copy = inputs.wrapped_phase.clone();
        wrap_phase(&mut copy.data, rows, cols);
        copy
    });
    let wrapped = wrapped_owned.as_ref().unwrap_or(inputs.wrapped_phase);

    let multi_tile = config.ntilerow != 1 || config.ntilecol != 1;
    let report = if multi_tile {
        if flows.is_some() {
            return Err(SnaphuError::Unsupported("flows output in multi-tile mode"));
        }
        if conn_comps.is_some() {
            return Err(SnaphuError::Unsupported(
                "connected components in multi-tile mode",
            ));
        }
        run_multi_tile_into(
            inputs,
            config,
            &mut mag,
            wrapped,
            rows,
            cols,
            unwrapped_phase,
        )?
    } else {
        run_single_tile_into(
            inputs,
            config,
            &mag,
            wrapped,
            rows,
            cols,
            unwrapped_phase,
            flows,
            conn_comps,
        )?
    };

    let magnitude_written = if let Some(buf) = magnitude {
        buf.copy_from_slice(&mag.data);
        true
    } else {
        false
    };

    Ok(UnwrapReport {
        magnitude_written,
        ..report
    })
}

#[allow(clippy::too_many_arguments)]
fn run_single_tile_into(
    inputs: &UnwrapInputs<'_>,
    config: &RunConfig,
    mag: &Raster<f32>,
    wrapped: &Raster<f32>,
    rows: usize,
    cols: usize,
    unwrapped_phase: &mut [f32],
    flows: Option<&mut [i16]>,
    conn_comps: Option<&mut [u32]>,
) -> Result<UnwrapReport, SnaphuError> {
    let initial_flows = inputs
        .initial_flows
        .map(|flat| unpack_arcs(flat, rows, cols))
        .transpose()?;

    let result = unwrap_tile(
        UnwrapTileParams {
            tile: TileRegion::new(0, 0, rows, cols),
            mag,
            wrapped_phase: wrapped,
            power: inputs.power,
            correlation: inputs.correlation,
            unwrapped_estimate: inputs.unwrapped_estimate,
            arc_weights: inputs.arc_weights,
            initial_flows,
            cost_threshold: config.tile_cost_threshold,
            min_region_size: config.min_region_size,
            max_components: config.max_conn_comps.unwrap_or(rows * cols).max(1),
        },
        config,
    )?;

    let mut flat = vec![0i16; arc_count(rows, cols)];
    pack_arcs_into(&result.flows, rows, cols, &mut flat)?;

    let integrated = integrate_phase(&wrapped.data, &flat, rows, cols);
    unwrapped_phase.copy_from_slice(&integrated);

    let flows_written = if let Some(buf) = flows {
        if buf.len() != flat.len() {
            return Err(SnaphuError::OutputLen {
                what: "flows",
                expected: flat.len(),
                got: buf.len(),
            });
        }
        buf.copy_from_slice(&flat);
        true
    } else {
        false
    };

    let conn_comps_written = if let Some(buf) = conn_comps {
        let mask = result
            .conn_comp_mask
            .as_ref()
            .ok_or(SnaphuError::ConnCompsNotRequested)?;
        for (row_index, row) in mask.iter().enumerate() {
            let start = row_index * cols;
            buf[start..start + cols].copy_from_slice(row);
        }
        true
    } else {
        false
    };

    Ok(UnwrapReport {
        rows,
        cols,
        multi_tile: false,
        flows_written,
        conn_comps_written,
        magnitude_written: false,
        diagnostics: Some(result.cost_arrays),
    })
}

fn run_multi_tile_into(
    inputs: &UnwrapInputs<'_>,
    config: &RunConfig,
    mag: &mut Raster<f32>,
    wrapped: &Raster<f32>,
    rows: usize,
    cols: usize,
    unwrapped_phase: &mut [f32],
) -> Result<UnwrapReport, SnaphuError> {
    use crate::unwrapping::multitile::{MultiTileRunParams, run_multi_tile};

    fn to_grid(raster: &Raster<f32>) -> Vec<Vec<f32>> {
        raster
            .data
            .chunks_exact(raster.width)
            .map(<[f32]>::to_vec)
            .collect()
    }

    let mag_grid = to_grid(&*mag);
    let wrapped_grid = to_grid(wrapped);
    let power_grid = inputs.power.map(to_grid);
    let corr_grid = inputs.correlation.map(to_grid);

    let integrated = run_multi_tile(MultiTileRunParams {
        mag_grid: &mag_grid,
        wrapped_grid: &wrapped_grid,
        power_grid: power_grid.as_deref(),
        corr_grid: corr_grid.as_deref(),
        tile_mask: inputs.tile_mask,
        nlines: rows,
        linelen: cols,
        params: config,
    })
    .map_err(|err| SnaphuError::MultiTile(err.to_string()))?;

    for (row_index, row) in integrated.unw_phase.iter().enumerate() {
        let start = row_index * cols;
        unwrapped_phase[start..start + cols].copy_from_slice(row);
    }
    // Tile assembly produces its own magnitude raster; report that one.
    for (row_index, row) in integrated.mag.iter().enumerate() {
        let start = row_index * cols;
        mag.data[start..start + cols].copy_from_slice(row);
    }

    Ok(UnwrapReport {
        rows,
        cols,
        multi_tile: true,
        flows_written: false,
        conn_comps_written: false,
        magnitude_written: false,
        diagnostics: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CostMode;
    use std::f32::consts::TAU;

    fn ramp(rows: usize, cols: usize) -> Raster<f32> {
        let mut data = Vec::with_capacity(rows * cols);
        for row in 0..rows {
            for col in 0..cols {
                let phase = 0.35 * col as f32 + 0.21 * row as f32;
                data.push(phase - TAU * (phase / TAU).floor());
            }
        }
        Raster::new(cols, rows, data)
    }

    fn smooth_config() -> RunConfig {
        RunConfig {
            cost_mode: CostMode::Smooth,
            ..RunConfig::default()
        }
    }

    #[test]
    fn run_snaphu_unwraps_a_linear_ramp() {
        let wrapped = ramp(12, 16);
        let out = run_snaphu(&UnwrapInputs::new(&wrapped), &smooth_config()).unwrap();

        assert_eq!(out.unwrapped_phase.height, 12);
        assert_eq!(out.unwrapped_phase.width, 16);
        assert_eq!(out.magnitude.data, vec![1.0f32; 12 * 16]);
        assert_eq!(out.flows.as_ref().unwrap().len(), arc_count(12, 16));
        assert!(out.conn_comps.is_none());
        assert!(out.diagnostics.is_some());

        // Unwrapping may only add whole cycles to the wrapped input.
        for (got, want) in out.unwrapped_phase.data.iter().zip(&wrapped.data) {
            let cycles = (got - want) / TAU;
            assert!((cycles - cycles.round()).abs() < 1.0e-3);
        }
    }

    #[test]
    fn inplace_matches_the_allocating_wrapper() {
        let wrapped = ramp(10, 9);
        let config = smooth_config();
        let owned = run_snaphu(&UnwrapInputs::new(&wrapped), &config).unwrap();

        let mut phase = vec![0.0f32; 10 * 9];
        let mut flows = vec![0i16; arc_count(10, 9)];
        let mut mag = vec![0.0f32; 10 * 9];
        let report = run_snaphu_inplace(
            &UnwrapInputs::new(&wrapped),
            &config,
            &mut phase,
            Some(&mut flows),
            None,
            Some(&mut mag),
        )
        .unwrap();

        assert_eq!((report.rows, report.cols), (10, 9));
        assert!(report.flows_written && report.magnitude_written);
        assert!(!report.conn_comps_written && !report.multi_tile);
        assert_eq!(phase, owned.unwrapped_phase.data);
        assert_eq!(flows, owned.flows.unwrap());
        assert_eq!(mag, owned.magnitude.data);
    }

    #[test]
    fn pre_wrapped_input_skips_wrapping_and_matches() {
        let wrapped = ramp(8, 8); // already in [0, 2pi)
        let config = smooth_config();
        let with_wrap = run_snaphu(&UnwrapInputs::new(&wrapped), &config).unwrap();

        let mut inputs = UnwrapInputs::new(&wrapped);
        inputs.wrap_input = false;
        let without_wrap = run_snaphu(&inputs, &config).unwrap();

        assert_eq!(
            without_wrap.unwrapped_phase.data,
            with_wrap.unwrapped_phase.data
        );
    }

    #[test]
    fn initial_flows_round_trip_through_the_packed_layout() {
        let wrapped = ramp(6, 7);
        let config = smooth_config();
        let first = run_snaphu(&UnwrapInputs::new(&wrapped), &config).unwrap();
        let flows = first.flows.clone().unwrap();

        let mut inputs = UnwrapInputs::new(&wrapped);
        inputs.initial_flows = Some(&flows);
        let second = run_snaphu(&inputs, &config).unwrap();

        assert_eq!(second.flows.unwrap(), flows);
        assert_eq!(
            second.unwrapped_phase.data, first.unwrapped_phase.data,
            "re-running from the solved flows must be a fixed point"
        );
    }

    #[test]
    fn conn_comps_require_the_config_flag() {
        let wrapped = ramp(6, 6);
        let mut phase = vec![0.0f32; 36];
        let mut comps = vec![0u32; 36];
        let err = run_snaphu_inplace(
            &UnwrapInputs::new(&wrapped),
            &smooth_config(),
            &mut phase,
            None,
            Some(&mut comps),
            None,
        )
        .unwrap_err();
        assert_eq!(err, SnaphuError::ConnCompsNotRequested);

        let config = RunConfig {
            regrow_conn_comps: true,
            ..smooth_config()
        };
        let out = run_snaphu(&UnwrapInputs::new(&wrapped), &config).unwrap();
        assert_eq!(out.conn_comps.unwrap().data.len(), 36);
    }

    #[test]
    fn output_buffers_must_be_exactly_sized() {
        let wrapped = ramp(5, 5);
        let mut short = vec![0.0f32; 24];
        let err = run_snaphu_inplace(
            &UnwrapInputs::new(&wrapped),
            &smooth_config(),
            &mut short,
            None,
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            SnaphuError::OutputLen {
                what: "unwrapped_phase",
                expected: 25,
                got: 24
            }
        );
    }

    #[test]
    fn mismatched_auxiliary_rasters_are_rejected() {
        let wrapped = ramp(6, 6);
        let corr = Raster::new(5, 6, vec![0.5f32; 30]);
        let mut inputs = UnwrapInputs::new(&wrapped);
        inputs.correlation = Some(&corr);
        let err = run_snaphu(&inputs, &smooth_config()).unwrap_err();
        assert_eq!(
            err,
            SnaphuError::ShapeMismatch {
                what: "correlation",
                expected: (6, 6),
                got: (6, 5)
            }
        );
    }

    #[test]
    fn non_finite_input_is_rejected() {
        let mut wrapped = ramp(4, 4);
        wrapped.data[5] = f32::NAN;
        let err = run_snaphu(&UnwrapInputs::new(&wrapped), &smooth_config()).unwrap_err();
        assert_eq!(
            err,
            SnaphuError::NonFinite {
                what: "wrapped_phase"
            }
        );
    }

    #[test]
    fn grids_smaller_than_2x2_are_rejected() {
        let wrapped = Raster::new(1, 4, vec![0.0f32; 4]);
        let err = run_snaphu(&UnwrapInputs::new(&wrapped), &smooth_config()).unwrap_err();
        assert_eq!(err, SnaphuError::TooSmall { rows: 4, cols: 1 });
    }

    #[test]
    fn mask_zeroes_magnitude() {
        let wrapped = ramp(4, 4);
        let mag = Raster::new(4, 4, vec![2.0f32; 16]);
        let mut mask = vec![1i8; 16];
        mask[0] = 0;
        mask[15] = 0;

        let mut inputs = UnwrapInputs::new(&wrapped);
        inputs.magnitude = Some(&mag);
        inputs.mask = Some(&mask);
        let out = run_snaphu(&inputs, &smooth_config()).unwrap();

        assert_eq!(out.magnitude.data[0], 0.0);
        assert_eq!(out.magnitude.data[15], 0.0);
        assert_eq!(out.magnitude.data[1], 2.0);
    }

    #[test]
    fn arc_counts_match_the_packed_layout() {
        assert_eq!(row_arc_count(4, 5), 15);
        assert_eq!(col_arc_count(4, 5), 16);
        assert_eq!(arc_count(4, 5), 31);
        assert_eq!(arc_row_widths(4, 5).iter().sum::<usize>(), arc_count(4, 5));
    }
}
