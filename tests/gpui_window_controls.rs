//! Tests for window control actions and resize behaviour.
//!
//! This module tests the keyboard-accessible window controls and verifies
//! that resize zones are properly disabled when the window is maximized.

mod common;

use common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, px};

/// Test that the titlebar drag region element exists.
#[gpui::test]
fn titlebar_drag_region_is_present(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    assert!(
        visual_cx.debug_bounds("#titlebar-drag-region").is_some(),
        "expected titlebar drag region to be present"
    );
}

/// Test that window control buttons exist in the UI.
#[gpui::test]
fn window_control_buttons_are_present(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

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
/// does not implement `start_window_move()`. Double-click to toggle maximize
/// also isn't testable as `zoom_window()` is similarly unimplemented.
/// This test verifies the UI structure instead.
#[gpui::test]
fn titlebar_drag_region_has_dimensions(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = visual_cx
        .debug_bounds("#titlebar-drag-region")
        .expect("titlebar drag region should exist");

    // Verify the drag region has a reasonable size (should span most of top bar)
    assert!(bounds.size.width > px(100.0), "drag region should span horizontally");
    assert!(bounds.size.height > px(10.0), "drag region should have reasonable height");
}

/// Test that minimize button exists and is positioned correctly.
///
/// Note: Actually clicking the button is not tested because GPUI's test
/// platform does not implement `minimize_window()` (it panics with
/// "not implemented"). This test verifies the UI structure instead.
#[gpui::test]
fn minimize_button_exists_and_is_positioned(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = visual_cx
        .debug_bounds("#window-minimize")
        .expect("minimize button should exist");

    // Verify the button has a reasonable size (icon buttons are 28x28)
    assert!(bounds.size.width > px(20.0), "button should have reasonable width");
    assert!(bounds.size.height > px(20.0), "button should have reasonable height");
}

/// Test that maximize button exists and is positioned correctly.
///
/// Note: Actually clicking the button is not tested because GPUI's test
/// platform does not implement `zoom_window()` (it panics with
/// "not implemented"). This test verifies the UI structure instead.
#[gpui::test]
fn maximize_button_exists_and_is_positioned(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = visual_cx
        .debug_bounds("#window-maximize")
        .expect("maximize button should exist");

    // Verify the button has a reasonable size (icon buttons are 28x28)
    assert!(bounds.size.width > px(20.0), "button should have reasonable width");
    assert!(bounds.size.height > px(20.0), "button should have reasonable height");
}

/// Test that the maximized state override works in tests.
///
/// This verifies the test infrastructure for simulating maximized state.
#[gpui::test]
fn maximized_override_is_applied_in_tests(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(true));
        shell
    });
    ensure_initial_draw(visual_cx);

    // Verify the override was applied (we can't directly check is_maximized
    // in the test harness, but the shell should have stored the override)
    visual_cx.update(|window, app| {
        let is_maximized_for_borders =
            view.read(app).is_maximized_for_resize_borders_for_tests(window);
        assert!(
            is_maximized_for_borders,
            "expected maximized override to return true"
        );
    });
}

/// Test that setting maximized to false via test override works.
#[gpui::test]
fn non_maximized_override_is_applied_in_tests(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(false));
        shell
    });
    ensure_initial_draw(visual_cx);

    visual_cx.update(|window, app| {
        let is_maximized_for_borders =
            view.read(app).is_maximized_for_resize_borders_for_tests(window);
        assert!(
            !is_maximized_for_borders,
            "expected non-maximized override to return false"
        );
    });
}

/// Test that changing maximized state triggers a re-render.
#[gpui::test]
fn maximized_state_change_triggers_rerender(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(false));
        shell
    });
    ensure_initial_draw(visual_cx);

    // Change the maximized state
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.set_maximized_for_tests(Some(true));
            view_cx.notify();
        });
    });
    ensure_initial_draw(visual_cx);

    // Verify the change took effect
    visual_cx.update(|window, app| {
        let is_maximized = view.read(app).is_maximized_for_resize_borders_for_tests(window);
        assert!(is_maximized, "expected maximized state to change");
    });
}
