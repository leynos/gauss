//! GPUI headless integration tests for Phase 0 segment toggling.
//!
//! Segment toggling converts a selected segment between a straight line and a
//! cubic Bézier curve. When a line becomes cubic, Phase 0 synthesises initial
//! handles so the segment can be manipulated immediately. The inverse toggle
//! clears the handles and restores a straight line.
//!
//! This test also covers undo: once a segment is toggled, `Undo` should restore
//! both the segment kind and the handle positions.

#![expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]
mod common;

use common::{
    anchor_to_canvas_point, assert_vec2_close, canvas_bounds, click_canvas_and_wait,
    ensure_initial_draw, init_test_app, read_document, read_history_len, require_draw_shape,
    simulate_escape,
};
use gauss::model::{SegmentKind, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult, math};

fn select_segment0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    select_point: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    if !selection.contains(&SelItem::Shape(shape_id)) {
        return Err(TestSupportError::expectation(format!(
            "expected shape to remain selected when selecting a segment; selection={selection:?}"
        )));
    }
    let expected_segment = SelItem::Segment {
        shape: shape_id,
        seg: 0,
    };
    if !selection.contains(&expected_segment) {
        return Err(TestSupportError::expectation(format!(
            "expected segment selection; selection={selection:?}"
        )));
    }

    visual_cx.simulate_mouse_up(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

fn assert_segment0_is_cubic_with_initial_handles(
    shape: &Shape,
    start_pos: Vec2,
    end_pos: Vec2,
) -> TestSupportResult<()> {
    let kind = shape
        .path
        .segments
        .first()
        .copied()
        .ok_or_else(|| TestSupportError::missing("segment", "after toggle"))?;
    if kind != SegmentKind::Cubic {
        return Err(TestSupportError::expectation(
            "expected segment to become cubic",
        ));
    }

    let start_anchor = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "after toggle"))?;
    let end_anchor = shape
        .path
        .anchors
        .get(1)
        .ok_or_else(|| TestSupportError::missing("anchor 1", "after toggle"))?;

    let handle_out = start_anchor
        .handle_out
        .ok_or_else(|| TestSupportError::missing("handle_out", "after line->cubic toggle"))?;
    let handle_in = end_anchor
        .handle_in
        .ok_or_else(|| TestSupportError::missing("handle_in", "after line->cubic toggle"))?;

    let delta = end_pos.sub(start_pos);
    let third = delta.mul(1.0 / 3.0);
    assert_vec2_close(
        handle_out,
        start_pos.add(third),
        "handle_out should be one third along the line",
    )?;
    assert_vec2_close(
        handle_in,
        end_pos.sub(third),
        "handle_in should be two thirds along the line",
    )?;
    Ok(())
}

fn assert_segment0_restores_line_without_handles(shape: &Shape) -> TestSupportResult<()> {
    let kind = shape
        .path
        .segments
        .first()
        .copied()
        .ok_or_else(|| TestSupportError::missing("segment", "after undo"))?;
    if kind != SegmentKind::Line {
        return Err(TestSupportError::expectation(
            "expected undo to restore line",
        ));
    }

    let start_anchor = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "after undo"))?;
    let end_anchor = shape
        .path
        .anchors
        .get(1)
        .ok_or_else(|| TestSupportError::missing("anchor 1", "after undo"))?;

    if start_anchor.handle_out.is_some() {
        return Err(TestSupportError::expectation(
            "expected handle_out cleared on undo",
        ));
    }
    if end_anchor.handle_in.is_some() {
        return Err(TestSupportError::expectation(
            "expected handle_in cleared on undo",
        ));
    }
    Ok(())
}

#[gpui::test]
fn tab_toggles_selected_segment_kind_and_undo_restores(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let p1 = point(
        bounds.origin.x + px(common::CANVAS_PADDING_PX),
        bounds.origin.y + px(common::CANVAS_PADDING_PX),
    );
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(common::CANVAS_PADDING_PX),
        bounds.origin.y + bounds.size.height - px(common::CANVAS_PADDING_PX),
    );

    click_canvas_and_wait(visual_cx, p1);
    click_canvas_and_wait(visual_cx, p2);

    let doc_before = read_document(visual_cx, &view);
    let shape_before = require_draw_shape(&doc_before, "after drawing")
        .expect("expected draw shape after drawing")
        .clone();
    let start_anchor = shape_before
        .path
        .anchors
        .first()
        .expect("expected first anchor after drawing");
    let end_anchor = shape_before
        .path
        .anchors
        .get(1)
        .expect("expected second anchor after drawing");

    simulate_escape(visual_cx);

    let midpoint = Vec2::new(
        math::midpoint(start_anchor.pos.x, end_anchor.pos.x),
        math::midpoint(start_anchor.pos.y, end_anchor.pos.y),
    );
    let select_point = anchor_to_canvas_point(&bounds, midpoint, p1);

    select_segment0(visual_cx, &view, select_point, shape_before.id)
        .expect("expected segment selection to succeed");

    let len_before_toggle = read_history_len(visual_cx, &view);

    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();

    let doc_after_toggle = read_document(visual_cx, &view);
    let shape_after_toggle = require_draw_shape(&doc_after_toggle, "after toggle")
        .expect("expected draw shape after toggle");
    assert_segment0_is_cubic_with_initial_handles(
        shape_after_toggle,
        start_anchor.pos,
        end_anchor.pos,
    )
    .expect("expected cubic handles after toggle");
    assert_eq!(
        read_history_len(visual_cx, &view),
        len_before_toggle + 1,
        "expected one undo entry for segment toggle"
    );

    common::simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let shape_after_undo =
        require_draw_shape(&doc_after_undo, "after undo").expect("expected draw shape after undo");
    assert_segment0_restores_line_without_handles(shape_after_undo)
        .expect("expected line without handles after undo");
}
