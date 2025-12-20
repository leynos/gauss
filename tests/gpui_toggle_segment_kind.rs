//! GPUI headless integration tests for Phase 0 segment toggling.
//!
//! Segment toggling converts a selected segment between a straight line and a
//! cubic Bézier curve. When a line becomes cubic, Phase 0 synthesises initial
//! handles so the segment can be manipulated immediately. The inverse toggle
//! clears the handles and restores a straight line.
//!
//! This test also covers undo: once a segment is toggled, `Undo` should restore
//! both the segment kind and the handle positions.

mod common;

use common::{
    anchor_to_canvas_point, assert_vec2_close, canvas_bounds, click_canvas_and_wait,
    ensure_initial_draw, init_test_app, read_document, require_draw_shape, simulate_key,
};
use gauss::model::{SegmentKind, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};

fn select_segment0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    select_point: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) {
    visual_cx.simulate_mouse_down(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection.contains(&SelItem::Shape(shape_id)),
        "expected shape to remain selected when selecting a segment; selection={selection:?}"
    );
    let expected_segment = SelItem::Segment {
        shape: shape_id,
        seg: 0,
    };
    assert!(
        selection.contains(&expected_segment),
        "expected segment selection; selection={selection:?}"
    );

    visual_cx.simulate_mouse_up(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
}

fn assert_segment0_is_cubic_with_initial_handles(shape: &Shape, start_pos: Vec2, end_pos: Vec2) {
    let Some(kind) = shape.path.segments.first().copied() else {
        panic!("expected segment after toggle");
    };
    assert_eq!(kind, SegmentKind::Cubic, "expected segment to become cubic");

    let Some(start_anchor) = shape.path.anchors.first() else {
        panic!("expected first anchor after toggle");
    };
    let Some(end_anchor) = shape.path.anchors.get(1) else {
        panic!("expected second anchor after toggle");
    };

    let Some(handle_out) = start_anchor.handle_out else {
        panic!("expected handle_out to be set after line->cubic toggle");
    };
    let Some(handle_in) = end_anchor.handle_in else {
        panic!("expected handle_in to be set after line->cubic toggle");
    };

    let delta = end_pos.sub(start_pos);
    let third = delta.mul(1.0 / 3.0);
    assert_vec2_close(
        handle_out,
        start_pos.add(third),
        "handle_out should be one third along the line",
    );
    assert_vec2_close(
        handle_in,
        end_pos.sub(third),
        "handle_in should be two thirds along the line",
    );
}

fn assert_segment0_restores_line_without_handles(shape: &Shape) {
    let Some(kind) = shape.path.segments.first().copied() else {
        panic!("expected segment after undo");
    };
    assert_eq!(kind, SegmentKind::Line, "expected undo to restore line");

    let Some(start_anchor) = shape.path.anchors.first() else {
        panic!("expected first anchor after undo");
    };
    let Some(end_anchor) = shape.path.anchors.get(1) else {
        panic!("expected second anchor after undo");
    };

    assert!(
        start_anchor.handle_out.is_none(),
        "expected handle_out cleared on undo"
    );
    assert!(
        end_anchor.handle_in.is_none(),
        "expected handle_in cleared on undo"
    );
}

#[gpui::test]
fn tab_toggles_selected_segment_kind_and_undo_restores(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx);
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
    let shape_before = require_draw_shape(&doc_before, "after drawing").clone();
    let Some(start_anchor) = shape_before.path.anchors.first() else {
        panic!("expected first anchor after drawing");
    };
    let Some(end_anchor) = shape_before.path.anchors.get(1) else {
        panic!("expected second anchor after drawing");
    };

    simulate_key(visual_cx, "escape", Modifiers::none());

    let midpoint = Vec2::new(
        f32::midpoint(start_anchor.pos.x, end_anchor.pos.x),
        f32::midpoint(start_anchor.pos.y, end_anchor.pos.y),
    );
    let select_point = anchor_to_canvas_point(&bounds, midpoint, p1);

    select_segment0(visual_cx, &view, select_point, shape_before.id);

    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();

    let doc_after_toggle = read_document(visual_cx, &view);
    let shape_after_toggle = require_draw_shape(&doc_after_toggle, "after toggle");
    assert_segment0_is_cubic_with_initial_handles(
        shape_after_toggle,
        start_anchor.pos,
        end_anchor.pos,
    );

    simulate_key(visual_cx, "z", Modifiers::secondary_key());

    let doc_after_undo = read_document(visual_cx, &view);
    let shape_after_undo = require_draw_shape(&doc_after_undo, "after undo");
    assert_segment0_restores_line_without_handles(shape_after_undo);
}
