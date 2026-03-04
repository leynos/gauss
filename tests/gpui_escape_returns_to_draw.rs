//! GPUI headless integration test for toggling back to draw mode.
//!
//! Phase 0 uses `Escape` as the mode toggle:
//! - Draw → Manipulate: commit the current open path and enter manipulate mode.
//! - Manipulate → Draw: return to draw mode so clicks place points again.

mod common;

use common::{
    CanvasDragScenario, canvas_bounds, canvas_drag_scenario, draw_point, ensure_initial_draw,
    init_test_app, read_document, read_history_len, require_draw_shape, simulate_escape,
    switch_to_manipulate_mode_and_verify,
};
use gauss::model::Anchor;
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, Pixels, Point, TestAppContext, VisualTestContext, point, px};
use test_support::math;

fn read_shape_count(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> usize {
    visual_cx.read(|app| view.read(app).document().len())
}

fn read_draw_shape_anchors(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    context: &str,
) -> Vec<Anchor> {
    let doc = read_document(visual_cx, view);
    let shape = match require_draw_shape(&doc, context) {
        Ok(shape) => shape,
        Err(error) => panic!("expected draw shape to exist: {error}"),
    };
    shape.path.anchors.clone()
}

fn drag_start_and_preview_points(scenario: &CanvasDragScenario) -> (Point<Pixels>, Point<Pixels>) {
    let drag_start = point(
        px(math::midpoint(
            f32::from(scenario.first.x),
            f32::from(scenario.second.x),
        )),
        px(math::midpoint(
            f32::from(scenario.first.y),
            f32::from(scenario.second.y),
        )),
    );
    let drag_preview = point(
        drag_start.x + px(scenario.delta.x),
        drag_start.y + px(scenario.delta.y),
    );
    (drag_start, drag_preview)
}

#[gpui::test]
fn escape_in_manipulate_returns_to_draw(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");

    let click_point = point(bounds.origin.x + px(10.0), bounds.origin.y + px(10.0));

    // Arrange: force manipulate mode so a click does not place points.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let shapes_before = read_shape_count(visual_cx, &view);
    visual_cx.simulate_mouse_move(click_point, None, Modifiers::none());
    visual_cx.simulate_click(click_point, Modifiers::none());
    visual_cx.run_until_parked();

    let shapes_after_click_in_manipulate = read_shape_count(visual_cx, &view);
    assert_eq!(
        shapes_after_click_in_manipulate, shapes_before,
        "expected manipulate mode click to not create new shapes"
    );

    // Act: press escape to return to draw mode, and click again.
    simulate_escape(visual_cx);
    visual_cx.simulate_mouse_move(click_point, None, Modifiers::none());
    visual_cx.simulate_click(click_point, Modifiers::none());
    visual_cx.run_until_parked();

    // Assert: draw mode click placed the first point of a new open path.
    let shapes_after_click_in_draw = read_shape_count(visual_cx, &view);
    assert_eq!(
        shapes_after_click_in_draw,
        shapes_before.saturating_add(1),
        "expected draw mode click to create a new shape"
    );
}

#[gpui::test]
fn escape_during_manipulate_drag_preview_cancels_without_history_commit(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let scenario =
        canvas_drag_scenario(visual_cx, 20.0, 10.0).expect("expected canvas drag scenario");
    draw_point(visual_cx, scenario.first);
    draw_point(visual_cx, scenario.second);

    switch_to_manipulate_mode_and_verify(visual_cx, &view, scenario.first);

    let history_before_drag = read_history_len(visual_cx, &view);
    let anchors_before_escape =
        read_draw_shape_anchors(visual_cx, &view, "before escape during drag preview");
    let (drag_start, drag_preview) = drag_start_and_preview_points(&scenario);

    visual_cx.simulate_mouse_down(drag_start, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    let is_dragging_after_down = visual_cx.read(|app| view.read(app).is_dragging());
    assert!(
        is_dragging_after_down,
        "expected mouse down to enter drag preview state"
    );

    visual_cx.simulate_mouse_move(drag_preview, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    assert_eq!(
        read_history_len(visual_cx, &view),
        history_before_drag,
        "preview move should not commit document history"
    );

    simulate_escape(visual_cx);

    let is_dragging_after_escape = visual_cx.read(|app| view.read(app).is_dragging());
    assert!(
        !is_dragging_after_escape,
        "escape should clear active drag preview state"
    );

    let history_after_escape = read_history_len(visual_cx, &view);
    assert_eq!(
        history_after_escape, history_before_drag,
        "escape during preview should not create history entries"
    );

    let anchors_after_escape =
        read_draw_shape_anchors(visual_cx, &view, "after escape during drag preview");
    assert_eq!(
        anchors_after_escape, anchors_before_escape,
        "escape during drag preview should not change shape geometry"
    );
}
