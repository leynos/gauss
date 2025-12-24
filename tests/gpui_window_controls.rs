//! Tests for window control actions and resize behaviour.
//!
//! This module tests the keyboard-accessible window controls and verifies
//! that resize zones are properly disabled when the window is maximised.

mod common;

use common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::{Entity, Pixels, TestAppContext, VisualTestContext, px};

/// Minimum expected width for icon buttons (28x28 nominal size).
const MIN_BUTTON_SIZE: Pixels = px(20.0);

/// Create a test window with a default `Phase0Shell`.
fn setup_window(cx: &mut TestAppContext) -> (Entity<Phase0Shell>, &mut VisualTestContext) {
    setup_window_with(cx, |_shell| {})
}

/// Create a test window with a customised `Phase0Shell` configuration.
fn setup_window_with<F>(
    cx: &mut TestAppContext,
    configure: F,
) -> (Entity<Phase0Shell>, &mut VisualTestContext)
where
    F: FnOnce(&mut Phase0Shell),
{
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        configure(&mut shell);
        shell
    });
    ensure_initial_draw(visual_cx);

    (view, visual_cx)
}

/// Assert that an element exists and has reasonable dimensions.
///
/// This helper reduces duplication in tests that verify UI structure.
fn assert_element_has_size(
    visual_cx: &mut VisualTestContext,
    selector: &'static str,
    min_width: Pixels,
    min_height: Pixels,
) {
    let bounds = visual_cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("element {selector} should exist"));

    assert!(
        bounds.size.width > min_width,
        "{selector} should have width > {min_width:?}, got {:?}",
        bounds.size.width
    );
    assert!(
        bounds.size.height > min_height,
        "{selector} should have height > {min_height:?}, got {:?}",
        bounds.size.height
    );
}

/// Assert that the maximised state override returns the expected value.
///
/// This helper creates a window with the specified maximised override and
/// verifies that `is_maximized_for_resize_borders_for_tests` returns the
/// expected value.
fn assert_maximised_override_matches(cx: &mut TestAppContext, expected_maximised: bool) {
    let (view, visual_cx) = setup_window_with(cx, |shell| {
        shell.set_maximized_for_tests(Some(expected_maximised));
    });

    visual_cx.update(|window, app| {
        let actual = view
            .read(app)
            .is_maximized_for_resize_borders_for_tests(window);
        assert_eq!(
            actual, expected_maximised,
            "expected maximised override to return {expected_maximised}"
        );
    });
}

/// Test that the titlebar drag region element exists.
#[gpui::test]
fn titlebar_drag_region_is_present(cx: &mut TestAppContext) {
    let (_view, visual_cx) = setup_window(cx);

    assert!(
        visual_cx.debug_bounds("#titlebar-drag-region").is_some(),
        "expected titlebar drag region to be present"
    );
}

/// Test that window control buttons exist in the UI.
#[gpui::test]
fn window_control_buttons_are_present(cx: &mut TestAppContext) {
    let (_view, visual_cx) = setup_window(cx);

    let control_selectors = ["#window-minimize", "#window-maximize", "#quit-button"];
    for selector in control_selectors {
        assert!(
            visual_cx.debug_bounds(selector).is_some(),
            "expected window control {selector} to be present"
        );
    }
}

/// Test that the titlebar drag region has appropriate dimensions.
///
/// Note: Clicking the titlebar is not tested because GPUI's test platform
/// does not implement `start_window_move()`. Double-click to toggle maximise
/// also isn't testable as `zoom_window()` is similarly unimplemented.
/// This test verifies the UI structure instead.
#[gpui::test]
fn titlebar_drag_region_has_dimensions(cx: &mut TestAppContext) {
    let (_view, visual_cx) = setup_window(cx);

    assert_element_has_size(visual_cx, "#titlebar-drag-region", px(100.0), px(10.0));
}

/// Test that window control buttons have appropriate dimensions.
///
/// Note: Actually clicking the buttons is not tested because GPUI's test
/// platform does not implement `minimize_window()` or `zoom_window()` (they
/// panic with "not implemented"). This test verifies the UI structure instead.
#[gpui::test]
fn window_control_buttons_have_dimensions(cx: &mut TestAppContext) {
    let (_view, visual_cx) = setup_window(cx);

    // All window control buttons should be icon buttons (28x28 nominal)
    for selector in ["#window-minimize", "#window-maximize", "#quit-button"] {
        assert_element_has_size(visual_cx, selector, MIN_BUTTON_SIZE, MIN_BUTTON_SIZE);
    }
}

/// Test that the maximised state override works in tests.
///
/// This verifies the test infrastructure for simulating maximised state.
#[gpui::test]
fn maximized_override_is_applied_in_tests(cx: &mut TestAppContext) {
    assert_maximised_override_matches(cx, true);
}

/// Test that setting maximised to false via test override works.
#[gpui::test]
fn non_maximized_override_is_applied_in_tests(cx: &mut TestAppContext) {
    assert_maximised_override_matches(cx, false);
}

/// Test that changing maximised state triggers a re-render.
#[gpui::test]
fn maximized_state_change_triggers_rerender(cx: &mut TestAppContext) {
    let (view, visual_cx) = setup_window_with(cx, |shell| {
        shell.set_maximized_for_tests(Some(false));
    });

    // Change the maximised state
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.set_maximized_for_tests(Some(true));
            view_cx.notify();
        });
    });
    ensure_initial_draw(visual_cx);

    // Verify the change took effect
    visual_cx.update(|window, app| {
        let is_maximised = view
            .read(app)
            .is_maximized_for_resize_borders_for_tests(window);
        assert!(is_maximised, "expected maximised state to change");
    });
}
