#![allow(dead_code)]

//! Per-tile metadata structures.

#[derive(Debug, Clone, Default)]
pub struct TileMetadata {
    pub id: usize,
    pub pixel_count: usize,
}
