//! Canvas-bound lookup for GPUI integration tests.

use gpui::{Bounds, Pixels, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

pub fn canvas_bounds(visual_cx: &mut VisualTestContext) -> TestSupportResult<Bounds<Pixels>> {
    visual_cx.debug_bounds("#phase0-canvas").ok_or_else(|| {
        TestSupportError::missing("canvas debug bounds (#phase0-canvas)", "after drawing")
    })
}
