use log::error;

const ALLOCATION_FAULT: i32 = 5;
const UNFEASIBLE: i32 = 2;
const PRICE_OFL: i32 = 6;
const ABNORMAL_EXIT: i32 = 1;

const PRICE_MIN: f64 = 0.0;

pub type NodeIndex = usize;
pub type ArcIndex = usize;
pub type BucketIndex = usize;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Arc {
    /// Residual capacity
    pub r_cap: i16,
    /// Cost of the arc
    pub cost: i16,
    /// Head node
    pub head: NodeIndex,
    /// Opposite arc
    pub sister: ArcIndex,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    /// First outgoing arc
    pub first: ArcIndex,
    /// Current outgoing arc
    pub current: ArcIndex,
    pub suspended: ArcIndex,
    /// Distance from a sink
    pub price: f64,
    /// Next node in a push queue
    pub q_next: NodeIndex,
    /// Next node in a bucket-list
    pub b_next: NodeIndex,
    /// Previous node in a bucket-list
    pub b_prev: NodeIndex,
    /// Bucket number
    pub rank: i64,
    /// Excess of the node
    pub excess: i64,
    /// Temporary number of input arcs
    pub inp: i8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Bucket {
    pub p_first: NodeIndex,
}

pub struct Graph {
    pub nodes: Vec<Node>,
    pub arcs: Vec<Arc>,
}

pub fn up_node_scan(
    nodes: &mut [Node],
    arcs: &[Arc],
    buckets: &mut [Bucket],
    current_bucket: BucketIndex,
    i: NodeIndex,
    n_scan: &mut usize,
    dn: f64,
    epsilon: f64,
    linf: i64,  /* number of l_bucket + 1 */
    dlinf: f64, /* copy of linf in double mode */
) {
    let mut j: NodeIndex; /* opposite node */
    let mut b_old: BucketIndex; /* old bucket contained j */
    let mut b_new: BucketIndex; /* new bucket for j */
    let mut j_rank: i64; /* ranks of nodes */
    let mut j_new_rank: i64;
    let i_rank: i64 = nodes[i].rank;
    let mut rc: f64; /* reduced cost of (j,i) */
    let mut dr: f64; /* rank difference */
    let mut a: ArcIndex = nodes[i].first; /* ( i, j ) */
    let a_stop: ArcIndex = nodes[i + 1].suspended; /* first arc from the next node */
    *n_scan += 1;
    while a != a_stop {
        let ra: ArcIndex = arcs[a].sister; /* ( j, i ) */
        if arcs[ra].r_cap > 0 {
            j = arcs[a].head as NodeIndex;
            j_rank = nodes[j].rank;
            if j_rank > i_rank {
                rc = nodes[j].price + dn * arcs[ra].cost as f64 - nodes[i].price;
                if rc < 0 as f64 {
                    j_new_rank = i_rank;
                } else {
                    dr = rc / epsilon;
                    j_new_rank = if dr < dlinf {
                        i_rank + dr as i64 + 1
                    } else {
                        linf
                    };
                }
                if j_rank > j_new_rank {
                    nodes[j].rank = j_new_rank;
                    nodes[j].current = ra;
                    if j_rank < linf {
                        b_old = (current_bucket as i64 + j_rank) as BucketIndex;
                        // TODO: reevaluate this REMOVE_FROM_BUCKET
                        if j == buckets[b_old].p_first {
                            buckets[b_old].p_first = nodes[j].b_next as NodeIndex;
                        } else {
                            nodes[nodes[j].b_prev].b_next = nodes[j].b_next;
                            nodes[nodes[j].b_next].b_prev = nodes[j].b_prev;
                        }
                    }
                    b_new = (current_bucket as i64 + j_new_rank) as BucketIndex;
                    // TODO: reevaluate this INSERT_TO_BUCKET
                    nodes[j].b_next = buckets[b_new].p_first;
                    nodes[buckets[b_new].p_first].b_prev = j;
                    buckets[b_new].p_first = j;
                }
            }
        }
        a = a + 1;
    }
    nodes[i].price -= i_rank as f64 * epsilon;
    nodes[i].rank = -1;
}

/// Relabels a node of index `i`.
pub fn relabel(
    nodes: &mut [Node],
    arcs: &[Arc],
    i: NodeIndex,
    price_min: f64,
    dn: f64,
    epsilon: f64,
    n_ref: i64,
    flag_price: &mut i32,
    n_relabel: &mut i64,
    n_rel: &mut i64,
) -> bool {
    // current arc from i
    let mut a: ArcIndex = nodes[i].current + 1;
    // first arc from the next node
    let mut a_stop: ArcIndex = nodes[i + 1].suspended;
    // arc which provides maximum price
    let mut a_max: ArcIndex = 0; // TODO: figure out if 0 is adequate here
    // current maximal price
    let mut p_max: f64 = price_min;
    // price of node i
    let i_price: f64 = nodes[i].price;
    // current arc partial residual cost
    let mut dp: f64;

    while a != a_stop {
        if arcs[a].r_cap > 0 && {
            let head: NodeIndex = arcs[a].head;
            dp = nodes[head].price - dn * arcs[a].cost as f64;
            dp > p_max
        } {
            if i_price < dp {
                nodes[i].current = a;
                return true;
            }
            p_max = dp;
            a_max = a;
        }
        a = a + 1;
    }
    a = nodes[i].first;
    a_stop = nodes[i].current + 1;
    while a != a_stop {
        if arcs[a].r_cap > 0 && {
            let head: NodeIndex = arcs[a].head;
            dp = nodes[head].price - dn * arcs[a].cost as f64;
            dp > p_max
        } {
            if i_price < dp {
                nodes[i].current = a;
                return true;
            }
            p_max = dp;
            a_max = a;
        }
        a = a + 1;
    }
    if p_max != price_min {
        nodes[i].price = p_max - epsilon;
        nodes[i].current = a_max;
    } else if nodes[i].suspended == nodes[i].first {
        if nodes[i].excess == 0 {
            nodes[i].price = price_min;
        } else if n_ref == 1 {
            err_end(UNFEASIBLE);
        } else {
            err_end(PRICE_OFL);
        }
    } else {
        *flag_price = 1;
    }
    *n_relabel += 1;
    *n_rel += 1;
    return false;
}

/// Logs the error type inside of the cs2 solver and panics, returning the [ABNORMAL_EXIT] code.
fn err_end(cc: i32) {
    error!("cs2 solver: Error {}", cc);
    if cc == ALLOCATION_FAULT {
        error!("allocation fault");
    } else if cc == UNFEASIBLE {
        error!("(problem infeasible)");
    } else if cc == PRICE_OFL {
        error!("(price overflow)");
    }
    /*
    2 - problem is unfeasible
    5 - allocation fault
    6 - price overflow
    */
    panic!("Abnormal Exit: {}", ABNORMAL_EXIT);
}
