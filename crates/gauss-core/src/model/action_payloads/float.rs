//! Float normalization and validation utilities.

use thiserror::Error;

/// Normalize a float for consistent hashing and equality.
///
/// Maps -0.0 to 0.0. NaN values should be rejected before calling this.
#[inline]
pub const fn normalize_float(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

/// Helper to validate and normalize a pair of finite f32 values.
#[inline]
pub(crate) const fn new_pair(x: f32, y: f32) -> Option<(f32, f32)> {
    if x.is_finite() && y.is_finite() {
        Some((normalize_float(x), normalize_float(y)))
    } else {
        None
    }
}

/// Generic length unit in document points.
///
/// This is an input type that does not validate its contents.
/// Use [`StrokeWidth`](super::StrokeWidth) for a validated length with
/// construction that rejects non-finite or negative values.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Points(pub f32);

impl Points {
    /// Return the inner value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}

/// Error returned when constructing a [`UnitF32`] from an `f32` fails.
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitF32Error {
    /// The supplied value was non-finite (NaN or infinity).
    #[error("value must be finite")]
    NonFinite,
    /// The supplied value was finite but outside the range `0.0..=1.0`.
    #[error("value out of range, expected 0.0..=1.0")]
    OutOfRange,
}

/// Unit float in the range [0.0, 1.0].
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct UnitF32(f32);

impl UnitF32 {
    /// Return the inner value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for UnitF32 {
    type Error = UnitF32Error;

    fn try_from(v: f32) -> Result<Self, Self::Error> {
        if !v.is_finite() {
            return Err(UnitF32Error::NonFinite);
        }
        if !(0.0..=1.0).contains(&v) {
            return Err(UnitF32Error::OutOfRange);
        }
        Ok(Self(normalize_float(v)))
    }
}

/// Opacity value (0.0..=1.0).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Opacity(f32);

impl Opacity {
    /// Construct a new opacity.
    ///
    /// Returns `Some(Opacity)` containing the normalized value. The `UnitF32`
    /// type already guarantees the value is finite and within [0.0, 1.0], so
    /// this constructor always succeeds. Returns `Option` for API consistency
    /// with other payload constructors.
    #[must_use]
    pub const fn new(value: UnitF32) -> Option<Self> {
        Some(Self(normalize_float(value.0)))
    }

    /// Return the opacity value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}

impl std::hash::Hash for Opacity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Eq for Opacity {}

/// Angle in degrees.
///
/// This is an input type that does not validate its contents.
/// Use [`Rotation`](super::Rotation) for a validated angle with construction
/// that rejects non-finite values.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Degrees(pub f32);

impl Degrees {
    /// Return the inner value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}
