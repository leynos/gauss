//! Fallible element-click support for shell BDD scenarios.

use gpui::{Modifiers, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult};

/// Click the element identified by `selector`, then drain pending GPUI work.
pub fn click_selector(
    visual_cx: &mut VisualTestContext,
    selector: &'static str,
) -> TestSupportResult<()> {
    let bounds = visual_cx.debug_bounds(selector).ok_or_else(|| {
        TestSupportError::missing("element bounds", format!("selector {selector}"))
    })?;
    let position = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    visual_cx.simulate_click(position, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}
