#![allow(dead_code)]

//! Basic raster representation for interferogram grids.

#[derive(Debug, Clone)]
pub struct Raster<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T> Raster<T> {
    pub fn new(width: usize, height: usize, data: Vec<T>) -> Self {
        Self { width, height, data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
