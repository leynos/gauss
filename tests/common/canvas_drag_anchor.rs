//! Canvas-drag scenario for anchor movement tests.

use super::canvas_drag_values::canvas_drag_values;
use gauss::model::Vec2;
use gpui::{Pixels, Point, VisualTestContext};
use test_support::TestSupportResult;

#[derive(Clone, Copy, Debug)]
pub struct CanvasDragScenario {
    pub first: Point<Pixels>,
    pub second: Point<Pixels>,
    pub drag_end: Point<Pixels>,
    pub delta: Vec2,
}

pub fn canvas_drag_scenario(
    visual_cx: &mut VisualTestContext,
    horizontal_limit: f32,
    vertical_limit: f32,
) -> TestSupportResult<CanvasDragScenario> {
    let (_bounds, first, second, drag_end, delta) =
        canvas_drag_values(visual_cx, horizontal_limit, vertical_limit)?;
    Ok(CanvasDragScenario {
        first,
        second,
        drag_end,
        delta,
    })
}
