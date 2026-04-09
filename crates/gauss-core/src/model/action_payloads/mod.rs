//! Payload types for Action variants.
//!
//! This module contains the value types used as payloads in Action enum variants,
//! extracted from action.rs to reduce file size and improve maintainability.

mod color;
mod float;
mod geometry;
mod stroke;

pub use color::{Color, Rgb8};
pub use float::{Degrees, Opacity, Points, UnitF32, UnitF32Error};
pub use geometry::{Dimensions, Point, Position, Size};
pub use stroke::{Rotation, StrokeWidth};

#[cfg(test)]
pub use float::normalize_float;

#[cfg(test)]
mod tests;
