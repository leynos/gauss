//! Step definitions for shell mode indicator BDD tests.

// Keep `expect_used` lint strict in this module.

use super::world;
use crate::common::{ensure_initial_draw, init_test_app};
use gauss::ui::Phase0Shell;
use gpui::{App, TestAppContext, VisualTestContext};
use rstest_bdd_macros::{given, then, when};
use test_support::{TestSupportError, TestSupportResult};

#[given("the Phase 0 shell is open")]
fn given_shell_open(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> TestSupportResult<()> {
    init_test_app(cx);
    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let window = cx
        .windows()
        .first()
        .copied()
        .ok_or_else(|| TestSupportError::expectation("window after add_window_view"))?;
    world::with_world(|cell| {
        let mut world_ref = cell.borrow_mut();
        world_ref.shell = Some(view);
        world_ref.window = Some(window);
    });
    Ok(())
}

#[when("I press the \"tab\" key")]
fn when_press_tab(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> TestSupportResult<()> {
    let window = world::with_world(|w| w.borrow().window)
        .ok_or_else(|| TestSupportError::expectation("window not set"))?;
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    visual_cx.simulate_keystrokes("tab");
    Ok(())
}

#[when("I enter manipulate mode")]
fn when_enter_manipulate(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> TestSupportResult<()> {
    let (shell, window) = world::with_world(|cell| {
        let world_ref = cell.borrow();
        let shell = world_ref
            .shell
            .clone()
            .ok_or_else(|| TestSupportError::expectation("shell not open"))?;
        let window = world_ref
            .window
            .ok_or_else(|| TestSupportError::expectation("window not set"))?;
        Ok((shell, window))
    })?;
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    visual_cx.update(|_window, app: &mut App| {
        shell.update(app, |s: &mut Phase0Shell, view_cx| {
            s.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    cx.run_until_parked();
    Ok(())
}

#[then("the mode indicator reads \"{expected}\"")]
fn then_mode_indicator_reads(
    #[from(rstest_bdd_harness_context)] cx: &TestAppContext,
    expected: String,
) -> TestSupportResult<()> {
    let shell = world::with_world(|cell| {
        cell.borrow()
            .shell
            .clone()
            .ok_or_else(|| TestSupportError::expectation("shell not open"))
    })?;
    let actual = cx.read(|app| shell.read(app).mode_status_line_for_tests());
    if actual == expected {
        Ok(())
    } else {
        Err(TestSupportError::expectation(format!(
            "expected mode indicator '{expected}', got '{actual}'",
        )))
    }
}
