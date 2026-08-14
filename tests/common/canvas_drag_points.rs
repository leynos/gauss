//! Canvas-drag scenario for tests that only need endpoints.

use super::canvas_drag_values::{CanvasDragValues, canvas_drag_values};
use gpui::{Pixels, Point, VisualTestContext};
use test_support::TestSupportResult;

/// Opposing padded canvas points used by drag-based tests.
#[derive(Clone, Copy, Debug)]
pub struct CanvasDragScenario {
    /// Padded point at the canvas's leading corner.
    pub first: Point<Pixels>,
    /// Padded point at the canvas's opposing corner.
    pub second: Point<Pixels>,
}

/// Resolves opposing padded points for the supplied drag limits.
///
/// The limits are forwarded to the shared bounded-value calculation even
/// though this facade exposes only the resulting points.
///
/// # Errors
///
/// Propagates failure to locate the canvas bounds.
pub fn canvas_drag_scenario(
    visual_cx: &mut VisualTestContext,
    horizontal_limit: f32,
    vertical_limit: f32,
) -> TestSupportResult<CanvasDragScenario> {
    let CanvasDragValues {
        bounds: _bounds,
        first,
        second,
        drag_end: _drag_end,
        delta: _delta,
    } = canvas_drag_values(visual_cx, horizontal_limit, vertical_limit)?;
    Ok(CanvasDragScenario { first, second })
}
