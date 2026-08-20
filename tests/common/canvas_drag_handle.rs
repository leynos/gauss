//! Canvas-drag scenario for handle movement tests.

use super::canvas_drag_values::{CanvasDragValues, canvas_drag_values};
use gauss::model::Vec2;
use gpui::{Bounds, Pixels, Point, VisualTestContext};
use test_support::TestSupportResult;

/// Canvas geometry and displacement used to exercise handle dragging.
#[derive(Clone, Copy, Debug)]
pub struct CanvasDragScenario {
    /// Bounds of the rendered canvas in window coordinates.
    pub bounds: Bounds<Pixels>,
    /// Padded point at the canvas's leading corner.
    pub first: Point<Pixels>,
    /// Padded point at the canvas's opposing corner.
    pub second: Point<Pixels>,
    /// Bounded document-space displacement represented by the drag.
    pub delta: Vec2,
}

/// Builds a handle-drag scenario for the supplied axis limits.
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
        bounds,
        first,
        second,
        drag_end: _drag_end,
        delta,
    } = canvas_drag_values(visual_cx, horizontal_limit, vertical_limit)?;
    Ok(CanvasDragScenario {
        bounds,
        first,
        second,
        delta,
    })
}
