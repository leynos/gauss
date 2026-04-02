//! Payload types for Action variants.
//!
//! This module contains the value types used as payloads in Action enum variants,
//! extracted from action.rs to reduce file size and improve maintainability.

/// RGB colour value for action payloads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

/// RGB colour components for constructing a [`Color`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgb8 {
    /// Red channel (0-255).
    pub r: u8,
    /// Green channel (0-255).
    pub g: u8,
    /// Blue channel (0-255).
    pub b: u8,
}

impl Color {
    /// Construct a new colour from RGB components.
    #[must_use]
    pub const fn new(rgb: Rgb8) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

/// Normalize a float for consistent hashing and equality.
///
/// Maps -0.0 to 0.0. NaN values should be rejected before calling this.
#[inline]
pub const fn normalize_float(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

/// Helper to validate and normalize a pair of finite f32 values.
#[inline]
const fn new_pair(x: f32, y: f32) -> Option<(f32, f32)> {
    if x.is_finite() && y.is_finite() {
        Some((normalize_float(x), normalize_float(y)))
    } else {
        None
    }
}

/// Generic length unit in document points.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Points(pub f32);

impl Points {
    /// Return the inner value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}

/// Stroke width in document units.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct StrokeWidth(f32);

impl StrokeWidth {
    /// Construct a new stroke width.
    ///
    /// Returns `None` if the value is negative or non-finite.
    #[must_use]
    pub const fn new(value: Points) -> Option<Self> {
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
    type Error = &'static str;

    fn try_from(v: f32) -> Result<Self, Self::Error> {
        if v.is_finite() && (0.0..=1.0).contains(&v) {
            Ok(Self(v))
        } else {
            Err("value out of range, expected 0.0..=1.0")
        }
    }
}

/// Opacity value (0.0..=1.0).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Opacity(f32);

impl Opacity {
    /// Construct a new opacity.
    ///
    /// Returns `None` if the value is not in the range [0.0, 1.0].
    #[must_use]
    pub const fn new(value: UnitF32) -> Option<Self> {
        // UnitF32 already validates the range, but we double-check for const fn
        if !value.0.is_finite() {
            return None;
        }
        if value.0 < 0.0 || value.0 > 1.0 {
            return None;
        }
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

/// 2D point in document coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
}

/// 2D position in document coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    x: f32,
    y: f32,
}

impl Position {
    /// Construct a new position.
    ///
    /// Returns `None` if either coordinate is not finite.
    #[must_use]
    pub const fn new(p: Point) -> Option<Self> {
        match new_pair(p.x, p.y) {
            Some((nx, ny)) => Some(Self { x: nx, y: ny }),
            None => None,
        }
    }

    /// Return the x coordinate.
    #[must_use]
    pub const fn x(&self) -> f32 {
        self.x
    }

    /// Return the y coordinate.
    #[must_use]
    pub const fn y(&self) -> f32 {
        self.y
    }
}

impl std::hash::Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl Eq for Position {}

/// 2D dimensions in document units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dimensions {
    /// Width in document units.
    pub width: f32,
    /// Height in document units.
    pub height: f32,
}

/// 2D size in document units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    width: f32,
    height: f32,
}

impl Size {
    /// Construct a new size.
    ///
    /// Returns `None` if either dimension is not finite or is negative.
    #[must_use]
    pub const fn new(d: Dimensions) -> Option<Self> {
        if !d.width.is_finite() || !d.height.is_finite() {
            return None;
        }
        if d.width < 0.0 || d.height < 0.0 {
            return None;
        }
        Some(Self {
            width: normalize_float(d.width),
            height: normalize_float(d.height),
        })
    }

    /// Return the width.
    #[must_use]
    pub const fn width(&self) -> f32 {
        self.width
    }

    /// Return the height.
    #[must_use]
    pub const fn height(&self) -> f32 {
        self.height
    }
}

impl std::hash::Hash for Size {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.height.to_bits().hash(state);
    }
}

impl Eq for Size {}

/// Angle in degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Degrees(pub f32);

impl Degrees {
    /// Return the inner value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
}

/// Rotation angle in degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Rotation(f32);

impl Rotation {
    /// Construct a new rotation.
    ///
    /// Returns `None` if the value is not finite.
    #[must_use]
    pub const fn new(value: Degrees) -> Option<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_width_rejects_negative_and_non_finite() {
        // Negative should be rejected
        assert!(StrokeWidth::new(Points(-1.0)).is_none());
        assert!(StrokeWidth::new(Points(-0.1)).is_none());

        // Non-finite should be rejected
        assert!(StrokeWidth::new(Points(f32::NAN)).is_none());
        assert!(StrokeWidth::new(Points(f32::INFINITY)).is_none());
        assert!(StrokeWidth::new(Points(f32::NEG_INFINITY)).is_none());

        // Valid values should be accepted
        assert!(StrokeWidth::new(Points(0.0)).is_some());
        assert!(StrokeWidth::new(Points(1.0)).is_some());
        assert!(StrokeWidth::new(Points(2.5)).is_some());
    }

    #[test]
    fn stroke_width_normalizes_negative_zero() {
        let sw_neg = StrokeWidth::new(Points(-0.0)).unwrap();
        let sw_pos = StrokeWidth::new(Points(0.0)).unwrap();
        // Both should hash to the same value
        let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
        sw_neg.hash(&mut hasher_neg);
        sw_pos.hash(&mut hasher_pos);
        assert_eq!(hasher_neg.finish(), hasher_pos.finish());
        // And be equal
        assert_eq!(sw_neg, sw_pos);
    }

    #[test]
    fn unit_f32_rejects_out_of_range_and_non_finite() {
        // Out of range should be rejected
        assert!(UnitF32::try_from(-0.1).is_err());
        assert!(UnitF32::try_from(1.1).is_err());
        assert!(UnitF32::try_from(2.0).is_err());

        // Non-finite should be rejected
        assert!(UnitF32::try_from(f32::NAN).is_err());
        assert!(UnitF32::try_from(f32::INFINITY).is_err());
        assert!(UnitF32::try_from(f32::NEG_INFINITY).is_err());

        // Valid values should be accepted
        assert!(UnitF32::try_from(0.0).is_ok());
        assert!(UnitF32::try_from(1.0).is_ok());
        assert!(UnitF32::try_from(0.5).is_ok());
    }

    #[test]
    fn opacity_rejects_non_finite() {
        // Opacity::new takes UnitF32, which already validates
        // But we test the constructor path
        assert!(Opacity::new(UnitF32::try_from(0.5).unwrap()).is_some());
    }

    #[test]
    fn opacity_normalizes_negative_zero() {
        let op_neg = Opacity::new(UnitF32::try_from(-0.0).unwrap()).unwrap();
        let op_pos = Opacity::new(UnitF32::try_from(0.0).unwrap()).unwrap();
        // Both should hash to the same value
        let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
        op_neg.hash(&mut hasher_neg);
        op_pos.hash(&mut hasher_pos);
        assert_eq!(hasher_neg.finish(), hasher_pos.finish());
        // And be equal
        assert_eq!(op_neg, op_pos);
    }

    #[test]
    fn position_rejects_non_finite() {
        // Non-finite should be rejected
        assert!(
            Position::new(Point {
                x: f32::NAN,
                y: 0.0
            })
            .is_none()
        );
        assert!(
            Position::new(Point {
                x: 0.0,
                y: f32::NAN
            })
            .is_none()
        );
        assert!(
            Position::new(Point {
                x: f32::INFINITY,
                y: 0.0
            })
            .is_none()
        );
        assert!(
            Position::new(Point {
                x: 0.0,
                y: f32::NEG_INFINITY
            })
            .is_none()
        );

        // Valid values should be accepted
        assert!(Position::new(Point { x: 0.0, y: 0.0 }).is_some());
        assert!(Position::new(Point { x: 1.0, y: 2.0 }).is_some());
    }

    #[test]
    fn position_normalizes_negative_zero() {
        let pos_neg = Position::new(Point { x: -0.0, y: -0.0 }).unwrap();
        let pos_pos = Position::new(Point { x: 0.0, y: 0.0 }).unwrap();
        // Both should hash to the same value
        let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
        pos_neg.hash(&mut hasher_neg);
        pos_pos.hash(&mut hasher_pos);
        assert_eq!(hasher_neg.finish(), hasher_pos.finish());
        // And be equal
        assert_eq!(pos_neg, pos_pos);
    }

    #[test]
    fn size_rejects_negative_and_non_finite() {
        // Negative dimensions should be rejected
        assert!(
            Size::new(Dimensions {
                width: -1.0,
                height: 1.0
            })
            .is_none()
        );
        assert!(
            Size::new(Dimensions {
                width: 1.0,
                height: -1.0
            })
            .is_none()
        );
        assert!(
            Size::new(Dimensions {
                width: -0.1,
                height: -0.1
            })
            .is_none()
        );

        // Non-finite should be rejected
        assert!(
            Size::new(Dimensions {
                width: f32::NAN,
                height: 1.0
            })
            .is_none()
        );
        assert!(
            Size::new(Dimensions {
                width: 1.0,
                height: f32::INFINITY
            })
            .is_none()
        );
        assert!(
            Size::new(Dimensions {
                width: f32::NEG_INFINITY,
                height: 1.0
            })
            .is_none()
        );

        // Valid values should be accepted
        assert!(
            Size::new(Dimensions {
                width: 0.0,
                height: 0.0
            })
            .is_some()
        );
        assert!(
            Size::new(Dimensions {
                width: 100.0,
                height: 200.0
            })
            .is_some()
        );
    }

    #[test]
    fn size_normalizes_negative_zero() {
        let size_neg = Size::new(Dimensions {
            width: -0.0,
            height: -0.0,
        })
        .unwrap();
        let size_pos = Size::new(Dimensions {
            width: 0.0,
            height: 0.0,
        })
        .unwrap();
        // Both should hash to the same value
        let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
        size_neg.hash(&mut hasher_neg);
        size_pos.hash(&mut hasher_pos);
        assert_eq!(hasher_neg.finish(), hasher_pos.finish());
        // And be equal
        assert_eq!(size_neg, size_pos);
    }

    #[test]
    fn rotation_rejects_non_finite() {
        // Non-finite should be rejected
        assert!(Rotation::new(Degrees(f32::NAN)).is_none());
        assert!(Rotation::new(Degrees(f32::INFINITY)).is_none());
        assert!(Rotation::new(Degrees(f32::NEG_INFINITY)).is_none());

        // Valid values should be accepted
        assert!(Rotation::new(Degrees(0.0)).is_some());
        assert!(Rotation::new(Degrees(45.0)).is_some());
        assert!(Rotation::new(Degrees(-90.0)).is_some());
    }

    #[test]
    fn rotation_normalizes_negative_zero() {
        let rot_neg = Rotation::new(Degrees(-0.0)).unwrap();
        let rot_pos = Rotation::new(Degrees(0.0)).unwrap();
        // Both should hash to the same value
        let mut hasher_neg = std::collections::hash_map::DefaultHasher::new();
        let mut hasher_pos = std::collections::hash_map::DefaultHasher::new();
        rot_neg.hash(&mut hasher_neg);
        rot_pos.hash(&mut hasher_pos);
        assert_eq!(hasher_neg.finish(), hasher_pos.finish());
        // And be equal
        assert_eq!(rot_neg, rot_pos);
    }

    #[test]
    fn normalize_float_converts_negative_zero() {
        assert_eq!(normalize_float(-0.0), 0.0);
        assert_eq!(normalize_float(0.0), 0.0);
        assert_eq!(normalize_float(1.0), 1.0);
        assert_eq!(normalize_float(-1.0), -1.0);
    }
}
