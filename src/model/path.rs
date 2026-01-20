//! Vector geometry types used by the editor.
//!
//! SVG represents paths as segments (`L`, `C`, `Q`, `Z`, ...). Editors, however,
//! need an editable representation with anchors and handles. This module stores
//! geometry in that editor-friendly form, while remaining straightforward to
//! compile into SVG path strings or a GPUI `PathBuilder` later.

#![expect(
    clippy::float_arithmetic,
    reason = "vector operations require floating-point arithmetic"
)]

use std::ops::{Add, Mul, Sub};

use slotmap::{Key, KeyData, new_key_type};

new_key_type! {
    /// Identifier for a [`Shape`].
    pub struct ShapeId;
}

impl ShapeId {
    /// Return the underlying key data.
    #[must_use]
    pub fn as_key_data(self) -> KeyData {
        self.data()
    }

    /// Convert this ID into a stable AccessKit node identifier.
    #[must_use]
    pub fn to_accesskit_node_id(self) -> u64 {
        self.data().as_ffi()
    }

    /// Reconstruct a shape ID from an AccessKit node identifier.
    ///
    /// The `raw` value must originate from [`ShapeId::to_accesskit_node_id`].
    #[must_use]
    pub fn from_accesskit_node_id(raw: u64) -> Self {
        KeyData::from_ffi(raw).into()
    }
}

/// A 2D point or vector in document (“world”) coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
}

impl Vec2 {
    /// The origin.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Construct a new vector.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Add two vectors.
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    /// Subtract two vectors.
    #[must_use]
    pub const fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    /// Multiply by a scalar.
    #[must_use]
    pub const fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    /// Return the squared magnitude of the vector.
    #[must_use]
    pub const fn magnitude_squared(self) -> f32 {
        (self.x * self.x) + (self.y * self.y)
    }

    /// Return the squared Euclidean distance to `other`.
    #[must_use]
    pub const fn distance_squared(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx) + (dy * dy)
    }

    /// Return the Euclidean distance to `other`.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        self.distance_squared(other).sqrt()
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        self.mul(scalar)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, vec: Vec2) -> Vec2 {
        vec.mul(self)
    }
}

/// An RGBA colour in 8-bit per channel sRGB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel (`0..=255`).
    pub a: u8,
}

impl Rgba {
    /// Construct a new colour from 8-bit channels.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Stroke and fill styling for a [`Shape`].
#[derive(Clone, Debug, PartialEq)]
pub struct PaintStyle {
    /// Stroke colour. `None` means no stroke.
    pub stroke: Option<Rgba>,
    /// Stroke width in document units (treat as pixels for the `PoC`).
    pub stroke_width: f32,
    /// Fill colour. `None` means no fill.
    pub fill: Option<Rgba>,
}

impl PaintStyle {
    /// Construct a new paint style.
    #[must_use]
    pub const fn new(stroke: Option<Rgba>, stroke_width: f32, fill: Option<Rgba>) -> Self {
        Self {
            stroke,
            stroke_width,
            fill,
        }
    }
}

/// Editable anchor point of a path, potentially with Bézier handles.
///
/// Handles are stored in document space (absolute positions), which simplifies
/// hit-testing and transformations.
#[derive(Clone, Debug, PartialEq)]
pub struct Anchor {
    /// Anchor position.
    pub pos: Vec2,
    /// Incoming handle influencing the curve segment entering this anchor.
    pub handle_in: Option<Vec2>,
    /// Outgoing handle influencing the curve segment leaving this anchor.
    pub handle_out: Option<Vec2>,
}

impl Anchor {
    /// Construct an anchor with no handles.
    #[must_use]
    pub const fn new(pos: Vec2) -> Self {
        Self {
            pos,
            handle_in: None,
            handle_out: None,
        }
    }
}

/// The kind of segment between two anchors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentKind {
    /// Straight line segment.
    Line,
    /// Cubic Bézier segment (uses `handle_out` of the start anchor and
    /// `handle_in` of the end anchor).
    Cubic,
}

/// Path geometry for a shape.
///
/// Invariants (expected by later code):
///
/// - `segments.len()` should equal `anchors.len().saturating_sub(1)` for open
///   paths.
/// - Closed paths still use the same `segments` length, with the closing edge
///   described by [`Self::closing_segment`].
#[derive(Clone, Debug, PartialEq)]
pub struct PathGeom {
    /// Path anchors in draw order.
    pub anchors: Vec<Anchor>,
    /// Segment kinds between successive anchors.
    pub segments: Vec<SegmentKind>,
    /// Whether the path is closed.
    pub closed: bool,
    /// Segment kind for the closing edge (from the last anchor back to the
    /// first anchor) when [`Self::closed`] is true.
    ///
    /// Note: the closing edge may be a cubic Bézier. In that case, the closing
    /// segment uses:
    /// - `handle_out` of the last anchor, and
    /// - `handle_in` of the first anchor.
    pub closing_segment: SegmentKind,
}

impl PathGeom {
    /// Construct an empty open path.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            anchors: Vec::new(),
            segments: Vec::new(),
            closed: false,
            closing_segment: SegmentKind::Line,
        }
    }

    /// Return whether the path has no anchors.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }
}

impl Default for PathGeom {
    fn default() -> Self {
        Self::new()
    }
}

/// A single drawable path shape in the document.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    /// Stable identifier for selection and referencing.
    pub id: ShapeId,
    /// Z ordering value (higher draws on top).
    pub z: i32,
    /// Stroke/fill styling.
    pub style: PaintStyle,
    /// Path geometry.
    pub path: PathGeom,
}
