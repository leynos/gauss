//! GPUI headless integration tests for Phase 0 handle dragging.


use crate::common::{
    CanvasDragScenario, anchor_to_canvas_point, assert_vec2_close, canvas_drag_scenario,
    ensure_initial_draw, init_test_app, read_document, read_history_len, require_draw_shape,
    simulate_document_undo, simulate_escape,
};
use gauss::model::{SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult};

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

#[derive(Debug)]
struct HandleDragSetup {
    scenario: CanvasDragScenario,
    shape_id: ShapeId,
    first_anchor_pos: Vec2,
    original_handle_out: Vec2,
    handle_start: gpui::Point<gpui::Pixels>,
    handle_end: gpui::Point<gpui::Pixels>,
}

fn setup_handle_drag(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> TestSupportResult<HandleDragSetup> {
    let scenario = canvas_drag_scenario(visual_cx, 18.0, 10.0)?;
    draw_two_point_bezier_path(visual_cx, scenario);

    let doc_before = read_document(visual_cx, view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points")?;
    let first_anchor = original_shape
        .path
        .anchors
        .first()
        .cloned()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "after drawing"))?;
    let original_handle_out = first_anchor.handle_out.ok_or_else(|| {
        TestSupportError::missing("handle_out", "after drawing bezier auto shape")
    })?;

    let handle_start =
        anchor_to_canvas_point(&scenario.bounds, original_handle_out, scenario.first);
    let handle_end = point(
        handle_start.x + px(scenario.delta.x),
        handle_start.y + px(scenario.delta.y),
    );

    simulate_escape(visual_cx);

    Ok(HandleDragSetup {
        scenario,
        shape_id: original_shape.id,
        first_anchor_pos: first_anchor.pos,
        original_handle_out,
        handle_start,
        handle_end,
    })
}

fn assert_handle_selection_includes_shape(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    if !selection.contains(&SelItem::Shape(shape_id)) {
        return Err(TestSupportError::expectation(format!(
            "expected handle interaction to keep the shape selected; selection={selection:?}"
        )));
    }
    if !selection.contains(&SelItem::HandleOut {
        shape: shape_id,
        anchor: 0,
    }) {
        return Err(TestSupportError::expectation(format!(
            "expected mouse down to select the handle; selection={selection:?}"
        )));
    }
    Ok(())
}

#[gpui::test]
fn dragging_handle_moves_it_and_undo_restores(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let setup = setup_handle_drag(visual_cx, &view).expect("expected handle drag setup");

    let len_before = read_history_len(visual_cx, &view);

    visual_cx.simulate_mouse_down(setup.handle_start, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    assert_handle_selection_includes_shape(visual_cx, &view, setup.shape_id)
        .expect("expected handle selection to include shape");

    visual_cx.simulate_mouse_move(setup.handle_end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(setup.handle_end, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let doc_after_drag = read_document(visual_cx, &view);
    let moved_shape = require_draw_shape(&doc_after_drag, "after dragging handle")
        .expect("expected draw shape after handle drag");
    let moved_anchor = moved_shape
        .path
        .anchors
        .first()
        .expect("expected first anchor after drag");

    assert_vec2_close(
        moved_anchor.pos,
        setup.first_anchor_pos,
        "anchor position stable when dragging handle",
    )
    .expect("expected anchor position to remain stable");
    let moved_handle_out = moved_anchor
        .handle_out
        .expect("expected handle_out to remain present after dragging");
    assert_vec2_close(
        moved_handle_out,
        setup.original_handle_out.add(setup.scenario.delta),
        "handle_out moved by delta",
    )
    .expect("expected handle_out to move by drag delta");

    let len_after = read_history_len(visual_cx, &view);
    assert_eq!(
        len_after,
        len_before + 1,
        "expected exactly one undo entry for handle drag"
    );

    simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let restored_shape =
        require_draw_shape(&doc_after_undo, "after undo").expect("expected draw shape after undo");
    let restored_anchor = restored_shape
        .path
        .anchors
        .first()
        .expect("expected first anchor after undo");
    let restored_handle_out = restored_anchor
        .handle_out
        .expect("expected handle_out after undo");

    assert_vec2_close(
        restored_anchor.pos,
        setup.first_anchor_pos,
        "anchor restored",
    )
    .expect("expected anchor to be restored");
    assert_vec2_close(
        restored_handle_out,
        setup.original_handle_out,
        "handle_out restored",
    )
    .expect("expected handle_out to be restored");
}
