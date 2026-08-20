//! Square-shape construction for GPUI integration tests.

use gauss::model::{
    Anchor, Document, PaintStyle, PathGeom, Rgba, SegmentKind, Shape, ShapeId, Vec2,
};
use test_support::{TestSupportError, TestSupportResult};

/// Appends a closed square spanning `min` to `max` and returns its generated ID.
///
/// The input points are opposite corners in document coordinates. The square is
/// inserted above the existing shapes with the standard test paint style.
///
/// # Errors
///
/// Returns an error when the document length cannot be represented as an `i32`
/// z-order value.
pub fn add_square(doc: &mut Document, min: Vec2, max: Vec2) -> TestSupportResult<ShapeId> {
    let shape = Shape {
        id: ShapeId::default(),
        z: i32::try_from(doc.len())
            .map_err(|error| TestSupportError::z_order_overflow("z-ordering", error))?,
        style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 2.0, None),
        path: PathGeom {
            anchors: vec![
                Anchor::new(min),
                Anchor::new(Vec2::new(max.x, min.y)),
                Anchor::new(max),
                Anchor::new(Vec2::new(min.x, max.y)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
        name: None,
        locked: false,
        hidden: false,
        gauss_metadata: Vec::new(),
    };
    Ok(doc.append_shape(shape))
}
