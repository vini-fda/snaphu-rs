//! Per-tile metadata structures.

#[derive(Debug, Clone, Default)]
pub struct TileMetadata {
    pub id: usize,
    pub pixel_count: usize,
}

/// Location and size of a tile in the interferogram grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileRegion {
    pub first_row: usize,
    pub first_col: usize,
    pub rows: usize,
    pub cols: usize,
}

impl TileRegion {
    pub fn new(first_row: usize, first_col: usize, rows: usize, cols: usize) -> Self {
        Self {
            first_row,
            first_col,
            rows,
            cols,
        }
    }
}
