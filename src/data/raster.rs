//! Basic row-major raster representation for interferogram grids.

/// Contiguous row-major raster data.
///
/// `width` is the number of samples per row, `height` is the number of rows,
/// and `data` stores exactly `width * height` samples in row-major order.
#[derive(Debug, Clone, PartialEq)]
pub struct Raster<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T> Raster<T> {
    /// Creates a contiguous row-major raster.
    ///
    /// The caller is responsible for ensuring that `data.len() == width * height`.
    pub fn new(width: usize, height: usize, data: Vec<T>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    /// Returns the number of stored samples.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` when the raster stores no samples.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the expected number of samples for this raster shape.
    pub fn expected_len(&self) -> usize {
        self.width.saturating_mul(self.height)
    }

    /// Returns the underlying contiguous row-major slice.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the underlying contiguous row-major mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns `true` when `data.len() == width * height`.
    pub fn has_valid_shape(&self) -> bool {
        self.len() == self.expected_len()
    }
}
