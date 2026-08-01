//! Anchor-to-canvas conversion for GPUI integration tests.

use super::canvas::CANVAS_PADDING_PX;
use gauss::model::Vec2;
use gpui::{Bounds, Pixels, Point, point, px};

pub fn anchor_to_canvas_point(
    bounds: &Bounds<Pixels>,
    anchor: Vec2,
    reference: Point<Pixels>,
) -> Point<Pixels> {
    let expected_local = Vec2::new(CANVAS_PADDING_PX, CANVAS_PADDING_PX);
    let expected_abs = Vec2::new(f32::from(reference.x), f32::from(reference.y));
    let use_local =
        anchor.distance_squared(expected_local) <= anchor.distance_squared(expected_abs);
    if use_local {
        point(
            bounds.origin.x + px(anchor.x),
            bounds.origin.y + px(anchor.y),
        )
    } else {
        point(px(anchor.x), px(anchor.y))
    }
}
