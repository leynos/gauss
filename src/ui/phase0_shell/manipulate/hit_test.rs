//! Hit-testing support for Phase 0 manipulate mode.
//!
//! Phase 0's editor model is intentionally simple, so we use a few pragmatic
//! hit-tests:
//!
//! - handles (if present) are hit-tested first,
//! - then anchors,
//! - then a loose shape bounding-box check.

use crate::model::{Document, Shape, ShapeId, Vec2};

#[derive(Clone, Copy)]
pub(super) struct AnchorHit {
    pub(super) shape_index: usize,
    pub(super) shape_id: ShapeId,
    pub(super) anchor_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HandleHitKind {
    In,
    Out,
}

#[derive(Clone, Copy)]
pub(super) struct HandleHit {
    pub(super) shape_index: usize,
    pub(super) shape_id: ShapeId,
    pub(super) anchor_index: usize,
    pub(super) kind: HandleHitKind,
}

pub(super) fn hit_test_topmost_handle(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<HandleHit> {
    let tolerance_squared = tolerance_world * tolerance_world;
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(shape_index, shape)| {
            shape
                .path
                .anchors
                .iter()
                .enumerate()
                .find_map(|(anchor_index, anchor)| {
                    if anchor
                        .handle_in
                        .is_some_and(|p| p.distance_squared(cursor_world) <= tolerance_squared)
                    {
                        return Some(HandleHit {
                            shape_index,
                            shape_id: shape.id,
                            anchor_index,
                            kind: HandleHitKind::In,
                        });
                    }

                    if anchor
                        .handle_out
                        .is_some_and(|p| p.distance_squared(cursor_world) <= tolerance_squared)
                    {
                        return Some(HandleHit {
                            shape_index,
                            shape_id: shape.id,
                            anchor_index,
                            kind: HandleHitKind::Out,
                        });
                    }

                    None
                })
        })
}

pub(super) fn hit_test_topmost_anchor(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<AnchorHit> {
    let tolerance_squared = tolerance_world * tolerance_world;
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(shape_index, shape)| {
            shape
                .path
                .anchors
                .iter()
                .enumerate()
                .find_map(|(anchor_index, anchor)| {
                    (anchor.pos.distance_squared(cursor_world) <= tolerance_squared).then_some(
                        AnchorHit {
                            shape_index,
                            shape_id: shape.id,
                            anchor_index,
                        },
                    )
                })
        })
}

pub(super) fn hit_test_topmost_shape(
    doc: &Document,
    cursor_world: Vec2,
    tolerance_world: f32,
) -> Option<(usize, ShapeId)> {
    doc.shapes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, shape)| {
            hit_test_shape_bbox(shape, cursor_world, tolerance_world).then_some((index, shape.id))
        })
}

fn hit_test_shape_bbox(shape: &Shape, cursor_world: Vec2, tolerance_world: f32) -> bool {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for anchor in &shape.path.anchors {
        min_x = min_x.min(anchor.pos.x);
        min_y = min_y.min(anchor.pos.y);
        max_x = max_x.max(anchor.pos.x);
        max_y = max_y.max(anchor.pos.y);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return false;
    }

    cursor_world.x >= (min_x - tolerance_world)
        && cursor_world.x <= (max_x + tolerance_world)
        && cursor_world.y >= (min_y - tolerance_world)
        && cursor_world.y <= (max_y + tolerance_world)
}
