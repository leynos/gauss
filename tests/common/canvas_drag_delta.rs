//! Canvas-drag scenario for shape and preview movement tests.

use super::canvas_drag_values::{CanvasDragValues, canvas_drag_values};
use gauss::model::Vec2;
use gpui::{Pixels, Point, VisualTestContext};
use test_support::TestSupportResult;

/// Canvas points and displacement used by whole-shape drag tests.
#[derive(Clone, Copy, Debug)]
pub struct CanvasDragScenario {
    /// Padded point at the canvas's leading corner.
    pub first: Point<Pixels>,
    /// Padded point at the canvas's opposing corner.
    pub second: Point<Pixels>,
    /// Bounded document-space displacement between drag positions.
    pub delta: Vec2,
}

/// Builds a whole-shape drag scenario bounded by the supplied axis limits.
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
        delta,
    } = canvas_drag_values(visual_cx, horizontal_limit, vertical_limit)?;
    Ok(CanvasDragScenario {
        first,
        second,
        delta,
    })
}
