//! Behavioural window-control coverage through `GpuiHarness`.

#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "shell_bdd/expect_equal.rs"]
mod expect_equal_support;
#[path = "shell_bdd/expect_true.rs"]
mod expect_true_support;
#[path = "shell_bdd/lifecycle.rs"]
mod lifecycle;
#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "shell_bdd/support.rs"]
mod support;

use std::cell::RefCell;

use expect_equal_support::expect_equal;
use expect_true_support::expect_true;
use gauss::ui::Phase0Shell;
use gpui::{Bounds, Modifiers, MouseButton, Pixels, TestAppContext, WindowBounds, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, fresh_shell_with, with_shell};
use test_support::TestSupportError;

thread_local! {
    static WINDOW_BOUNDS_BEFORE: RefCell<Option<WindowBounds>> = const { RefCell::new(None) };
    static TITLEBAR_DRAG_BOUNDS_BEFORE: RefCell<Option<Bounds<Pixels>>> = const { RefCell::new(None) };
}

fn reset_window_control_state() {
    WINDOW_BOUNDS_BEFORE.with(|cell| *cell.borrow_mut() = None);
    TITLEBAR_DRAG_BOUNDS_BEFORE.with(|cell| *cell.borrow_mut() = None);
}

#[given("a fresh non-maximized Phase 0 shell window")]
fn fresh_non_maximized_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_window_control_state();
    fresh_shell_with(cx, |view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(false));
        shell
    })?;
    with_shell(cx, |visual_cx, _view| {
        let bounds = visual_cx
            .debug_bounds("#titlebar-drag-region")
            .ok_or_else(|| {
                TestSupportError::missing("titlebar drag region", "initial shell render")
            })?;
        TITLEBAR_DRAG_BOUNDS_BEFORE.with(|cell| *cell.borrow_mut() = Some(bounds));
        Ok(())
    })?;
    Ok(())
}

#[given("a fresh maximized Phase 0 shell window")]
fn fresh_maximized_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    reset_window_control_state();
    fresh_shell_with(cx, |view_cx| {
        let mut shell = Phase0Shell::new(view_cx);
        shell.set_maximized_for_tests(Some(true));
        shell
    })
}

#[when("the window is changed to maximized")]
fn change_window_to_maximized(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _view_cx| {
                shell.set_maximized_for_tests(Some(true));
            });
        });
        lifecycle::ensure_initial_draw(visual_cx);
        Ok(())
    })
}

#[when("the window resize zone is dragged")]
fn drag_window_resize_zone(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        let before = visual_cx.update(|window, _app| window.window_bounds());
        WINDOW_BOUNDS_BEFORE.with(|cell| *cell.borrow_mut() = Some(before));
        let start = point(px(5.0), px(5.0));
        let target = point(px(50.0), px(50.0));
        visual_cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_move(target, MouseButton::Left, Modifiers::none());
        visual_cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the shell observes the maximized state")]
fn shell_observes_maximized_state(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let before_render = TITLEBAR_DRAG_BOUNDS_BEFORE
        .with(|cell| *cell.borrow())
        .ok_or_else(|| TestSupportError::missing("titlebar drag region", "before maximization"))?;
    with_shell(cx, |visual_cx, view| {
        let after_render = visual_cx
            .debug_bounds("#titlebar-drag-region")
            .ok_or_else(|| {
                TestSupportError::missing("titlebar drag region", "after maximization")
            })?;
        expect_true(
            after_render != before_render,
            "maximized state did not change the titlebar render bounds",
        )?;
        let is_maximized = visual_cx.update(|window, app| {
            view.read(app)
                .is_maximized_for_resize_borders_for_tests(window)
        });
        expect_true(is_maximized, "expected shell to observe maximized state")
    })
}

#[then("the window bounds are unchanged")]
fn window_bounds_are_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let before = WINDOW_BOUNDS_BEFORE
        .with(|cell| *cell.borrow())
        .ok_or_else(|| TestSupportError::missing("window bounds", "before resize-zone drag"))?;
    with_shell(cx, |visual_cx, _view| {
        let after = visual_cx.update(|window, _app| window.window_bounds());
        expect_equal(
            &after,
            &before,
            "window bounds changed while maximized resize was prevented",
        )
    })
}

#[scenario(
    path = "tests/features/shell_window_controls.feature",
    name = "Maximized state changes trigger a rerender",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn maximized_state_changes_trigger_rerender(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(
    path = "tests/features/shell_window_controls.feature",
    name = "Resize interaction is prevented while maximized",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn resize_interaction_is_prevented_while_maximized(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
