//! Opposing canvas points for GPUI integration tests.

use super::canvas::CANVAS_PADDING_PX;
use gpui::{Pixels, Point, VisualTestContext, point, px};
use test_support::TestSupportResult;

use super::canvas_bounds::canvas_bounds;

/// Returns opposing canvas points inset by [`CANVAS_PADDING_PX`].
///
/// # Errors
///
/// Propagates failure to locate the canvas bounds.
pub fn canvas_points(
    visual_cx: &mut VisualTestContext,
) -> TestSupportResult<(Point<Pixels>, Point<Pixels>)> {
    let bounds = canvas_bounds(visual_cx)?;
    let first = point(
        bounds.origin.x + px(CANVAS_PADDING_PX),
        bounds.origin.y + px(CANVAS_PADDING_PX),
    );
    let second = point(
        bounds.origin.x + bounds.size.width - px(CANVAS_PADDING_PX),
        bounds.origin.y + bounds.size.height - px(CANVAS_PADDING_PX),
    );
    Ok((first, second))
}
