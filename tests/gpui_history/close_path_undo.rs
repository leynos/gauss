//! GPUI headless integration test verifying that closing a path produces
//! exactly one undo entry and that undo reopens it.


use crate::common::{
    anchor_to_canvas_point, canvas_bounds, draw_point, ensure_initial_draw, init_test_app,
    read_document, read_history_len, require_draw_shape, simulate_document_undo,
};
use gauss::ui::Phase0Shell;
use gpui::{Bounds, Pixels, TestAppContext, point, px};

fn triangle_points(
    bounds: &Bounds<Pixels>,
) -> (
    gpui::Point<Pixels>,
    gpui::Point<Pixels>,
    gpui::Point<Pixels>,
) {
    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + px(12.0),
    );
    let p3 = point(
        bounds.origin.x + bounds.size.width - px(12.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );
    (p1, p2, p3)
}

#[gpui::test]
fn close_path_creates_one_undo_entry_and_undo_reopens(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let (p1, p2, p3) = triangle_points(&bounds);

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);
    draw_point(visual_cx, p3);

    let doc_before_close = read_document(visual_cx, &view);
    let shape_before = require_draw_shape(&doc_before_close, "before close")
        .expect("expected draw shape before close");
    assert!(
        !shape_before.path.closed,
        "expected path to be open before close"
    );
    assert_eq!(
        shape_before.path.anchors.len(),
        3,
        "expected three anchors before close"
    );

    let len_before_close = read_history_len(visual_cx, &view);

    // Close the path by clicking near the first anchor.
    let first_anchor_pos = shape_before.path.anchors.first().expect("anchor 0").pos;
    let close_point = anchor_to_canvas_point(&bounds, first_anchor_pos, p1);
    draw_point(visual_cx, close_point);

    let doc_after_close = read_document(visual_cx, &view);
    let shape_after = require_draw_shape(&doc_after_close, "after close")
        .expect("expected draw shape after close");
    assert!(shape_after.path.closed, "expected path to be closed");

    let len_after_close = read_history_len(visual_cx, &view);
    assert_eq!(
        len_after_close,
        len_before_close + 1,
        "expected exactly one undo entry for close path"
    );

    // Undo should reopen the path.
    simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let shape_after_undo =
        require_draw_shape(&doc_after_undo, "after undo").expect("expected draw shape after undo");
    assert!(
        !shape_after_undo.path.closed,
        "expected undo to reopen the path"
    );
    assert_eq!(
        shape_after_undo.path.anchors.len(),
        3,
        "expected three anchors after undo"
    );
}
