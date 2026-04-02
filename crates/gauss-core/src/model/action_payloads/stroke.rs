//! Stroke and rotation types for action payloads.

use super::float::normalize_float;

/// Stroke width in document units.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct StrokeWidth(f32);

impl StrokeWidth {
    /// Construct a new stroke width.
    ///
    /// Returns `None` if the value is negative or non-finite.
    #[must_use]
    pub const fn new(value: super::Points) -> Option<Self> {
        if value.0.is_finite() && value.0 >= 0.0 {
            Some(Self(normalize_float(value.0)))
        } else {
            None
        }
    }

    /// Return the stroke width value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}

impl std::hash::Hash for StrokeWidth {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Eq for StrokeWidth {}

/// Rotation angle in degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Rotation(f32);

impl Rotation {
    /// Construct a new rotation.
    ///
    /// Returns `None` if the value is not finite.
    #[must_use]
    pub const fn new(value: super::Degrees) -> Option<Self> {
        if value.0.is_finite() {
            Some(Self(normalize_float(value.0)))
        } else {
            None
        }
    }

    /// Return the rotation value in degrees.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}

impl std::hash::Hash for Rotation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Eq for Rotation {}
