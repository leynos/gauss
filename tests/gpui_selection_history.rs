//! GPUI headless integration tests for Phase 0 selection history.
#![expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]

mod common;

use common::{
    anchor_to_canvas_point, click_left_and_wait, draw_point, ensure_initial_draw, init_test_app,
    require_draw_shape, simulate_escape,
};
use gauss::model::Vec2;
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, point, px};

#[gpui::test]
fn selection_undo_uses_shift_modified_stack(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = common::canvas_bounds(visual_cx).expect("canvas bounds should be available");

    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));

    let dx = (width - 4.0).clamp(1.0, 40.0);
    let dy = (height - 4.0).clamp(1.0, 24.0);
    let p2 = point(p1.x + px(dx), p1.y + px(dy));

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);

    let doc_before = visual_cx.read(|app| view.read(app).document().clone());
    let draw_shape = require_draw_shape(&doc_before, "after drawing")
        .expect("expected draw shape after drawing");
    let shapes_len_before = doc_before.shapes.len();

    simulate_escape(visual_cx);

    let anchor0 = draw_shape
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |anchor| anchor.pos);
    let select_point = anchor_to_canvas_point(&bounds, anchor0, p1);

    click_left_and_wait(visual_cx, select_point);

    let selection_after_select = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        !selection_after_select.items.is_empty(),
        "expected selection to be non-empty; got selection={selection_after_select:?}"
    );
    let selected_snapshot = selection_after_select.clone();

    let clear_x = (width - 12.0).max(1.0);
    let clear_y = (height - 12.0).max(1.0);
    let clear_point = point(bounds.origin.x + px(clear_x), bounds.origin.y + px(clear_y));
    click_left_and_wait(visual_cx, clear_point);

    let selection_after_clear = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_clear.items.is_empty(),
        "expected selection to be cleared; got selection={selection_after_clear:?}"
    );

    common::simulate_selection_undo(visual_cx);
    let selection_after_undo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_undo, selected_snapshot,
        "expected Shift+Undo to restore the selection"
    );

    common::simulate_selection_redo(visual_cx);
    let selection_after_redo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_redo, selection_after_clear,
        "expected Shift+Redo to reapply the selection clear"
    );

    let doc_after = visual_cx.read(|app| view.read(app).document().clone());
    assert_eq!(
        doc_after.shapes.len(),
        shapes_len_before,
        "selection undo/redo should not change the document"
    );
}
