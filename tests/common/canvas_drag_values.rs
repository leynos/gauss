//! Raw bounded canvas-drag values for domain-specific test scenarios.
#![expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]

use super::{canvas::CANVAS_PADDING_PX, canvas_bounds::canvas_bounds};
use gauss::model::Vec2;
use gpui::{Bounds, Pixels, Point, VisualTestContext, point, px};
use test_support::TestSupportResult;

pub type CanvasDragValues = (
    Bounds<Pixels>,
    Point<Pixels>,
    Point<Pixels>,
    Point<Pixels>,
    Vec2,
);

pub fn canvas_drag_values(
    visual_cx: &mut VisualTestContext,
    horizontal_limit: f32,
    vertical_limit: f32,
) -> TestSupportResult<CanvasDragValues> {
    let bounds = canvas_bounds(visual_cx)?;
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let first = point(
        bounds.origin.x + px(CANVAS_PADDING_PX),
        bounds.origin.y + px(CANVAS_PADDING_PX),
    );
    let second = point(
        bounds.origin.x + px((width - CANVAS_PADDING_PX).max(CANVAS_PADDING_PX)),
        bounds.origin.y + px((height - CANVAS_PADDING_PX).max(CANVAS_PADDING_PX)),
    );
    let max_horizontal_delta = (width - (2.0 * CANVAS_PADDING_PX)).max(1.0);
    let max_vertical_delta = (height - (2.0 * CANVAS_PADDING_PX)).max(1.0);
    let horizontal_delta = max_horizontal_delta.min(horizontal_limit);
    let vertical_delta = max_vertical_delta.min(vertical_limit);
    let drag_end = point(first.x + px(horizontal_delta), first.y + px(vertical_delta));

    Ok((
        bounds,
        first,
        second,
        drag_end,
        Vec2::new(horizontal_delta, vertical_delta),
    ))
}
