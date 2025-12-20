//! GPUI headless integration tests for Phase 0 anchor insertion/deletion.

mod common;

use common::{
    anchor_to_canvas_point, assert_vec2_close, canvas_bounds, click_canvas_and_wait,
    ensure_initial_draw, init_test_app, read_document, require_draw_shape, simulate_escape,
    simulate_key,
};
use gauss::model::{SelItem, Shape, ShapeId, Vec2};
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

fn require_anchor_pos(shape: &Shape, index: usize, context: &str) -> Vec2 {
    shape.path.anchors.get(index).map_or_else(
        || panic!("missing anchor {index}: {context}"),
        |anchor| anchor.pos,
    )
}

fn require_anchor_len(shape: &Shape, expected: usize, context: &str) {
    assert_eq!(
        shape.path.anchors.len(),
        expected,
        "{context}: expected {expected} anchors, got {}",
        shape.path.anchors.len()
    );
}

fn require_segment_len(shape: &Shape, expected: usize, context: &str) {
    assert_eq!(
        shape.path.segments.len(),
        expected,
        "{context}: expected {expected} segments, got {}",
        shape.path.segments.len()
    );
}

fn require_path_counts(shape: &Shape, anchors: usize, segments: usize, context: &str) {
    require_anchor_len(shape, anchors, context);
    require_segment_len(shape, segments, context);
}

fn read_draw_shape(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    context: &str,
) -> Shape {
    let doc = read_document(visual_cx, view);
    require_draw_shape(&doc, context).clone()
}

#[gpui::test]
fn insert_and_delete_anchor_are_doc_undoable(cx: &mut TestAppContext) {
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

    let shape_before = read_draw_shape(visual_cx, &view, "after drawing");
    require_path_counts(&shape_before, 2, 1, "after drawing");

    let start_pos = require_anchor_pos(&shape_before, 0, "after drawing");
    let end_pos = require_anchor_pos(&shape_before, 1, "after drawing");

    simulate_escape(visual_cx);

    let midpoint = Vec2::new(
        f32::midpoint(start_pos.x, end_pos.x),
        f32::midpoint(start_pos.y, end_pos.y),
    );
    let select_point = anchor_to_canvas_point(&bounds, midpoint, p1);
    select_segment0(visual_cx, &view, select_point, shape_before.id);

    simulate_key(visual_cx, "i", Modifiers::none());
    let shape_after_insert = read_draw_shape(visual_cx, &view, "after insert");
    require_path_counts(&shape_after_insert, 3, 2, "after insert");

    let inserted_pos = require_anchor_pos(&shape_after_insert, 1, "after insert");
    assert_vec2_close(
        inserted_pos,
        midpoint,
        "inserted anchor should be at midpoint",
    );

    let selection_after_insert = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_insert.contains(&SelItem::Anchor {
            shape: shape_before.id,
            anchor: 1
        }),
        "expected inserted anchor to become selected; selection={selection_after_insert:?}"
    );

    simulate_key(visual_cx, "backspace", Modifiers::none());
    let shape_after_delete = read_draw_shape(visual_cx, &view, "after delete");
    require_path_counts(&shape_after_delete, 2, 1, "after delete");

    let selection_after_delete = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_delete.items.is_empty(),
        "expected delete to clear selection; selection={selection_after_delete:?}"
    );

    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    let shape_after_undo_delete = read_draw_shape(visual_cx, &view, "after undo delete");
    require_path_counts(&shape_after_undo_delete, 3, 2, "after undo delete");

    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    let shape_after_undo_insert = read_draw_shape(visual_cx, &view, "after undo insert");
    require_path_counts(&shape_after_undo_insert, 2, 1, "after undo insert");

    simulate_key(visual_cx, "y", Modifiers::secondary_key());
    let shape_after_redo_insert = read_draw_shape(visual_cx, &view, "after redo insert");
    require_path_counts(&shape_after_redo_insert, 3, 2, "after redo insert");

    simulate_key(visual_cx, "y", Modifiers::secondary_key());
    let shape_after_redo_delete = read_draw_shape(visual_cx, &view, "after redo delete");
    require_path_counts(&shape_after_redo_delete, 2, 1, "after redo delete");
}
