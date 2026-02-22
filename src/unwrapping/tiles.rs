#![allow(dead_code)]

//! Tile assembly workflows.
//!
//! This module contains the typed Rust translation of secondary-arc tracing
//! primitives used during tiled unwrapping (`TraceSecondaryArc` in C).

use crate::data::ops::l_round;
use std::collections::HashMap;

const LARGE_INT: i64 = 2_000_000_000;
const ZERO_COST_ARC: i64 = -LARGE_INT;
const MAX_OFFSET_REFINEMENTS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcDirection {
    Right = 1,
    Down = 2,
    Left = 3,
    Up = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrimaryCoord {
    pub row: i64,
    pub col: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimaryArcHop {
    /// `None` matches the C `LARGESHORT` sentinel path where the arc is
    /// considered zero-cost for variance accumulation.
    pub sigma_sq: Option<f64>,
    /// Marks arcs that are forced zero-cost because they lie on global edges.
    pub forced_zero_cost: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecondaryCostParams {
    pub flowmax: usize,
    pub nshortcycle: i64,
    pub tileedgeweight: f64,
    pub is_tile_edge_arc: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryArcCostProfile {
    /// Offset used while sampling per-flow costs (`scndrycostarr[0]` in C).
    pub arroffset: i64,
    /// Incremental costs for `+/-nflow` (`scndrycostarr[1..=2*flowmax]`).
    pub incremental_costs: Vec<i64>,
    /// Variance metadata stored in the final slot (`scndrycostarr[2*flowmax+1]`).
    pub variance_sum_tag: i64,
    /// Number of traversed primary arcs while tracing the secondary arc.
    pub arc_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceCostError {
    EmptyPath,
    InvalidFlowMax,
    OffsetRefinementDidNotConverge,
}

/// Compute secondary-arc cost profile from a traced primary-arc path.
///
/// This is the typed Rust equivalent of the cost-building core in C
/// `TraceSecondaryArc()`, including:
/// - centered cost-table search via `arroffset`
/// - zero-cost arc handling
/// - tile-edge weighting
/// - variance-tag storage in the trailing slot
///
/// The `eval_costs` callback must return `(nominal, pos, neg)` costs for a
/// hop at the current `arroffset` and requested `nflow` increment.
pub fn trace_secondary_arc_costs<F>(
    hops: &[PrimaryArcHop],
    params: SecondaryCostParams,
    mut eval_costs: F,
) -> Result<SecondaryArcCostProfile, TraceCostError>
where
    F: FnMut(usize, i64, i64) -> (i64, i64, i64),
{
    if hops.is_empty() {
        return Err(TraceCostError::EmptyPath);
    }
    if params.flowmax == 0 {
        return Err(TraceCostError::InvalidFlowMax);
    }

    let flowmax = params.flowmax as i64;
    let mut arroffset = 0i64;
    let mut refinements = 0usize;
    let mut zerocost: bool;

    let mut incremental = vec![0i64; (2 * params.flowmax) as usize];
    let mut sumsigsqinv: f64;

    loop {
        incremental.fill(0);
        sumsigsqinv = 0.0;
        zerocost = false;

        for (hop_idx, hop) in hops.iter().enumerate() {
            if hop.forced_zero_cost {
                zerocost = true;
                break;
            }

            for nflow in 1..=flowmax {
                let (nom, pos, neg) = eval_costs(hop_idx, arroffset, nflow);
                let pos_idx = (nflow - 1) as usize;
                let neg_idx = (flowmax + nflow - 1) as usize;
                incremental[pos_idx] =
                    clip_large_int(incremental[pos_idx].saturating_add(pos.saturating_sub(nom)));
                incremental[neg_idx] =
                    clip_large_int(incremental[neg_idx].saturating_add(neg.saturating_sub(nom)));
            }

            if let Some(sigsq) = hop.sigma_sq {
                if sigsq > 0.0 && sigsq.is_finite() {
                    sumsigsqinv += 1.0 / sigsq;
                }
            }
        }

        if zerocost {
            break;
        }

        let mut min_cost = 0i64;
        let mut max_cost = 0i64;
        let mut min_cost_flow = 0i64;
        for nflow in 1..=flowmax {
            let pos = incremental[(nflow - 1) as usize];
            if pos < min_cost {
                min_cost = pos;
                min_cost_flow = nflow;
            }
            if pos > max_cost {
                max_cost = pos;
            }

            let neg = incremental[(flowmax + nflow - 1) as usize];
            if neg < min_cost {
                min_cost = neg;
                min_cost_flow = -nflow;
            }
            if neg > max_cost {
                max_cost = neg;
            }
        }

        // All-zero profile behaves as zero-cost arc in the C implementation.
        if max_cost == min_cost {
            zerocost = true;
            sumsigsqinv = 0.0;
            break;
        }

        if min_cost_flow == 0 {
            break;
        }

        if min_cost_flow == flowmax {
            arroffset -= (1.5 * flowmax as f64).floor() as i64;
        } else if min_cost_flow == -flowmax {
            arroffset += (1.5 * flowmax as f64).floor() as i64;
        } else {
            arroffset -= min_cost_flow;
        }

        refinements += 1;
        if refinements > MAX_OFFSET_REFINEMENTS {
            return Err(TraceCostError::OffsetRefinementDidNotConverge);
        }
    }

    let variance_sum_tag = if zerocost {
        incremental.fill(0);
        0
    } else {
        if params.is_tile_edge_arc {
            for v in &mut incremental {
                *v = clip_large_int(l_round(*v as f64 * params.tileedgeweight));
            }
            sumsigsqinv *= params.tileedgeweight;
        }

        let weighted = sumsigsqinv * (params.nshortcycle * params.nshortcycle) as f64;
        if weighted < LARGE_INT as f64 {
            l_round(weighted)
        } else {
            LARGE_INT
        }
    };

    Ok(SecondaryArcCostProfile {
        arroffset,
        incremental_costs: incremental,
        variance_sum_tag: if zerocost {
            ZERO_COST_ARC
        } else {
            variance_sum_tag
        },
        arc_len: hops.len(),
    })
}

#[inline]
fn clip_large_int(value: i64) -> i64 {
    value.clamp(-LARGE_INT, LARGE_INT)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SecondaryNodeKey {
    pub tile: usize,
    pub primary_row: i64,
    pub primary_col: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryNode {
    pub key: SecondaryNodeKey,
    pub neighbors: Vec<usize>,
    pub outarcs: Vec<Option<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryArc {
    pub arcrow: usize,
    pub arccol: usize,
    pub from: usize,
    pub to: usize,
    pub fromdir: ArcDirection,
    pub cost_profile: SecondaryArcCostProfile,
}

#[derive(Debug, Default)]
pub struct SecondaryGraph {
    pub nodes: Vec<SecondaryNode>,
    pub arcs: Vec<SecondaryArc>,
    node_map: HashMap<SecondaryNodeKey, usize>,
}

impl SecondaryGraph {
    pub fn get_or_insert_node(&mut self, key: SecondaryNodeKey) -> usize {
        if let Some(&idx) = self.node_map.get(&key) {
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(SecondaryNode {
            key,
            neighbors: Vec::new(),
            outarcs: Vec::new(),
        });
        self.node_map.insert(key, idx);
        idx
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonTileArcUpdate {
    pub node_idx: usize,
    pub outarc_slot: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceSecondaryArcOutcome {
    IgnoredSourceOrPriorTileEdge,
    IgnoredLoop,
    AddedNewArc { arc_idx: usize },
    ReusedExistingArc { arc_idx: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceSecondaryArcResult {
    pub outcome: TraceSecondaryArcOutcome,
    pub updated_non_tile_nodes: Vec<NonTileArcUpdate>,
    pub total_arc_len_delta: usize,
}

/// Register one traced secondary arc in the mutable secondary graph.
///
/// This is the idiomatic Rust equivalent of the graph-update half of C
/// `TraceSecondaryArc()` (node lookup/creation, duplicate-arc handling,
/// neighbor/outarc bookkeeping, and non-tile update tracking).
pub fn trace_secondary_arc(
    graph: &mut SecondaryGraph,
    current_tile: usize,
    head_key: SecondaryNodeKey,
    tail_key: SecondaryNodeKey,
    fromdir: ArcDirection,
    cost_profile: SecondaryArcCostProfile,
    skip_for_source_or_prev_tile: bool,
) -> TraceSecondaryArcResult {
    if skip_for_source_or_prev_tile {
        return TraceSecondaryArcResult {
            outcome: TraceSecondaryArcOutcome::IgnoredSourceOrPriorTileEdge,
            updated_non_tile_nodes: Vec::new(),
            total_arc_len_delta: 0,
        };
    }

    if head_key == tail_key {
        return TraceSecondaryArcResult {
            outcome: TraceSecondaryArcOutcome::IgnoredLoop,
            updated_non_tile_nodes: Vec::new(),
            total_arc_len_delta: 0,
        };
    }

    let tail_idx = graph.get_or_insert_node(tail_key);
    let head_idx = graph.get_or_insert_node(head_key);

    // Find existing adjacency.
    let mut existing_tail_slot = None;
    for (i, &neighbor) in graph.nodes[tail_idx].neighbors.iter().enumerate() {
        if neighbor == head_idx {
            existing_tail_slot = Some(i);
            break;
        }
    }

    // Existing relation: update arc if present, else create and bind one.
    if let Some(tail_slot) = existing_tail_slot {
        let arc_idx = if let Some(arc_idx) = graph.nodes[tail_idx].outarcs[tail_slot] {
            graph.arcs[arc_idx].fromdir = fromdir;
            graph.arcs[arc_idx].cost_profile = cost_profile.clone();
            arc_idx
        } else {
            let arc_idx = graph.arcs.len();
            let arccol = arc_idx;
            graph.arcs.push(SecondaryArc {
                arcrow: current_tile,
                arccol,
                from: tail_idx,
                to: head_idx,
                fromdir,
                cost_profile: cost_profile.clone(),
            });

            graph.nodes[tail_idx].outarcs[tail_slot] = Some(arc_idx);
            if let Some(head_slot) = find_neighbor_slot(&graph.nodes[head_idx], tail_idx) {
                graph.nodes[head_idx].outarcs[head_slot] = Some(arc_idx);
            }
            arc_idx
        };

        return TraceSecondaryArcResult {
            outcome: TraceSecondaryArcOutcome::ReusedExistingArc { arc_idx },
            updated_non_tile_nodes: collect_non_tile_updates(
                graph,
                current_tile,
                tail_idx,
                head_idx,
            ),
            total_arc_len_delta: cost_profile.arc_len,
        };
    }

    // New relation + new arc.
    let arc_idx = graph.arcs.len();
    graph.arcs.push(SecondaryArc {
        arcrow: current_tile,
        arccol: arc_idx,
        from: tail_idx,
        to: head_idx,
        fromdir,
        cost_profile: cost_profile.clone(),
    });

    graph.nodes[tail_idx].neighbors.push(head_idx);
    graph.nodes[tail_idx].outarcs.push(Some(arc_idx));
    graph.nodes[head_idx].neighbors.push(tail_idx);
    graph.nodes[head_idx].outarcs.push(Some(arc_idx));

    TraceSecondaryArcResult {
        outcome: TraceSecondaryArcOutcome::AddedNewArc { arc_idx },
        updated_non_tile_nodes: collect_non_tile_updates(graph, current_tile, tail_idx, head_idx),
        total_arc_len_delta: cost_profile.arc_len,
    }
}

fn find_neighbor_slot(node: &SecondaryNode, neighbor_idx: usize) -> Option<usize> {
    node.neighbors.iter().position(|&n| n == neighbor_idx)
}

fn collect_non_tile_updates(
    graph: &SecondaryGraph,
    current_tile: usize,
    tail_idx: usize,
    head_idx: usize,
) -> Vec<NonTileArcUpdate> {
    let mut updates = Vec::new();
    if graph.nodes[tail_idx].key.tile != current_tile {
        let outarc_slot = find_neighbor_slot(&graph.nodes[tail_idx], head_idx).unwrap_or(0);
        updates.push(NonTileArcUpdate {
            node_idx: tail_idx,
            outarc_slot,
        });
    }
    if graph.nodes[head_idx].key.tile != current_tile {
        let outarc_slot = find_neighbor_slot(&graph.nodes[head_idx], tail_idx).unwrap_or(0);
        updates.push(NonTileArcUpdate {
            node_idx: head_idx,
            outarc_slot,
        });
    }
    updates
}

/// Placeholder while higher-level tile orchestration is still under migration.
pub fn assemble_tiles() {
    // TODO: call the translated secondary-arc helpers from TraceRegions.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_secondary_arc_costs_handles_zero_cost_arcs() {
        let hops = vec![
            PrimaryArcHop {
                sigma_sq: Some(4.0),
                forced_zero_cost: true,
            },
            PrimaryArcHop {
                sigma_sq: Some(2.0),
                forced_zero_cost: false,
            },
        ];
        let profile = trace_secondary_arc_costs(
            &hops,
            SecondaryCostParams {
                flowmax: 3,
                nshortcycle: 5,
                tileedgeweight: 2.0,
                is_tile_edge_arc: true,
            },
            |_hop, _offset, _n| (0, 10, 10),
        )
        .unwrap();

        assert_eq!(profile.incremental_costs, vec![0; 6]);
        assert_eq!(profile.variance_sum_tag, ZERO_COST_ARC);
    }

    #[test]
    fn trace_secondary_arc_costs_scales_tile_edge_costs() {
        let hops = vec![PrimaryArcHop {
            sigma_sq: Some(2.0),
            forced_zero_cost: false,
        }];
        let profile = trace_secondary_arc_costs(
            &hops,
            SecondaryCostParams {
                flowmax: 2,
                nshortcycle: 3,
                tileedgeweight: 2.0,
                is_tile_edge_arc: true,
            },
            |_hop, _offset, n| {
                // nom=10, positive increment goes +n, negative +2n
                (10, 10 + n, 10 + 2 * n)
            },
        )
        .unwrap();

        // Original [1,2,2,4] then scaled by x2.
        assert_eq!(profile.incremental_costs, vec![2, 4, 4, 8]);
        assert_eq!(profile.variance_sum_tag, 9); // (1/2)*2 * 3^2 = 9
    }

    #[test]
    fn trace_secondary_arc_costs_recenters_using_arroffset() {
        let hops = vec![PrimaryArcHop {
            sigma_sq: Some(1.0),
            forced_zero_cost: false,
        }];
        let profile = trace_secondary_arc_costs(
            &hops,
            SecondaryCostParams {
                flowmax: 3,
                nshortcycle: 1,
                tileedgeweight: 1.0,
                is_tile_edge_arc: false,
            },
            |_hop, offset, n| {
                // Synthetic convergent model:
                // at offset 0, n=1 gives negative forward delta -> arroffset -= 1
                // at offset -1, all deltas are non-negative -> converged.
                let nom = 10;
                if offset == 0 && n == 1 {
                    (nom, 8, 12)
                } else {
                    (nom, 12, 12)
                }
            },
        )
        .unwrap();

        assert_eq!(profile.arroffset, -1);
    }

    #[test]
    fn trace_secondary_arc_adds_new_arc_and_nodes() {
        let mut graph = SecondaryGraph::default();
        let profile = SecondaryArcCostProfile {
            arroffset: 0,
            incremental_costs: vec![1, 2, 3, 4],
            variance_sum_tag: 7,
            arc_len: 2,
        };
        let tail = SecondaryNodeKey {
            tile: 1,
            primary_row: 4,
            primary_col: 5,
        };
        let head = SecondaryNodeKey {
            tile: 1,
            primary_row: 4,
            primary_col: 6,
        };

        let result = trace_secondary_arc(
            &mut graph,
            1,
            head,
            tail,
            ArcDirection::Right,
            profile,
            false,
        );
        assert!(matches!(
            result.outcome,
            TraceSecondaryArcOutcome::AddedNewArc { .. }
        ));
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.arcs.len(), 1);
        assert_eq!(result.total_arc_len_delta, 2);
    }

    #[test]
    fn trace_secondary_arc_reuses_existing_arc() {
        let mut graph = SecondaryGraph::default();
        let tail = SecondaryNodeKey {
            tile: 2,
            primary_row: 1,
            primary_col: 1,
        };
        let head = SecondaryNodeKey {
            tile: 2,
            primary_row: 1,
            primary_col: 2,
        };
        let p1 = SecondaryArcCostProfile {
            arroffset: 0,
            incremental_costs: vec![1, 1],
            variance_sum_tag: 1,
            arc_len: 1,
        };
        let p2 = SecondaryArcCostProfile {
            arroffset: 3,
            incremental_costs: vec![9, 9],
            variance_sum_tag: 5,
            arc_len: 1,
        };

        let _ = trace_secondary_arc(
            &mut graph,
            2,
            head,
            tail,
            ArcDirection::Right,
            p1.clone(),
            false,
        );
        let result = trace_secondary_arc(
            &mut graph,
            2,
            head,
            tail,
            ArcDirection::Left,
            p2.clone(),
            false,
        );

        assert!(matches!(
            result.outcome,
            TraceSecondaryArcOutcome::ReusedExistingArc { .. }
        ));
        assert_eq!(graph.arcs.len(), 1);
        assert_eq!(graph.arcs[0].fromdir, ArcDirection::Left);
        assert_eq!(graph.arcs[0].cost_profile, p2);
    }

    #[test]
    fn trace_secondary_arc_ignores_source_or_loop_cases() {
        let mut graph = SecondaryGraph::default();
        let key = SecondaryNodeKey {
            tile: 0,
            primary_row: 0,
            primary_col: 0,
        };
        let profile = SecondaryArcCostProfile {
            arroffset: 0,
            incremental_costs: vec![1, 2],
            variance_sum_tag: 3,
            arc_len: 1,
        };

        let source_skip = trace_secondary_arc(
            &mut graph,
            0,
            key,
            SecondaryNodeKey {
                tile: 0,
                primary_row: 0,
                primary_col: 1,
            },
            ArcDirection::Right,
            profile.clone(),
            true,
        );
        assert!(matches!(
            source_skip.outcome,
            TraceSecondaryArcOutcome::IgnoredSourceOrPriorTileEdge
        ));

        let loop_skip =
            trace_secondary_arc(&mut graph, 0, key, key, ArcDirection::Right, profile, false);
        assert!(matches!(
            loop_skip.outcome,
            TraceSecondaryArcOutcome::IgnoredLoop
        ));
    }
}
