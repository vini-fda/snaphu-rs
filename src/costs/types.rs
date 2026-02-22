#![allow(dead_code)]

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
}
