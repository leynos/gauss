//! Reusable selection queries and coordinate helpers for integration tests.

use gauss_core::model::{Document, SelItem, Selection, Shape, ShapeId, Vec2, Viewport};
use gpui::{Pixels, Point, px};

use crate::{TestSupportError, TestSupportResult, math};

/// Return a shape or a contextual missing-data error.
///
/// # Errors
///
/// Returns [`TestSupportError::Missing`] when the document does not contain
/// `id`.
pub fn require_shape<'a>(
    document: &'a Document,
    id: ShapeId,
    context: &str,
) -> TestSupportResult<&'a Shape> {
    document
        .shape(id)
        .ok_or_else(|| TestSupportError::missing("shape", format!("shape {id:?}: {context}")))
}

/// Calculate the centre of a shape's anchor bounding box.
///
/// # Errors
///
/// Returns [`TestSupportError::Missing`] when the shape has no anchors.
pub fn shape_bbox_centre(shape: &Shape) -> TestSupportResult<Vec2> {
    let Some(first) = shape.path.anchors.first() else {
        return Err(TestSupportError::missing(
            "shape anchor",
            "computing bounding-box centre",
        ));
    };
    let (mut min_x, mut min_y) = (first.pos.x, first.pos.y);
    let (mut max_x, mut max_y) = (first.pos.x, first.pos.y);
    for anchor in shape.path.anchors.iter().skip(1) {
        min_x = min_x.min(anchor.pos.x);
        min_y = min_y.min(anchor.pos.y);
        max_x = max_x.max(anchor.pos.x);
        max_y = max_y.max(anchor.pos.y);
    }
    Ok(Vec2::new(
        math::midpoint(min_x, max_x),
        math::midpoint(min_y, max_y),
    ))
}

/// Convert a world-space point into GPUI screen coordinates.
#[must_use]
pub const fn viewport_to_screen_point(viewport: Viewport, world: Vec2) -> Point<Pixels> {
    let screen = viewport.world_to_screen(world);
    gpui::point(px(screen.x), px(screen.y))
}

/// Require a selection to contain exactly the expected shapes.
///
/// # Errors
///
/// Returns [`TestSupportError::Expectation`] when the selection contains a
/// different set of items.
pub fn require_selection_contains_shapes(
    selection: &Selection,
    expected: &[ShapeId],
    context: &str,
) -> TestSupportResult<()> {
    let has_expected_items = selection.items.len() == expected.len()
        && expected
            .iter()
            .all(|id| selection.contains(&SelItem::Shape(*id)));
    if !has_expected_items {
        return Err(TestSupportError::expectation(format!(
            "expected selected shapes {expected:?} ({context}); selection={selection:?}"
        )));
    }
    Ok(())
}
