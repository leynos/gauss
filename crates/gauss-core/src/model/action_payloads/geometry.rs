//! Geometry types for action payloads.

use super::float::normalize_float;

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
        match super::float::new_pair(p.x, p.y) {
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
