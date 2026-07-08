//! GPUI headless integration tests for Phase 0 draw-mode interactions.
//!
//! The draw/undo/redo behaviour that previously lived here as
//! `draw_click_adds_points_and_undo_removes` has been migrated to a Gherkin
//! scenario driven through the first-party GPUI harness; see
//! `tests/gpui_draw_undo_bdd.rs` and `tests/features/draw_undo.feature`.

mod common;

use common::{
    canvas_bounds, ensure_initial_draw, init_test_app, read_document, require_draw_shape,
};
use gauss::model::ShapeId;
use gauss::ui::{GpuiActivatePenTool, Phase0Shell};
use gpui::{Modifiers, TestAppContext, point, px};

#[gpui::test]
fn activate_pen_tool_from_manipulate_allows_drawing(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let click_in_manipulate = point(bounds.origin.x + px(8.0), bounds.origin.y + px(8.0));
    let click_in_draw = point(bounds.origin.x + px(24.0), bounds.origin.y + px(24.0));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let shape_count_before = read_document(visual_cx, &view).len();

    visual_cx.simulate_mouse_move(click_in_manipulate, None, Modifiers::none());
    visual_cx.simulate_click(click_in_manipulate, Modifiers::none());
    visual_cx.run_until_parked();

    let shape_count_after_manipulate_click = read_document(visual_cx, &view).len();
    assert_eq!(
        shape_count_after_manipulate_click, shape_count_before,
        "expected manipulate-mode click to keep shape count unchanged"
    );

    visual_cx.dispatch_action(GpuiActivatePenTool);
    visual_cx.run_until_parked();

    visual_cx.simulate_mouse_move(click_in_draw, None, Modifiers::none());
    visual_cx.simulate_click(click_in_draw, Modifiers::none());
    visual_cx.run_until_parked();

    let shape_count_after_draw_click = read_document(visual_cx, &view).len();
    assert_eq!(
        shape_count_after_draw_click,
        shape_count_before.saturating_add(1),
        "expected pen tool activation to restore draw-click shape insertion"
    );
}

#[gpui::test]
fn stale_active_path_is_recovered_when_pen_draws_new_shape(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let click_point = point(bounds.origin.x + px(12.0), bounds.origin.y + px(12.0));
    let stale_path = ShapeId::from_accesskit_node_id(9_999);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.set_draw_active_shape_for_tests(Some(stale_path));
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let active_before_click = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
    assert_eq!(
        active_before_click,
        Some(stale_path),
        "expected test setup to install a stale active path"
    );

    common::click_canvas_and_wait(visual_cx, click_point);

    let doc_after_click = read_document(visual_cx, &view);
    let shape = require_draw_shape(&doc_after_click, "after stale active path click")
        .expect("expected draw shape after recovering from stale active path");
    assert_eq!(
        shape.path.anchors.len(),
        1,
        "expected stale active path recovery to start a new single-anchor shape"
    );
    assert_eq!(
        shape.path.segments.len(),
        0,
        "expected no segments after first anchor in recovered draw flow"
    );

    let active_after_click = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
    assert_eq!(
        active_after_click,
        Some(shape.id),
        "expected active path to track the newly created shape after recovery"
    );
    assert_ne!(
        active_after_click,
        Some(stale_path),
        "expected stale active path id to be replaced during recovery"
    );

    let history_error = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(
        history_error.is_none(),
        "expected stale active path recovery to avoid surfacing history errors"
    );
}
