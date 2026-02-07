use log::error;

const ALLOCATION_FAULT: i32 = 5;
const UNFEASIBLE: i32 = 2;
const PRICE_OFL: i32 = 6;
const ABNORMAL_EXIT: i32 = 1;

const PRICE_MIN: f64 = 0.0;

/// Sentinel value for buckets
const DUMMY_NODE: NodeIndex = usize::MAX;
/// Sentinel value for ranks (bucket indices)
const DUMMY_RANK: BucketIndex = usize::MAX;

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
    pub rank: BucketIndex,
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

impl Bucket {
    /// Create an empty bucket
    pub fn new() -> Self {
        Self {
            p_first: DUMMY_NODE,
        }
    }

    /// Reset bucket to empty
    ///
    /// Corresponds to SNAPHU's `RESET_BUCKET(b)` C macro
    pub fn reset(&mut self) {
        self.p_first = DUMMY_NODE;
    }

    /// Check if bucket is non-empty
    ///
    /// Corresponds to SNAPHU's `NONEMPTY_BUCKET(b)` C macro
    pub fn is_nonempty(&self) -> bool {
        self.p_first != DUMMY_NODE
    }

    /// Insert node of index `i` from `nodes` at front of bucket (O(1))
    ///
    /// Corresponds to SNAPHU's `INSERT_TO_BUCKET(i,b)` C macro
    pub fn push_front(&mut self, i: NodeIndex, nodes: &mut [Node]) {
        let first = self.p_first;

        nodes[i].b_next = first;
        nodes[i].b_prev = DUMMY_NODE;

        if first != DUMMY_NODE {
            nodes[first].b_prev = i;
        }

        self.p_first = i;
    }

    /// Remove and return first node from bucket (O(1))
    ///
    /// Corresponds to SNAPHU's `GET_FROM_BUCKET(i,b)` C macro
    pub fn pop_front(&mut self, nodes: &mut [Node]) -> Option<NodeIndex> {
        let i = self.p_first;
        if i == DUMMY_NODE {
            return None;
        }

        let next = nodes[i].b_next;
        self.p_first = next;

        if next != DUMMY_NODE {
            nodes[next].b_prev = DUMMY_NODE;
        }

        nodes[i].b_next = DUMMY_NODE;
        nodes[i].b_prev = DUMMY_NODE;

        Some(i)
    }

    /// Remove arbitrary node from bucket (O(1))
    ///
    /// Corresponds to SNAPHU's `REMOVE_FROM_BUCKET(i,b)` C macro
    pub fn remove(&mut self, i: NodeIndex, nodes: &mut [Node]) {
        let prev = nodes[i].b_prev;
        let next = nodes[i].b_next;

        if prev == DUMMY_NODE {
            // i is first element
            self.p_first = next;
        } else {
            nodes[prev].b_next = next;
        }

        if next != DUMMY_NODE {
            nodes[next].b_prev = prev;
        }

        nodes[i].b_next = DUMMY_NODE;
        nodes[i].b_prev = DUMMY_NODE;
    }

    /// Peek first element without removing
    pub fn peek(&self) -> Option<NodeIndex> {
        if self.p_first == DUMMY_NODE {
            None
        } else {
            Some(self.p_first)
        }
    }
}

pub struct Graph {
    pub nodes: Vec<Node>,
    pub arcs: Vec<Arc>,
}

/// Returns whether arc index `a` in `arcs` is open.
///
/// Corresponds to SNAPHU's `OPEN(a)` C macro
pub fn is_open(arcs: &[Arc], a: ArcIndex) -> bool {
    arcs[a].r_cap > 0
}

/// Scans all outgoing arcs of an active node `i` and relaxes neighboring nodes
/// using a bucketed label update.
///
/// This is a core primitive in the push–relabel algorithm,
/// where buckets replace priority queues to achieve linear-time label updates.
pub fn up_node_scan(
    nodes: &mut [Node],
    arcs: &[Arc],
    buckets: &mut [Bucket],
    i: NodeIndex, /* node for scanning */
    n_scan: &mut usize,
    dn: f64,
    epsilon: f64,
    linf: BucketIndex, /* number of l_bucket + 1 */
    dlinf: f64,        /* copy of linf in double mode */
) {
    let mut j: NodeIndex; /* opposite node */
    let mut b_old: Bucket; /* old bucket contained j */
    let mut b_new: Bucket; /* new bucket for j */
    let mut j_rank: BucketIndex; /* ranks of nodes */
    let mut j_new_rank: BucketIndex;
    let i_rank: BucketIndex = nodes[i].rank;
    let mut rc: f64; /* reduced cost of (j,i) */
    let mut dr: f64; /* rank difference */
    let a_first: ArcIndex = nodes[i].first; /* "a" represents arcs ( i, j ) */
    let a_stop: ArcIndex = nodes[i + 1].suspended; /* first arc from the next node */
    *n_scan += 1;
    // Loop over all outgoing arcs from node i
    for a in a_first..a_stop {
        let ra: ArcIndex = arcs[a].sister; /* ( j, i ) */
        if is_open(arcs, ra) {
            j = arcs[a].head as NodeIndex;
            j_rank = nodes[j].rank;
            if j_rank > i_rank {
                rc = nodes[j].price + dn * arcs[ra].cost as f64 - nodes[i].price;
                if rc < 0.0 {
                    j_new_rank = i_rank;
                } else {
                    dr = rc / epsilon;
                    j_new_rank = if dr < dlinf {
                        i_rank + dr as BucketIndex + 1
                    } else {
                        linf
                    };
                }
                if j_rank > j_new_rank {
                    nodes[j].rank = j_new_rank;
                    nodes[j].current = ra;
                    if j_rank < linf {
                        b_old = buckets[j_rank];
                        b_old.remove(j, nodes);
                    }
                    b_new = buckets[j_new_rank];
                    b_new.push_front(j, nodes);
                }
            }
        }
    }
    nodes[i].price -= i_rank as f64 * epsilon;
    nodes[i].rank = DUMMY_RANK;
}

/// Attempts to relabel node of index `i` by scanning its outgoing residual arcs and
/// updating its price (label) to the best admissible value.
///
/// This is a local relaxation step over the adjacency list
/// of node `i`: it finds the neighbor that maximizes a reduced-price expression
/// and either selects an admissible arc or raises `i`’s label accordingly.
/// Returns `true` if an admissible outgoing arc is found, and `false` otherwise.
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
        if is_open(arcs, a) && {
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
        if is_open(arcs, a) && {
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
