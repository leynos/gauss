//! GPUI headless integration tests for Phase 0 manipulate-mode interactions.

#[path = "common/gpui_history_drag_shape_undo.rs"]
mod common;

use common::{
    assert_shape_translated_by_delta, canvas_drag_scenario, draw_point, ensure_initial_draw,
    init_test_app, read_document, read_history_len, require_draw_shape, simulate_document_undo,
    switch_to_manipulate_mode_and_verify,
};
use gauss::model::{SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, Pixels, Point, TestAppContext, point, px};
use test_support::{TestSupportError, TestSupportResult, math};

/// Coordinates for a drag gesture from start to end point.
#[derive(Clone, Copy)]
struct DragCoordinates {
    start: Point<Pixels>,
    end: Point<Pixels>,
}

fn execute_shape_drag_and_verify_selection(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    shape_id: ShapeId,
    drag: DragCoordinates,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(drag.start, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    let selection_after_down = visual_cx.read(|app| view.read(app).selection().clone());
    let did_select = selection_after_down.items.iter().any(|item| match item {
        SelItem::Shape(id) => *id == shape_id,
        SelItem::Segment { shape, .. } => *shape == shape_id,
        _ => false,
    });
    if !did_select {
        return Err(TestSupportError::expectation(format!(
            "expected mouse down to select the draw shape; selection={selection_after_down:?}"
        )));
    }

    let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
    if !is_dragging {
        return Err(TestSupportError::expectation(
            "expected mouse down to start a drag gesture".to_owned(),
        ));
    }

    visual_cx.simulate_mouse_move(drag.end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(drag.end, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let stopped_dragging = visual_cx.read(|app| !view.read(app).is_dragging());
    if !stopped_dragging {
        return Err(TestSupportError::expectation(
            "expected mouse up to end the active drag gesture".to_owned(),
        ));
    }
    Ok(())
}

#[gpui::test]
fn dragging_demo_shape_moves_it_and_undo_restores(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    // Create a new shape in draw mode at deterministic in-canvas coordinates so
    // the hit-testing and drag logic can operate on it.
    let scenario =
        canvas_drag_scenario(visual_cx, 20.0, 10.0).expect("expected canvas drag scenario");
    draw_point(visual_cx, scenario.first);
    draw_point(visual_cx, scenario.second);

    let doc_before = read_document(visual_cx, &view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points")
        .expect("expected draw shape after drawing two points")
        .clone();

    // Switch to manipulate mode (Phase0Shell defaults to draw mode).
    switch_to_manipulate_mode_and_verify(visual_cx, &view, scenario.first)
        .expect("manipulate-mode click should not add a shape");

    let len_before = read_history_len(visual_cx, &view);

    let drag_start = {
        let start_x = math::midpoint(f32::from(scenario.first.x), f32::from(scenario.second.x));
        let start_y = math::midpoint(f32::from(scenario.first.y), f32::from(scenario.second.y));
        point(px(start_x), px(start_y))
    };
    let drag_end = point(
        drag_start.x + px(scenario.delta.x),
        drag_start.y + px(scenario.delta.y),
    );

    execute_shape_drag_and_verify_selection(
        visual_cx,
        &view,
        original_shape.id,
        DragCoordinates {
            start: drag_start,
            end: drag_end,
        },
    )
    .expect("expected shape drag to select, drag, and release");

    let len_after = read_history_len(visual_cx, &view);
    assert_eq!(
        len_after,
        len_before + 1,
        "expected exactly one undo entry for shape drag"
    );

    let doc_after_drag = read_document(visual_cx, &view);
    let moved_shape = require_draw_shape(&doc_after_drag, "after dragging draw shape")
        .expect("expected draw shape after drag");
    assert_shape_translated_by_delta(moved_shape, &original_shape, scenario.delta, "after drag")
        .expect("expected shape to move by drag delta");

    simulate_document_undo(visual_cx);
    let doc_after_undo = read_document(visual_cx, &view);
    let restored_shape =
        require_draw_shape(&doc_after_undo, "after undo").expect("expected draw shape after undo");
    assert_shape_translated_by_delta(restored_shape, &original_shape, Vec2::ZERO, "after undo")
        .expect("expected shape to return to original position");
}
