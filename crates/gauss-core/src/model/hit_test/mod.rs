//! Shared deterministic hit-testing service for pointer interactions.
//!
//! This module provides a model-layer hit-test boundary that can be reused by
//! selection, hover, and future accessibility object-navigation workflows.
//! `HitTestIndex` currently uses deterministic linear scans over draw-order
//! shapes while keeping a stable API boundary for future spatial indices.

#![expect(
    clippy::float_arithmetic,
    reason = "hit-testing relies on floating-point distance maths"
)]

use crate::model::select_tool::{
    SelectAnchorHit, SelectHandleHit, SelectHandleHitKind, SelectPointerHit, SelectSegmentHit,
    SelectShapeHit,
};
use crate::model::{Document, Shape, ShapeId, Vec2};

mod geometry;

/// Strategy used by [`HitTestIndex`] to resolve hit queries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HitTestBackend {
    /// Deterministic linear scan over draw-order shapes.
    #[default]
    LinearScan,
}

/// Shape-level hit result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
}

/// Anchor-level hit result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
    /// Anchor index in `shape.path.anchors`.
    pub anchor_index: usize,
}

/// Handle endpoint hit kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleHitKind {
    /// Incoming handle endpoint.
    In,
    /// Outgoing handle endpoint.
    Out,
}

/// Handle-level hit result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandleHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
    /// Anchor index in `shape.path.anchors`.
    pub anchor_index: usize,
    /// Handle endpoint kind.
    pub kind: HandleHitKind,
}

/// Segment-level hit result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentHit {
    /// Draw-order index compatible with `Document::shape_at`.
    pub shape_index: usize,
    /// Stable shape identifier.
    pub shape_id: ShapeId,
    /// Segment index in `shape.path.segments`.
    pub seg_index: usize,
}

/// Topmost pointer target resolved by hit-testing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTarget {
    /// Pointer intersects a handle endpoint.
    Handle(HandleHit),
    /// Pointer intersects an anchor point.
    Anchor(AnchorHit),
    /// Pointer intersects a segment.
    Segment(SegmentHit),
    /// Pointer intersects a shape bounding box.
    Shape(ShapeHit),
}

#[derive(Clone, Copy, Debug)]
struct IndexedShape<'a> {
    /// Position in the document's draw order, used to preserve hit priority.
    shape_index: usize,
    /// Borrowed geometry for the shape at `shape_index`.
    shape: &'a Shape,
}

/// Queryable hit-test index built from a document snapshot.
///
/// `HitTestIndex` is intentionally a lightweight borrowed view over
/// [`Document`]. Rebuilding it for high-frequency pointer queries is
/// allocation-free, which avoids introducing a self-referential cache inside
/// `Phase0Shell` before a real spatial index exists.
#[derive(Clone, Debug)]
pub struct HitTestIndex<'a> {
    /// Strategy used for the index's shape traversal.
    backend: HitTestBackend,
    /// Document whose shapes are queried.
    doc: &'a Document,
}

impl<'a> HitTestIndex<'a> {
    /// Build a hit-test index from the document's draw-order snapshot.
    #[must_use]
    pub const fn from_document(doc: &'a Document) -> Self {
        Self {
            backend: HitTestBackend::LinearScan,
            doc,
        }
    }

    /// Return the active hit-test strategy.
    #[must_use]
    pub const fn backend(&self) -> HitTestBackend {
        self.backend
    }

    /// Visit shapes from front to back, returning the first result produced by `f`.
    /// Later draw-order shapes are offered to `f` before earlier shapes.
    fn scan_shapes_rev<T, F>(&self, f: F) -> Option<T>
    where
        F: Fn(&IndexedShape<'_>) -> Option<T>,
    {
        (0..self.doc.len()).rev().find_map(|shape_index| {
            let shape = self.doc.shape_at(shape_index)?;
            let indexed = IndexedShape { shape_index, shape };
            f(&indexed)
        })
    }

    /// Normalize the tolerance, then apply `f` in draw order until it finds a hit.
    /// A non-finite tolerance returns `None` without visiting any shapes.
    fn topmost_in_shapes<T, F>(&self, tolerance_world: f32, f: F) -> Option<T>
    where
        F: Fn(&IndexedShape<'_>, f32) -> Option<T>,
    {
        let tolerance = normalize_tolerance(tolerance_world)?;
        self.scan_shapes_rev(|indexed| f(indexed, tolerance))
    }

    /// Resolve the deterministic pointer-down hit target.
    #[must_use]
    pub fn pointer_hit(&self, cursor_world: Vec2, tolerance_world: f32) -> SelectPointerHit {
        self.select_pointer_hit(cursor_world, tolerance_world)
    }

    /// Resolve the deterministic hover hit target.
    ///
    /// Hover and pointer-down currently share identical resolution ordering.
    #[must_use]
    pub fn hover_hit(&self, cursor_world: Vec2, tolerance_world: f32) -> SelectPointerHit {
        self.select_pointer_hit(cursor_world, tolerance_world)
    }

    /// Return the topmost hit target at the cursor, if any.
    #[must_use]
    pub fn topmost_target(&self, cursor_world: Vec2, tolerance_world: f32) -> Option<HitTarget> {
        self.topmost_handle(cursor_world, tolerance_world)
            .map(HitTarget::Handle)
            .or_else(|| {
                self.topmost_anchor(cursor_world, tolerance_world)
                    .map(HitTarget::Anchor)
            })
            .or_else(|| {
                self.topmost_segment(cursor_world, tolerance_world)
                    .map(HitTarget::Segment)
            })
            .or_else(|| {
                self.topmost_shape(cursor_world, tolerance_world)
                    .map(HitTarget::Shape)
            })
    }

    /// Return the topmost handle hit at the cursor, if any.
    #[must_use]
    pub fn topmost_handle(&self, cursor_world: Vec2, tolerance_world: f32) -> Option<HandleHit> {
        self.topmost_in_shapes(tolerance_world, |indexed, tolerance| {
            topmost_handle_in_shape(indexed, cursor_world, tolerance * tolerance)
        })
    }

    /// Return the topmost anchor hit at the cursor, if any.
    #[must_use]
    pub fn topmost_anchor(&self, cursor_world: Vec2, tolerance_world: f32) -> Option<AnchorHit> {
        self.topmost_in_shapes(tolerance_world, |indexed, tolerance| {
            topmost_anchor_in_shape(indexed, cursor_world, tolerance * tolerance)
        })
    }

    /// Return the topmost segment hit at the cursor, if any.
    #[must_use]
    pub fn topmost_segment(&self, cursor_world: Vec2, tolerance_world: f32) -> Option<SegmentHit> {
        self.topmost_in_shapes(tolerance_world, |indexed, tolerance| {
            segment_hit_in_shape(indexed, cursor_world, tolerance * tolerance)
        })
    }

    /// Return the topmost shape bounding-box hit at the cursor, if any.
    #[must_use]
    pub fn topmost_shape(&self, cursor_world: Vec2, tolerance_world: f32) -> Option<ShapeHit> {
        self.topmost_in_shapes(tolerance_world, |indexed, tolerance| {
            shape_hit_in_shape(indexed, cursor_world, tolerance)
        })
    }

    /// Convert the topmost model hit into the selection tool's result type.
    /// No matching target becomes `SelectPointerHit::None`.
    fn select_pointer_hit(&self, cursor_world: Vec2, tolerance_world: f32) -> SelectPointerHit {
        self.topmost_target(cursor_world, tolerance_world)
            .map_or(SelectPointerHit::None, map_target)
    }
}

/// Convert a public hit-test result to the matching selection-tool variant.
/// For example, an anchor hit keeps its shape and anchor identifiers.
const fn map_target(target: HitTarget) -> SelectPointerHit {
    match target {
        HitTarget::Handle(handle_hit) => SelectPointerHit::Handle(SelectHandleHit {
            shape_index: handle_hit.shape_index,
            shape_id: handle_hit.shape_id,
            anchor_index: handle_hit.anchor_index,
            kind: map_handle_kind(handle_hit.kind),
        }),
        HitTarget::Anchor(anchor_hit) => SelectPointerHit::Anchor(SelectAnchorHit {
            shape_index: anchor_hit.shape_index,
            shape_id: anchor_hit.shape_id,
            anchor_index: anchor_hit.anchor_index,
        }),
        HitTarget::Segment(segment_hit) => SelectPointerHit::Segment(SelectSegmentHit {
            shape_index: segment_hit.shape_index,
            shape_id: segment_hit.shape_id,
            seg_index: segment_hit.seg_index,
        }),
        HitTarget::Shape(shape_hit) => SelectPointerHit::Shape(SelectShapeHit {
            shape_index: shape_hit.shape_index,
            shape_id: shape_hit.shape_id,
        }),
    }
}

/// Convert the model handle direction to the selection tool's equivalent kind.
/// `HandleHitKind::Out` becomes `SelectHandleHitKind::Out`.
const fn map_handle_kind(kind: HandleHitKind) -> SelectHandleHitKind {
    match kind {
        HandleHitKind::In => SelectHandleHitKind::In,
        HandleHitKind::Out => SelectHandleHitKind::Out,
    }
}

/// Reject non-finite tolerances and clamp finite negative values to zero.
/// For example, `-1.0` becomes `Some(0.0)` and `f32::NAN` becomes `None`.
fn normalize_tolerance(tolerance_world: f32) -> Option<f32> {
    tolerance_world
        .is_finite()
        .then_some(tolerance_world.max(0.0))
}

/// Resolve the first handle hit found in `indexed.shape.path.anchors`.
///
/// This checks `anchor.handle_in` before `anchor.handle_out`, so
/// `HandleHitKind::In` wins deterministically over `HandleHitKind::Out` via
/// `build_handle_hit` even when the outgoing handle is closer. That ordering is
/// the expected user-visible behaviour when both handles fall within tolerance.
fn topmost_handle_in_shape(
    indexed: &IndexedShape<'_>,
    cursor_world: Vec2,
    tolerance_squared: f32,
) -> Option<HandleHit> {
    for (anchor_index, anchor) in indexed.shape.path.anchors.iter().enumerate() {
        if anchor
            .handle_in
            .is_some_and(|position| position.distance_squared(cursor_world) <= tolerance_squared)
        {
            return Some(build_handle_hit(indexed, anchor_index, HandleHitKind::In));
        }

        if anchor
            .handle_out
            .is_some_and(|position| position.distance_squared(cursor_world) <= tolerance_squared)
        {
            return Some(build_handle_hit(indexed, anchor_index, HandleHitKind::Out));
        }
    }

    None
}

/// Attach shape and anchor identity to a handle endpoint hit.
/// An outgoing endpoint keeps the `Out` kind alongside its shape and anchor IDs.
const fn build_handle_hit(
    indexed: &IndexedShape<'_>,
    anchor_index: usize,
    kind: HandleHitKind,
) -> HandleHit {
    HandleHit {
        shape_index: indexed.shape_index,
        shape_id: indexed.shape.id,
        anchor_index,
        kind,
    }
}

/// Find the first anchor within squared tolerance in one shape.
/// If several anchors qualify, the earliest anchor in path order wins.
fn topmost_anchor_in_shape(
    indexed: &IndexedShape<'_>,
    cursor_world: Vec2,
    tolerance_squared: f32,
) -> Option<AnchorHit> {
    for (anchor_index, anchor) in indexed.shape.path.anchors.iter().enumerate() {
        if anchor.pos.distance_squared(cursor_world) <= tolerance_squared {
            return Some(AnchorHit {
                shape_index: indexed.shape_index,
                shape_id: indexed.shape.id,
                anchor_index,
            });
        }
    }

    None
}

/// Return the closest matching segment in one shape with its document identity.
/// The result carries the chosen segment index and its owning shape's identity.
fn segment_hit_in_shape(
    indexed: &IndexedShape<'_>,
    cursor_world: Vec2,
    tolerance_squared: f32,
) -> Option<SegmentHit> {
    geometry::find_best_segment_hit(indexed.shape, cursor_world, tolerance_squared).map(
        |seg_index| SegmentHit {
            shape_index: indexed.shape_index,
            shape_id: indexed.shape.id,
            seg_index,
        },
    )
}

/// Return a bounding-box hit for one shape with its document identity.
/// A hit carries the shape index and stable ID used by selection.
fn shape_hit_in_shape(
    indexed: &IndexedShape<'_>,
    cursor_world: Vec2,
    tolerance: f32,
) -> Option<ShapeHit> {
    geometry::hit_test_shape_bbox(indexed.shape, cursor_world, tolerance).then_some(ShapeHit {
        shape_index: indexed.shape_index,
        shape_id: indexed.shape.id,
    })
}
