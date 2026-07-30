//! GPUI headless integration tests for Phase 0 layout behaviour.
//!
//! This regression test asserts that the canvas area expands to fill a
//! meaningful portion of the window. We rely on debug bounds for the canvas
//! container, not the inner `Canvas` element itself.

mod common;

use common::{canvas_bounds, ensure_initial_draw, init_test_app};

/// Minimum canvas height the Phase 0 chrome must leave for the canvas.
const MIN_CANVAS_HEIGHT_PX: f32 = 200.0;
use gauss::ui::Phase0Shell;
use gpui::TestAppContext;

#[gpui::test]
fn canvas_is_not_collapsed_to_a_tiny_height(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");

    let height = f32::from(bounds.size.height);
    assert!(
        height >= MIN_CANVAS_HEIGHT_PX,
        "expected canvas to be reasonably tall; got height={height}"
    );
}
