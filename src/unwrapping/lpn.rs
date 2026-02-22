#![allow(dead_code)]

//! Linear programming cost helpers.

use crate::data::ops::l_round;
use crate::unwrapping::tiles::SecondaryArcCostProfile;

const ZERO_COST_ARC: i64 = -2_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiDirWeight {
    pub posweight: i16,
    pub negweight: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LpCostError {
    InvalidArcIndex,
    InvalidFlowIncrement,
    InvalidCostTable,
}

#[inline]
fn abs_pow(flow: i64, p: f64) -> f64 {
    (flow.abs() as f64).powf(p)
}

/// Compute incremental +/- cost for one arc in scalar weighted Lp mode.
///
/// This is the typed Rust equivalent of C `CalcCostLP()`.
pub fn calc_cost_lp(
    costs: &[Vec<i16>],
    flow: i64,
    arcrow: usize,
    arccol: usize,
    nflow: i64,
    p: f64,
) -> Result<(i64, i64), LpCostError> {
    let w = i64::from(
        *costs
            .get(arcrow)
            .and_then(|row| row.get(arccol))
            .ok_or(LpCostError::InvalidArcIndex)?,
    );
    let pos = l_round(w as f64 * (abs_pow(flow + nflow, p) - abs_pow(flow, p)));
    let neg = l_round(w as f64 * (abs_pow(flow - nflow, p) - abs_pow(flow, p)));
    Ok((pos, neg))
}

/// Compute incremental +/- cost for one arc in bidirectional weighted Lp mode.
///
/// This is the typed Rust equivalent of C `CalcCostLPBiDir()`.
pub fn calc_cost_lp_bidir(
    costs: &[Vec<BiDirWeight>],
    flow: i64,
    arcrow: usize,
    arccol: usize,
    nflow: i64,
    p: f64,
) -> Result<(i64, i64), LpCostError> {
    let w = costs
        .get(arcrow)
        .and_then(|row| row.get(arccol))
        .copied()
        .ok_or(LpCostError::InvalidArcIndex)?;

    let cost0 = if flow > 0 {
        f64::from(w.posweight) * (flow as f64).powf(p)
    } else {
        f64::from(w.negweight) * ((-flow) as f64).powf(p)
    };

    let new_pos = flow + nflow;
    let pos = if new_pos > 0 {
        l_round(f64::from(w.posweight) * (new_pos as f64).powf(p) - cost0)
    } else {
        l_round(f64::from(w.negweight) * (new_pos as f64).powf(p) - cost0)
    };

    let new_neg = flow - nflow;
    let neg = if new_neg > 0 {
        l_round(f64::from(w.posweight) * (new_neg as f64).powf(p) - cost0)
    } else {
        l_round(f64::from(w.negweight) * (new_neg as f64).powf(p) - cost0)
    };
    Ok((pos, neg))
}

fn non_grid_abs_cost(table: &SecondaryArcCostProfile, xflow: i64, flowmax: i64) -> i64 {
    if xflow > flowmax {
        let c1 = table.incremental_costs[(flowmax - 1) as usize] as f64 / flowmax as f64
            - table.variance_sum_tag as f64 * flowmax as f64;
        (table.variance_sum_tag * xflow + l_round(c1)) * xflow
    } else if xflow < -flowmax {
        let c1 = table.incremental_costs[(2 * flowmax - 1) as usize] as f64 / flowmax as f64
            - table.variance_sum_tag as f64 * flowmax as f64;
        (table.variance_sum_tag * xflow + l_round(c1)) * xflow
    } else if xflow > 0 {
        table.incremental_costs[(xflow - 1) as usize]
    } else if xflow < 0 {
        table.incremental_costs[(flowmax - xflow - 1) as usize]
    } else {
        0
    }
}

/// Compute incremental +/- cost for one arc in secondary (non-grid) mode.
///
/// This is the typed Rust equivalent of C `CalcCostNonGrid()`.
pub fn calc_cost_non_grid(
    costs: &[Vec<SecondaryArcCostProfile>],
    flow: i64,
    arcrow: usize,
    arccol: usize,
    nflow: i64,
) -> Result<(i64, i64), LpCostError> {
    if nflow == 0 {
        return Err(LpCostError::InvalidFlowIncrement);
    }
    let table = costs
        .get(arcrow)
        .and_then(|row| row.get(arccol))
        .ok_or(LpCostError::InvalidArcIndex)?;
    let flowmax = (table.incremental_costs.len() / 2) as i64;
    if flowmax == 0 {
        return Err(LpCostError::InvalidCostTable);
    }
    if table.variance_sum_tag == ZERO_COST_ARC {
        return Ok((0, 0));
    }

    let abs0 = non_grid_abs_cost(table, flow + table.arroffset, flowmax);
    let pos = non_grid_abs_cost(table, flow + table.arroffset + nflow, flowmax) - abs0;
    let neg = non_grid_abs_cost(table, flow + table.arroffset - nflow, flowmax) - abs0;
    let nflow_sq = nflow * nflow;

    let pos_unit = if pos > 0 {
        (pos as f64 / nflow_sq as f64).ceil() as i64
    } else {
        (pos as f64 / nflow_sq as f64).floor() as i64
    };
    let neg_unit = if neg > 0 {
        (neg as f64 / nflow_sq as f64).ceil() as i64
    } else {
        (neg as f64 / nflow_sq as f64).floor() as i64
    };
    Ok((pos_unit, neg_unit))
}

/// Evaluate one scalar weighted Lp arc cost.
///
/// This is the typed Rust equivalent of C `EvalCostLP()`.
pub fn eval_cost_lp(
    costs: &[Vec<i16>],
    flows: &[Vec<i16>],
    arcrow: usize,
    arccol: usize,
    p: f64,
) -> Result<i64, LpCostError> {
    let w = i64::from(
        *costs
            .get(arcrow)
            .and_then(|row| row.get(arccol))
            .ok_or(LpCostError::InvalidArcIndex)?,
    );
    let f = i64::from(
        *flows
            .get(arcrow)
            .and_then(|row| row.get(arccol))
            .ok_or(LpCostError::InvalidArcIndex)?,
    );
    Ok(l_round(w as f64 * abs_pow(f, p)))
}

/// Evaluate one bidirectional weighted Lp arc cost.
///
/// This is the typed Rust equivalent of C `EvalCostLPBiDir()`.
pub fn eval_cost_lp_bidir(
    costs: &[Vec<BiDirWeight>],
    flows: &[Vec<i16>],
    arcrow: usize,
    arccol: usize,
    p: f64,
) -> Result<i64, LpCostError> {
    let w = costs
        .get(arcrow)
        .and_then(|row| row.get(arccol))
        .copied()
        .ok_or(LpCostError::InvalidArcIndex)?;
    let f = i64::from(
        *flows
            .get(arcrow)
            .and_then(|row| row.get(arccol))
            .ok_or(LpCostError::InvalidArcIndex)?,
    );
    if f > 0 {
        Ok(l_round(f64::from(w.posweight) * (f as f64).powf(p)))
    } else {
        // C uses posweight for both branches in EvalCostLPBiDir().
        Ok(l_round(f64::from(w.posweight) * ((-f) as f64).powf(p)))
    }
}

/// Evaluate one secondary (non-grid) arc cost at current flow.
///
/// This is the typed Rust equivalent of C `EvalCostNonGrid()`.
pub fn eval_cost_non_grid(
    costs: &[Vec<SecondaryArcCostProfile>],
    flows: &[Vec<i16>],
    arcrow: usize,
    arccol: usize,
) -> Result<i64, LpCostError> {
    let table = costs
        .get(arcrow)
        .and_then(|row| row.get(arccol))
        .ok_or(LpCostError::InvalidArcIndex)?;
    let flow = i64::from(
        *flows
            .get(arcrow)
            .and_then(|row| row.get(arccol))
            .ok_or(LpCostError::InvalidArcIndex)?,
    );
    let flowmax = (table.incremental_costs.len() / 2) as i64;
    if flowmax == 0 {
        return Err(LpCostError::InvalidCostTable);
    }
    if table.variance_sum_tag == ZERO_COST_ARC {
        return Ok(0);
    }
    Ok(non_grid_abs_cost(table, flow + table.arroffset, flowmax))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_and_eval_lp_match_simple_cases() {
        let costs = vec![vec![10i16]];
        let flows = vec![vec![3i16]];
        let (pos, neg) = calc_cost_lp(&costs, 3, 0, 0, 1, 2.0).unwrap();
        assert_eq!(pos, 70);
        assert_eq!(neg, -50);
        assert_eq!(eval_cost_lp(&costs, &flows, 0, 0, 2.0).unwrap(), 90);
    }

    #[test]
    fn bidir_lp_uses_directional_weights() {
        let costs = vec![vec![BiDirWeight {
            posweight: 3,
            negweight: 7,
        }]];
        let flows = vec![vec![-2i16]];
        let (pos, neg) = calc_cost_lp_bidir(&costs, -2, 0, 0, 1, 1.0).unwrap();
        assert!(pos < 0);
        assert!(neg < 0);
        assert_eq!(eval_cost_lp_bidir(&costs, &flows, 0, 0, 2.0).unwrap(), 12);
    }

    #[test]
    fn non_grid_cost_zero_arc_short_circuits() {
        let costs = vec![vec![SecondaryArcCostProfile {
            arroffset: 0,
            incremental_costs: vec![1, 2, 3, 4],
            variance_sum_tag: ZERO_COST_ARC,
            arc_len: 1,
        }]];
        let flows = vec![vec![5i16]];
        assert_eq!(calc_cost_non_grid(&costs, 5, 0, 0, 1).unwrap(), (0, 0));
        assert_eq!(eval_cost_non_grid(&costs, &flows, 0, 0).unwrap(), 0);
    }
}
