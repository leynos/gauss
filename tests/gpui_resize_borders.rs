//! Tests for resize border visibility based on window state.
//!
//! On Linux, resize hit regions are provided by `GaussWindowBorder` in the
//! window shadow area, so the inner `resize_borders()` function returns empty.
//! These tests only run on non-Linux platforms where the inner resize borders
//! are used.

#[cfg(not(target_os = "linux"))]
mod common;

#[cfg(not(target_os = "linux"))]
use common::{ensure_initial_draw, init_test_app};
#[cfg(not(target_os = "linux"))]
use gauss::ui::Phase0Shell;
#[cfg(not(target_os = "linux"))]
use gpui::TestAppContext;

/// Resize border element IDs that should be present when the window is not maximized.
#[cfg(not(target_os = "linux"))]
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

/// On Linux, resize borders are provided by `GaussWindowBorder`, not the inner
/// `resize_borders()` function.
#[cfg(not(target_os = "linux"))]
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

/// On Linux, resize borders are provided by `GaussWindowBorder`, not the inner
/// `resize_borders()` function.
#[cfg(not(target_os = "linux"))]
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

/// On Linux, resize borders are provided by `GaussWindowBorder`, not the inner
/// `resize_borders()` function.
#[cfg(not(target_os = "linux"))]
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
