use log::error;

const ALLOCATION_FAULT: i32 = 5;
const UNFEASIBLE: i32 = 2;
const PRICE_OFL: i32 = 6;
const ABNORMAL_EXIT: i32 = 1;

const PRICE_MIN: f64 = 0.0;

pub type NodeIndex = usize;
pub type ArcIndex = usize;

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
    pub rank: u64,
    /// Excess of the node
    pub excess: u64,
    /// Temporary number of input arcs
    pub inp: i8,
}

pub struct Graph {
    pub nodes: Vec<Node>,
    pub arcs: Vec<Arc>,
}

/// Relabels a node of index `i`.
pub fn relabel(
    nodes: &mut [Node],
    arcs: &[Arc],
    i: NodeIndex,
    price_min: f64,
    dn: f64,
    epsilon: f64,
    n_ref: u64,
    flag_price: &mut i32,
    n_relabel: &mut u64,
    n_rel: &mut u64,
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
