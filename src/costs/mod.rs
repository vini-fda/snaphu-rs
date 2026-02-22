#![allow(dead_code)]

//! Cost calculation traits and helpers.

pub mod gradients;
pub mod lookup;
pub mod types;

/// Trait implemented by each cost component once translated.
pub trait CostBuilder {
    fn name(&self) -> &str;
}

pub use gradients::{
    GradientDirection, WrappedGradientError, WrappedGradientField, compute_azimuth_gradients,
    compute_range_gradients,
};
pub use types::{
    Cost, MaskWeightsError, SmoothCost, mask_cost, mask_prespecified_arc_costs, mask_smooth_cost,
};
