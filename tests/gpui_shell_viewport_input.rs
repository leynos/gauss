//! GPUI headless integration tests for viewport input mapping.
//!
//! These tests exercise scroll wheel event dispatch against the Phase 0 view,
//! asserting the viewport pan/zoom state updates as expected.

#![expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]
mod common;

use common::{canvas_bounds, ensure_initial_draw, init_test_app};
use gauss::model::Vec2;
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, point, px};
use test_support::TestSupportResult;

fn canvas_position(
    visual_cx: &mut gpui::VisualTestContext,
) -> TestSupportResult<gpui::Point<gpui::Pixels>> {
    let bounds = canvas_bounds(visual_cx)?;
    Ok(point(
        bounds.origin.x + px(common::CANVAS_PADDING_PX),
        bounds.origin.y + px(common::CANVAS_PADDING_PX),
    ))
}

#[gpui::test]
fn scroll_wheel_pans_viewport(cx: &mut TestAppContext) {
    init_test_app(cx);

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);

        let before = visual_cx.read(|app| view.read(app).viewport());
        let position = canvas_position(visual_cx).expect("canvas bounds should be available");

        visual_cx.simulate_event(ScrollWheelEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(10.0), px(-20.0))),
            modifiers: Modifiers::none(),
            touch_phase: TouchPhase::Moved,
        });

        let after = visual_cx.read(|app| view.read(app).viewport());
        let expected_pan = before.pan.add(Vec2::new(10.0, -20.0));
        assert_eq!(after.pan, expected_pan);
        view
    };
    cx.run_until_parked();

    let viewport = cx.read(|app| view.read(app).viewport());
    assert_eq!(viewport.pan, Vec2::new(10.0, -20.0));
}

#[gpui::test]
fn secondary_scroll_wheel_zooms_around_cursor(cx: &mut TestAppContext) {
    init_test_app(cx);

    let view: gpui::Entity<Phase0Shell> = {
        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);

        let before = visual_cx.read(|app| view.read(app).viewport());
        let position = canvas_position(visual_cx).expect("canvas bounds should be available");
        let cursor = Vec2::new(f32::from(position.x), f32::from(position.y));
        let world_before = before.screen_to_world(cursor);

        visual_cx.simulate_event(ScrollWheelEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.0), px(120.0))),
            modifiers: Modifiers::secondary_key(),
            touch_phase: TouchPhase::Moved,
        });

        let after = visual_cx.read(|app| view.read(app).viewport());
        assert!(
            (after.zoom() - before.zoom()).abs() > 0.0001,
            "zoom should change when using the secondary scroll modifier"
        );

        let world_after = after.screen_to_world(cursor);
        let drift = world_before.distance(world_after);
        assert!(
            drift < 0.01,
            "zoom should preserve world point under cursor (drift: {drift})"
        );

        view
    };
    cx.run_until_parked();

    let viewport = cx.read(|app| view.read(app).viewport());
    assert!(viewport.zoom() > 1.0);
}
