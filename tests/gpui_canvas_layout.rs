//! GPUI headless integration tests for Phase 0 layout behaviour.
//!
//! This regression test asserts that the canvas area expands to fill a
//! meaningful portion of the window. We rely on debug bounds for the canvas
//! container, not the inner `Canvas` element itself.

use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, VisualTestContext};

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

#[gpui::test]
fn canvas_is_not_collapsed_to_a_tiny_height(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };

    let height = f32::from(bounds.size.height);
    assert!(
        height >= 200.0,
        "expected canvas to be reasonably tall; got height={height}"
    );
}
