//! GPUI headless integration tests for Phase 0 handle dragging.

mod common;

use common::{
    CanvasDragScenario, anchor_to_canvas_point, assert_vec2_close, canvas_drag_scenario,
    ensure_initial_draw, init_test_app, read_document, require_draw_shape, simulate_document_undo,
    simulate_escape,
};
use gauss::model::{SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};

fn toggle_bezier_auto(visual_cx: &mut VisualTestContext) {
    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();
}

fn draw_two_point_bezier_path(visual_cx: &mut VisualTestContext, scenario: CanvasDragScenario) {
    toggle_bezier_auto(visual_cx);
    visual_cx.simulate_mouse_move(scenario.first, None, Modifiers::none());
    visual_cx.simulate_click(scenario.first, Modifiers::none());
    visual_cx.simulate_mouse_move(scenario.second, None, Modifiers::none());
    visual_cx.simulate_click(scenario.second, Modifiers::none());
    visual_cx.run_until_parked();
}

fn model_point_to_canvas_point(
    bounds: gpui::Bounds<gpui::Pixels>,
    model: Vec2,
    reference: gpui::Point<gpui::Pixels>,
) -> gpui::Point<gpui::Pixels> {
    anchor_to_canvas_point(&bounds, model, reference)
}

fn assert_handle_selection_includes_shape(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    shape_id: ShapeId,
) {
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection.contains(&SelItem::Shape(shape_id)),
        "expected handle interaction to keep the shape selected; selection={selection:?}"
    );
    assert!(
        selection.contains(&SelItem::HandleOut {
            shape: shape_id,
            anchor: 0,
        }),
        "expected mouse down to select the handle; selection={selection:?}"
    );
}

#[gpui::test]
fn dragging_handle_moves_it_and_undo_restores(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let scenario = canvas_drag_scenario(visual_cx, 18.0, 10.0);
    draw_two_point_bezier_path(visual_cx, scenario);

    let doc_before = read_document(visual_cx, &view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points").clone();
    let Some(first_anchor) = original_shape.path.anchors.first().cloned() else {
        panic!("expected first anchor");
    };
    let Some(original_handle_out) = first_anchor.handle_out else {
        panic!("expected handle_out on first anchor in bezier mode");
    };

    let handle_start =
        model_point_to_canvas_point(scenario.bounds, original_handle_out, scenario.first);
    let handle_end = point(
        handle_start.x + px(scenario.delta.x),
        handle_start.y + px(scenario.delta.y),
    );

    simulate_escape(visual_cx);

    visual_cx.simulate_mouse_down(handle_start, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    assert_handle_selection_includes_shape(visual_cx, &view, original_shape.id);

    visual_cx.simulate_mouse_move(handle_end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(handle_end, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let doc_after_drag = read_document(visual_cx, &view);
    let moved_shape = require_draw_shape(&doc_after_drag, "after dragging handle");
    let Some(moved_anchor) = moved_shape.path.anchors.first() else {
        panic!("expected first anchor after drag");
    };

    assert_vec2_close(
        moved_anchor.pos,
        first_anchor.pos,
        "anchor position stable when dragging handle",
    );
    let Some(moved_handle_out) = moved_anchor.handle_out else {
        panic!("expected handle_out to remain present after dragging");
    };
    assert_vec2_close(
        moved_handle_out,
        original_handle_out.add(scenario.delta),
        "handle_out moved by delta",
    );

    simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let restored_shape = require_draw_shape(&doc_after_undo, "after undo");
    let Some(restored_anchor) = restored_shape.path.anchors.first() else {
        panic!("expected first anchor after undo");
    };
    let Some(restored_handle_out) = restored_anchor.handle_out else {
        panic!("expected handle_out after undo");
    };

    assert_vec2_close(restored_anchor.pos, first_anchor.pos, "anchor restored");
    assert_vec2_close(
        restored_handle_out,
        original_handle_out,
        "handle_out restored",
    );
}
