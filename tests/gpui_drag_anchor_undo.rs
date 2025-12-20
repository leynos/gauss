//! GPUI headless integration tests for Phase 0 anchor dragging.

mod common;

use common::{
    CanvasDragScenario, assert_vec2_close, canvas_drag_scenario, draw_point, ensure_initial_draw,
    init_test_app, read_document, require_draw_shape, simulate_document_undo, simulate_escape,
};
use gauss::model::{Anchor, SelItem, Shape, ShapeId};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

fn draw_two_point_line_path(visual_cx: &mut VisualTestContext, scenario: CanvasDragScenario) {
    draw_point(visual_cx, scenario.first);
    draw_point(visual_cx, scenario.second);
}

fn first_two_anchors(shape: &Shape) -> TestSupportResult<(Anchor, Anchor)> {
    let first = shape
        .path
        .anchors
        .first()
        .cloned()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "first two anchors"))?;
    let second = shape
        .path
        .anchors
        .get(1)
        .cloned()
        .ok_or_else(|| TestSupportError::missing("anchor 1", "first two anchors"))?;
    Ok((first, second))
}

fn drag_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    shape_id: ShapeId,
    scenario: CanvasDragScenario,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(scenario.first, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selected_anchor = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    if !selection.contains(&SelItem::Shape(shape_id)) {
        return Err(TestSupportError::expectation(format!(
            "expected anchor interaction to keep the shape selected; selection={selection:?}"
        )));
    }
    if !selection.contains(&selected_anchor) {
        return Err(TestSupportError::expectation(format!(
            "expected mouse down to select the first anchor; selection={selection:?}"
        )));
    }

    visual_cx.simulate_mouse_move(scenario.drag_end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(scenario.drag_end, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

#[gpui::test]
fn dragging_anchor_moves_it_and_undo_restores(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let scenario =
        canvas_drag_scenario(visual_cx, 24.0, 12.0).expect("expected canvas drag scenario");
    draw_two_point_line_path(visual_cx, scenario);

    let doc_before = read_document(visual_cx, &view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points")
        .expect("expected draw shape after drawing")
        .clone();
    let (original_first_anchor, original_second_anchor) =
        first_two_anchors(&original_shape).expect("expected two anchors in drawn shape");

    simulate_escape(visual_cx);
    drag_first_anchor(visual_cx, &view, original_shape.id, scenario)
        .expect("expected drag on first anchor");

    let doc_after_drag = read_document(visual_cx, &view);
    let moved_shape = require_draw_shape(&doc_after_drag, "after dragging anchor")
        .expect("expected draw shape after dragging anchor");
    let (moved_first_anchor, moved_second_anchor) =
        first_two_anchors(moved_shape).expect("expected two anchors after dragging");

    assert_vec2_close(
        moved_first_anchor.pos,
        original_first_anchor.pos.add(scenario.delta),
        "first anchor moved",
    )
    .expect("expected first anchor to move by drag delta");
    assert_vec2_close(
        moved_second_anchor.pos,
        original_second_anchor.pos,
        "second anchor stable",
    )
    .expect("expected second anchor to remain stable");

    if let Some(handle_out) = original_first_anchor.handle_out {
        let moved_handle_out = moved_first_anchor
            .handle_out
            .expect("expected moved handle_out to remain present");
        assert_vec2_close(
            moved_handle_out,
            handle_out.add(scenario.delta),
            "first anchor handle_out moved",
        )
        .expect("expected handle_out to move with anchor");
    }

    simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let restored_shape =
        require_draw_shape(&doc_after_undo, "after undo").expect("expected draw shape after undo");
    let (restored_first_anchor, restored_second_anchor) =
        first_two_anchors(restored_shape).expect("expected two anchors after undo");

    assert_vec2_close(
        restored_first_anchor.pos,
        original_first_anchor.pos,
        "first anchor restored",
    )
    .expect("expected first anchor to restore");
    assert_vec2_close(
        restored_second_anchor.pos,
        original_second_anchor.pos,
        "second anchor still stable after undo",
    )
    .expect("expected second anchor to remain stable after undo");
}
