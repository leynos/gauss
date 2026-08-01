//! Canvas-drag scenario for tests that only need endpoints.

use super::canvas_drag_values::canvas_drag_values;
use gpui::{Pixels, Point, VisualTestContext};
use test_support::TestSupportResult;

#[derive(Clone, Copy, Debug)]
pub struct CanvasDragScenario {
    pub first: Point<Pixels>,
    pub second: Point<Pixels>,
}

pub fn canvas_drag_scenario(
    visual_cx: &mut VisualTestContext,
    horizontal_limit: f32,
    vertical_limit: f32,
) -> TestSupportResult<CanvasDragScenario> {
    let (_bounds, first, second, _drag_end, _delta) =
        canvas_drag_values(visual_cx, horizontal_limit, vertical_limit)?;
    Ok(CanvasDragScenario { first, second })
}
