//! Shared cost record structs translated from the SNAPHU leaf types.

use crate::constants::LARGE_SHORT;

/// Equivalent to SNAPHU's `costT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cost {
    pub offset: i16,
    pub sigma_sq: i16,
    pub dz_max: i16,
    pub lay_cost: i16,
}

impl Cost {
    pub fn new(offset: i16, sigma_sq: i16, dz_max: i16, lay_cost: i16) -> Self {
        Self {
            offset,
            sigma_sq,
            dz_max,
            lay_cost,
        }
    }

    pub fn mask(&mut self) {
        self.lay_cost = 0;
        self.offset = LARGE_SHORT / 2;
        self.dz_max = LARGE_SHORT;
        self.sigma_sq = LARGE_SHORT;
    }
}

pub fn mask_cost(cost: &mut Cost) {
    cost.mask();
}

impl Default for Cost {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

/// Equivalent to SNAPHU's `smoothcostT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmoothCost {
    pub offset: i16,
    pub sigma_sq: i16,
}

impl SmoothCost {
    pub fn new(offset: i16, sigma_sq: i16) -> Self {
        Self { offset, sigma_sq }
    }

    pub fn mask(&mut self) {
        self.offset = LARGE_SHORT / 2;
        self.sigma_sq = LARGE_SHORT;
    }
}

pub fn mask_smooth_cost(cost: &mut SmoothCost) {
    cost.mask();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaskWeightsError {
    LengthMismatch {
        kind: &'static str,
        expected: usize,
        actual: usize,
    },
}

fn expect_length(
    kind: &'static str,
    expected: usize,
    actual: usize,
) -> Result<(), MaskWeightsError> {
    if expected == actual {
        Ok(())
    } else {
        Err(MaskWeightsError::LengthMismatch {
            kind,
            expected,
            actual,
        })
    }
}

pub fn mask_prespecified_arc_costs<T>(
    row_costs: &mut [T],
    col_costs: &mut [T],
    row_weights: &[i16],
    col_weights: &[i16],
    nrow: usize,
    ncol: usize,
    mask_fn: fn(&mut T),
) -> Result<(), MaskWeightsError> {
    let row_rows = nrow.saturating_sub(1);
    let col_cols = ncol.saturating_sub(1);
    let expected_row_len = row_rows.saturating_mul(ncol);
    let expected_col_len = nrow.saturating_mul(col_cols);
    expect_length("row_costs", expected_row_len, row_costs.len())?;
    expect_length("col_costs", expected_col_len, col_costs.len())?;
    expect_length("row_weights", expected_row_len, row_weights.len())?;
    expect_length("col_weights", expected_col_len, col_weights.len())?;

    for r in 0..row_rows {
        for c in 0..ncol {
            let idx = r * ncol + c;
            if row_weights[idx] == 0 {
                mask_fn(&mut row_costs[idx]);
            }
        }
    }

    for r in 0..nrow {
        for c in 0..col_cols {
            let idx = r * col_cols + c;
            if col_weights[idx] == 0 {
                mask_fn(&mut col_costs[idx]);
            }
        }
    }

    Ok(())
}

impl Default for SmoothCost {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Incremental cost record for a single arc.
///
/// Equivalent to the C `incrcostT` struct. Each arc in the network flow graph
/// stores the cost of incrementing flow in the positive direction (`poscost`)
/// and the negative direction (`negcost`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IncrCost {
    pub poscost: i16,
    pub negcost: i16,
}

impl IncrCost {
    pub fn new(poscost: i16, negcost: i16) -> Self {
        Self { poscost, negcost }
    }

    /// Returns the incremental cost for the given arc direction.
    ///
    /// This is the idiomatic Rust equivalent of the C `GetCost()` function.
    /// If `arcdir` is positive the forward (positive) cost is returned;
    /// otherwise the reverse (negative) cost is returned.
    #[inline]
    pub fn get_cost(&self, arcdir: i64) -> i16 {
        if arcdir > 0 {
            self.poscost
        } else {
            self.negcost
        }
    }
}

/// Smooth region-growing incremental costs and store results in `negcost`.
///
/// Equivalent to C `ThickenCosts()`. The function convolves `poscost` values
/// along row arcs and column arcs separately and writes the smoothed value to
/// `negcost`, clipping to `LARGE_SHORT` when needed. Returns the maximum
/// `negcost` written.
pub fn thicken_costs(incrcosts: &mut [IncrCost], nrow: usize, ncol: usize) -> i64 {
    assert!(nrow >= 1 && ncol >= 1, "dimensions must be at least 1x1");
    let row_arc_count = (nrow - 1) * ncol;
    let col_arc_count = nrow * (ncol - 1);
    assert_eq!(
        incrcosts.len(),
        row_arc_count + col_arc_count,
        "incrcosts length does not match row-col layout"
    );

    let idx = |arcrow: usize, arccol: usize| -> usize {
        if arcrow < nrow - 1 {
            arcrow * ncol + arccol
        } else {
            row_arc_count + (arcrow - (nrow - 1)) * (ncol - 1) + arccol
        }
    };

    let mut maxcost = i64::MIN;

    for row in 0..(nrow - 1) {
        for col in 0..ncol {
            let mut accum = 2_i64 * incrcosts[idx(row, col)].poscost as i64;
            let mut n = 2.0;
            if col != 0 {
                accum += incrcosts[idx(row, col - 1)].poscost as i64;
                n += 1.0;
            }
            if col != ncol - 1 {
                accum += incrcosts[idx(row, col + 1)].poscost as i64;
                n += 1.0;
            }
            let mut thick = (accum as f64 / n).round_ties_even() as i64;
            if thick > LARGE_SHORT as i64 {
                thick = LARGE_SHORT as i64;
            }
            incrcosts[idx(row, col)].negcost = thick as i16;
            if thick > maxcost {
                maxcost = thick;
            }
        }
    }

    for row in (nrow - 1)..(2 * nrow - 1) {
        for col in 0..(ncol - 1) {
            let mut accum = 2_i64 * incrcosts[idx(row, col)].poscost as i64;
            let mut n = 2.0;
            if row != nrow - 1 {
                accum += incrcosts[idx(row - 1, col)].poscost as i64;
                n += 1.0;
            }
            if row != 2 * nrow - 2 {
                accum += incrcosts[idx(row + 1, col)].poscost as i64;
                n += 1.0;
            }
            let mut thick = (accum as f64 / n).round_ties_even() as i64;
            if thick > LARGE_SHORT as i64 {
                thick = LARGE_SHORT as i64;
            }
            incrcosts[idx(row, col)].negcost = thick as i16;
            if thick > maxcost {
                maxcost = thick;
            }
        }
    }

    maxcost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_mask_sets_expected_sentinels() {
        let mut cost = Cost::new(5, 10, 15, 20);
        mask_cost(&mut cost);
        assert_eq!(cost.lay_cost, 0);
        assert_eq!(cost.offset, LARGE_SHORT / 2);
        assert_eq!(cost.dz_max, LARGE_SHORT);
        assert_eq!(cost.sigma_sq, LARGE_SHORT);
    }

    #[test]
    fn smooth_cost_mask_sets_expected_sentinels() {
        let mut cost = SmoothCost::new(-10, 42);
        mask_smooth_cost(&mut cost);
        assert_eq!(cost.offset, LARGE_SHORT / 2);
        assert_eq!(cost.sigma_sq, LARGE_SHORT);
    }

    #[test]
    fn mask_prespecified_costs_handles_row_and_col_grids() {
        let nrow = 3;
        let ncol = 4;
        let row_len = (nrow - 1) * ncol;
        let col_len = nrow * (ncol - 1);
        let mut row_costs: Vec<Cost> = (0..row_len)
            .map(|i| Cost::new(i as i16, i as i16, i as i16, i as i16))
            .collect();
        let mut col_costs: Vec<Cost> = (0..col_len)
            .map(|i| Cost::new(i as i16, i as i16, i as i16, i as i16))
            .collect();
        let mut row_weights = vec![1i16; row_len];
        row_weights[1] = 0;
        row_weights[row_len - 2] = 0;
        let mut col_weights = vec![1i16; col_len];
        col_weights[0] = 0;
        col_weights[col_len - 1] = 0;

        mask_prespecified_arc_costs(
            &mut row_costs,
            &mut col_costs,
            &row_weights,
            &col_weights,
            nrow,
            ncol,
            Cost::mask,
        )
        .unwrap();

        assert_eq!(row_costs[1].sigma_sq, LARGE_SHORT);
        assert_eq!(row_costs[row_len - 2].sigma_sq, LARGE_SHORT);
        assert_ne!(row_costs[0].sigma_sq, LARGE_SHORT);
        assert_eq!(col_costs[0].sigma_sq, LARGE_SHORT);
        assert_eq!(col_costs[col_len - 1].sigma_sq, LARGE_SHORT);
        assert_ne!(col_costs[1].sigma_sq, LARGE_SHORT);
    }

    #[test]
    fn mask_prespecified_costs_validates_lengths() {
        let result =
            mask_prespecified_arc_costs::<Cost>(&mut [], &mut [], &[0], &[], 1, 1, Cost::mask);
        assert!(matches!(
            result,
            Err(MaskWeightsError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn incr_cost_get_cost_positive_direction() {
        let ic = IncrCost::new(42, -10);
        assert_eq!(ic.get_cost(1), 42);
        assert_eq!(ic.get_cost(5), 42);
    }

    #[test]
    fn incr_cost_get_cost_negative_direction() {
        let ic = IncrCost::new(42, -10);
        assert_eq!(ic.get_cost(-1), -10);
        assert_eq!(ic.get_cost(-99), -10);
    }

    #[test]
    fn incr_cost_get_cost_zero_direction_returns_negcost() {
        // The C code uses `if(arcdir>0)`, so zero maps to the else branch.
        let ic = IncrCost::new(42, -10);
        assert_eq!(ic.get_cost(0), -10);
    }

    #[test]
    fn incr_cost_default_is_zero() {
        let ic = IncrCost::default();
        assert_eq!(ic.poscost, 0);
        assert_eq!(ic.negcost, 0);
    }

    #[test]
    fn thicken_costs_smooths_row_col_arcs_and_returns_max() {
        let mut costs = vec![
            IncrCost::new(10, 0),
            IncrCost::new(20, 0),
            IncrCost::new(30, 0),
            IncrCost::new(40, 0),
            IncrCost::new(50, 0),
            IncrCost::new(60, 0),
            IncrCost::new(70, 0),
        ];
        let max = thicken_costs(&mut costs, 2, 3);

        // Row arcs (first 3), then col arcs (last 4)
        assert_eq!(costs[0].negcost, 13);
        assert_eq!(costs[1].negcost, 20);
        assert_eq!(costs[2].negcost, 27);
        assert_eq!(costs[3].negcost, 47);
        assert_eq!(costs[4].negcost, 57);
        assert_eq!(costs[5].negcost, 53);
        assert_eq!(costs[6].negcost, 63);
        assert_eq!(max, 63);
    }

    #[test]
    fn thicken_costs_clips_to_large_short() {
        let mut costs = vec![
            IncrCost::new(i16::MAX, 0),
            IncrCost::new(i16::MAX, 0),
            IncrCost::new(i16::MAX, 0),
            IncrCost::new(i16::MAX, 0),
        ];
        let max = thicken_costs(&mut costs, 2, 2);
        assert!(costs.iter().all(|c| c.negcost == LARGE_SHORT));
        assert_eq!(max, LARGE_SHORT as i64);
    }
}
