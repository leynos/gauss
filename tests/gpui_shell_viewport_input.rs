//! Behavioural coverage for shell viewport input through `GpuiHarness`.

#[path = "common/gpui_shell_viewport_input.rs"]
mod common;
#[path = "shell_bdd/expect_equal.rs"]
mod expect_equal_support;
#[path = "shell_bdd/expect_true.rs"]
mod expect_true_support;
#[path = "shell_bdd/support.rs"]
mod support;

use std::cell::RefCell;

use expect_equal_support::expect_equal;
use expect_true_support::expect_true;
use gauss::model::Vec2;
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, fresh_shell_with, with_shell};
use test_support::{TestSupportError, math};

#[derive(Default)]
struct InputState {
    pan_before: Option<Vec2>,
    zoom_before: Option<f32>,
    cursor: Option<Vec2>,
    world_before: Option<Vec2>,
}

thread_local! {
    static INPUT_STATE: RefCell<InputState> = RefCell::new(InputState::default());
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    INPUT_STATE.with(|cell| *cell.borrow_mut() = InputState::default());
    fresh_shell_with(cx, Phase0Shell::new);
    with_shell(cx, |_visual_cx, _view| Ok(()))?;
    Ok(())
}

fn canvas_position(
    visual_cx: &mut gpui::VisualTestContext,
) -> Result<gpui::Point<gpui::Pixels>, TestSupportError> {
    let bounds = common::canvas_bounds(visual_cx)?;
    Ok(point(
        bounds.origin.x + px(math::midpoint(0.0, f32::from(bounds.size.width))),
        bounds.origin.y + px(math::midpoint(0.0, f32::from(bounds.size.height))),
    ))
}

#[when("the scroll wheel moves by 10 pixels right and 20 pixels up")]
fn pan_with_scroll_wheel(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let before = visual_cx.read(|app| view.read(app).viewport());
        INPUT_STATE.with(|cell| cell.borrow_mut().pan_before = Some(before.pan));
        let position = canvas_position(visual_cx)?;
        visual_cx.simulate_event(ScrollWheelEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(10.0), px(-20.0))),
            modifiers: Modifiers::none(),
            touch_phase: TouchPhase::Moved,
        });
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[when("the secondary-modified scroll wheel zooms at the canvas cursor")]
fn zoom_with_secondary_scroll_wheel(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let before = visual_cx.read(|app| view.read(app).viewport());
        let position = canvas_position(visual_cx)?;
        let cursor = Vec2::new(f32::from(position.x), f32::from(position.y));
        INPUT_STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            state.zoom_before = Some(before.zoom());
            state.cursor = Some(cursor);
            state.world_before = Some(before.screen_to_world(cursor));
        });
        visual_cx.simulate_event(ScrollWheelEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.0), px(120.0))),
            modifiers: Modifiers::secondary_key(),
            touch_phase: TouchPhase::Moved,
        });
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the viewport is panned 10 pixels right and 20 pixels up")]
fn viewport_is_panned(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let before = INPUT_STATE
        .with(|cell| cell.borrow().pan_before)
        .ok_or_else(|| {
            TestSupportError::missing("initial viewport pan", "scroll action must run first")
        })?;
    with_shell(cx, |visual_cx, view| {
        let after = visual_cx.read(|app| view.read(app).viewport());
        expect_equal(
            &after.pan,
            &before.add(Vec2::new(10.0, -20.0)),
            "viewport pan",
        )
    })
}

#[then("the viewport zoom increases")]
fn viewport_zoom_increases(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let before = INPUT_STATE
        .with(|cell| cell.borrow().zoom_before)
        .ok_or_else(|| {
            TestSupportError::missing("initial viewport zoom", "zoom action must run first")
        })?;
    with_shell(cx, |visual_cx, view| {
        let after = visual_cx.read(|app| view.read(app).viewport());
        expect_true(after.zoom() > before, "expected viewport zoom to increase")
    })
}

#[then("the world point beneath the cursor is preserved")]
fn world_point_is_preserved(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let (cursor_option, world_before_option) = INPUT_STATE.with(|cell| {
        let state = cell.borrow();
        (state.cursor, state.world_before)
    });
    let cursor = cursor_option.ok_or_else(|| TestSupportError::missing("cursor", "zoom action"))?;
    let world_before = world_before_option
        .ok_or_else(|| TestSupportError::missing("world point", "before zoom action"))?;
    with_shell(cx, |visual_cx, view| {
        let after = visual_cx.read(|app| view.read(app).viewport());
        let drift = world_before.distance(after.screen_to_world(cursor));
        expect_true(
            drift < 0.01,
            format!("world point under cursor drifted by {drift}"),
        )
    })
}

#[scenario(
    path = "tests/features/shell_viewport_input.feature",
    name = "Scroll wheel pans the viewport",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn scroll_wheel_pans_viewport(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(
    path = "tests/features/shell_viewport_input.feature",
    name = "Secondary scroll wheel zooms around the cursor",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn secondary_scroll_wheel_zooms_around_cursor(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
