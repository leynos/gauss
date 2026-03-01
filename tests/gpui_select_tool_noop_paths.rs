//! GPUI headless integration tests for `SelectTool` unhappy and edge paths.

mod common;

use common::{
    canvas_bounds, canvas_drag_scenario, draw_point, ensure_initial_draw, init_test_app,
    read_history_len, read_selection, switch_to_manipulate_mode_and_verify,
};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, point, px};
use test_support::math;

#[gpui::test]
fn right_click_in_manipulate_mode_is_noop(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let click_point = point(bounds.origin.x + px(8.0), bounds.origin.y + px(8.0));

    switch_to_manipulate_mode_and_verify(visual_cx, &view, click_point);

    let selection_before = read_selection(visual_cx, &view);

    visual_cx.simulate_mouse_down(click_point, MouseButton::Right, Modifiers::none());
    visual_cx.simulate_mouse_up(click_point, MouseButton::Right, Modifiers::none());
    visual_cx.run_until_parked();

    let selection_after = read_selection(visual_cx, &view);
    let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());

    assert_eq!(
        selection_after, selection_before,
        "right click should not change selection in manipulate mode"
    );
    assert!(!is_dragging, "right click should not start drag state");
}

#[gpui::test]
fn zero_delta_drag_does_not_create_history_entry(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let scenario =
        canvas_drag_scenario(visual_cx, 18.0, 12.0).expect("expected canvas drag scenario");
    draw_point(visual_cx, scenario.first);
    draw_point(visual_cx, scenario.second);

    switch_to_manipulate_mode_and_verify(visual_cx, &view, scenario.first);

    let history_before = read_history_len(visual_cx, &view);

    let start = point(
        px(math::midpoint(
            f32::from(scenario.first.x),
            f32::from(scenario.second.x),
        )),
        px(math::midpoint(
            f32::from(scenario.first.y),
            f32::from(scenario.second.y),
        )),
    );

    visual_cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_move(start, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(start, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let history_after = read_history_len(visual_cx, &view);
    let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());

    assert_eq!(
        history_after, history_before,
        "zero-delta drag should not add document history entries"
    );
    assert!(!is_dragging, "drag state should be idle after mouse up");
}
