//! Safe helpers mirroring the legacy bucket queue utilities.

use std::fmt;

pub type BucketResult<T> = Result<T, BucketError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BucketError {
    InvalidBucket,
    InvalidNode,
    NodeAlreadyLinked,
    NodeNotInBucket,
}

impl fmt::Display for BucketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BucketError::InvalidBucket => write!(f, "bucket index out of range"),
            BucketError::InvalidNode => write!(f, "node index out of range"),
            BucketError::NodeAlreadyLinked => write!(f, "node is already linked to a bucket"),
            BucketError::NodeNotInBucket => write!(f, "node is not linked to the specified bucket"),
        }
    }
}

impl std::error::Error for BucketError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeGroup {
    #[default]
    Unknown,
    OnTree,
    InBucket,
    NotInBucket,
}

#[derive(Debug, Clone)]
pub struct BucketNode {
    next: Option<usize>,
    prev: Option<usize>,
    bucket: Option<usize>,
    pub group: NodeGroup,
}

impl Default for BucketNode {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
            bucket: None,
            group: NodeGroup::Unknown,
        }
    }
}

impl BucketNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(&self) -> Option<usize> {
        self.next
    }

    pub fn prev(&self) -> Option<usize> {
        self.prev
    }
}

#[derive(Debug)]
pub struct Buckets {
    heads: Vec<Option<usize>>,
}

impl Buckets {
    pub fn new(size: usize) -> Self {
        Self {
            heads: vec![None; size],
        }
    }

    pub fn insert(
        &mut self,
        nodes: &mut [BucketNode],
        node_idx: usize,
        bucket_idx: usize,
    ) -> BucketResult<()> {
        let head = self
            .heads
            .get_mut(bucket_idx)
            .ok_or(BucketError::InvalidBucket)?;
        let head_idx = *head;
        {
            let node = nodes.get_mut(node_idx).ok_or(BucketError::InvalidNode)?;
            if node.bucket.is_some() {
                return Err(BucketError::NodeAlreadyLinked);
            }
            node.next = head_idx;
            node.prev = None;
            node.bucket = Some(bucket_idx);
            node.group = NodeGroup::InBucket;
        }
        if let Some(idx) = head_idx {
            nodes[idx].prev = Some(node_idx);
        }
        *head = Some(node_idx);
        Ok(())
    }

    pub fn remove(
        &mut self,
        nodes: &mut [BucketNode],
        node_idx: usize,
        bucket_idx: usize,
    ) -> BucketResult<()> {
        let head = self
            .heads
            .get_mut(bucket_idx)
            .ok_or(BucketError::InvalidBucket)?;
        let (next_idx, prev_idx) = {
            let node = nodes.get(node_idx).ok_or(BucketError::InvalidNode)?;
            if node.bucket != Some(bucket_idx) {
                return Err(BucketError::NodeNotInBucket);
            }
            (node.next, node.prev)
        };

        if let Some(next) = next_idx {
            nodes[next].prev = prev_idx;
        }
        if let Some(prev) = prev_idx {
            nodes[prev].next = next_idx;
        } else {
            *head = next_idx;
        }

        let node = &mut nodes[node_idx];
        node.next = None;
        node.prev = None;
        node.bucket = None;
        node.group = NodeGroup::NotInBucket;
        Ok(())
    }

    pub fn head(&self, bucket_idx: usize) -> Option<usize> {
        self.heads.get(bucket_idx).copied().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_vec(count: usize) -> Vec<BucketNode> {
        (0..count).map(|_| BucketNode::new()).collect()
    }

    #[test]
    fn insert_sets_head_and_pointers() {
        let mut buckets = Buckets::new(1);
        let mut nodes = node_vec(3);
        buckets.insert(&mut nodes, 0, 0).unwrap();
        buckets.insert(&mut nodes, 1, 0).unwrap();
        buckets.insert(&mut nodes, 2, 0).unwrap();
        assert_eq!(buckets.head(0), Some(2));
        assert_eq!(nodes[2].next(), Some(1));
        assert_eq!(nodes[1].next(), Some(0));
        assert_eq!(nodes[0].next(), None);
        assert_eq!(nodes[1].prev(), Some(2));
        assert_eq!(nodes[0].prev(), Some(1));
    }

    #[test]
    fn remove_detaches_middle_node() {
        let mut buckets = Buckets::new(1);
        let mut nodes = node_vec(3);
        buckets.insert(&mut nodes, 0, 0).unwrap();
        buckets.insert(&mut nodes, 1, 0).unwrap();
        buckets.insert(&mut nodes, 2, 0).unwrap();
        buckets.remove(&mut nodes, 1, 0).unwrap();
        assert_eq!(nodes[2].next(), Some(0));
        assert_eq!(nodes[0].prev(), Some(2));
        assert!(matches!(nodes[1].group, NodeGroup::NotInBucket));
        assert!(nodes[1].next().is_none());
        assert!(nodes[1].prev().is_none());
    }

    #[test]
    fn remove_head_updates_bucket() {
        let mut buckets = Buckets::new(1);
        let mut nodes = node_vec(2);
        buckets.insert(&mut nodes, 0, 0).unwrap();
        buckets.insert(&mut nodes, 1, 0).unwrap();
        buckets.remove(&mut nodes, 1, 0).unwrap();
        assert_eq!(buckets.head(0), Some(0));
        buckets.remove(&mut nodes, 0, 0).unwrap();
        assert_eq!(buckets.head(0), None);
    }

    #[test]
    fn removing_non_member_errors() {
        let mut buckets = Buckets::new(1);
        let mut nodes = node_vec(1);
        assert!(matches!(
            buckets.remove(&mut nodes, 0, 0),
            Err(BucketError::NodeNotInBucket)
        ));
    }
}
