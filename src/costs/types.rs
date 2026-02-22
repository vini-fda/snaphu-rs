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
}
