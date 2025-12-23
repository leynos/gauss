//! Tests for resize border visibility based on window state.

mod common;

use common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::TestAppContext;

/// Resize border element IDs that should be present when the window is not maximized.
const RESIZE_BORDER_SELECTORS: [&str; 8] = [
    "#resize-edge-top",
    "#resize-edge-bottom",
    "#resize-edge-left",
    "#resize-edge-right",
    "#resize-corner-tl",
    "#resize-corner-tr",
    "#resize-corner-bl",
    "#resize-corner-br",
];

#[gpui::test]
fn resize_borders_hidden_when_maximized(cx: &mut TestAppContext) {
    init_test_app(cx);

    // Create view with maximized state set before first render
    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(true));
        shell
    });
    ensure_initial_draw(visual_cx);

    // Verify all resize borders are hidden when maximized
    for selector in RESIZE_BORDER_SELECTORS {
        assert!(
            visual_cx.debug_bounds(selector).is_none(),
            "expected resize border {selector} to be hidden when maximized"
        );
    }
}

#[gpui::test]
fn resize_borders_visible_when_not_maximized(cx: &mut TestAppContext) {
    init_test_app(cx);

    // Create view with non-maximized state set before first render
    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(false));
        shell
    });
    ensure_initial_draw(visual_cx);

    // Verify all resize borders are visible when not maximized
    for selector in RESIZE_BORDER_SELECTORS {
        assert!(
            visual_cx.debug_bounds(selector).is_some(),
            "expected resize border {selector} to be visible when not maximized"
        );
    }
}

#[gpui::test]
fn resize_borders_appear_when_restoring_from_maximized(cx: &mut TestAppContext) {
    init_test_app(cx);

    // Start maximized
    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(true));
        shell
    });
    ensure_initial_draw(visual_cx);

    // Verify borders are initially hidden
    assert!(
        visual_cx.debug_bounds("#resize-edge-top").is_none(),
        "borders should be hidden when maximized"
    );

    // Restore from maximized
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.set_maximized_for_tests(Some(false));
            view_cx.notify();
        });
    });
    ensure_initial_draw(visual_cx);

    // Verify borders appear after restoring
    for selector in RESIZE_BORDER_SELECTORS {
        assert!(
            visual_cx.debug_bounds(selector).is_some(),
            "expected resize border {selector} to appear after restoring from maximized"
        );
    }
}
