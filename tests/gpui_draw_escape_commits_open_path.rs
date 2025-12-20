//! GPUI headless integration tests for Draw-mode `Escape` behaviour.
//!
//! In Phase 0, pressing `Escape` while drawing should:
//! - keep the current open path in the document, and
//! - switch to manipulate mode (so clicks no longer place points).

mod common;

use common::{
    canvas_bounds, click_canvas_and_wait, ensure_initial_draw, init_test_app, read_document,
    require_draw_shape, simulate_escape,
};
use gauss::model::Vec2;
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, point, px};

#[gpui::test]
fn escape_commits_open_path_and_enters_manipulate(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");

    let p1 = point(bounds.origin.x + px(10.0), bounds.origin.y + px(10.0));
    let p2 = point(bounds.origin.x + px(80.0), bounds.origin.y + px(30.0));

    click_canvas_and_wait(visual_cx, p1);
    click_canvas_and_wait(visual_cx, p2);

    let doc_before_escape = read_document(visual_cx, &view);
    let shape_before_escape = require_draw_shape(&doc_before_escape, "after drawing two points")
        .expect("expected draw shape after drawing two points");
    assert!(
        !shape_before_escape.path.closed,
        "expected newly drawn path to be open before closing; shape={shape_before_escape:?}"
    );

    let anchor_count_before = shape_before_escape.path.anchors.len();
    let seg_count_before = shape_before_escape.path.segments.len();

    simulate_escape(visual_cx);

    // Clicking after escape should not add points (we should be in manipulate
    // mode), and the open path should still exist in the document.
    click_canvas_and_wait(visual_cx, p2);

    let doc_after = read_document(visual_cx, &view);
    let shape_after = require_draw_shape(&doc_after, "after escape and click")
        .expect("expected draw shape after escape and click");

    assert_eq!(
        shape_after.id, shape_before_escape.id,
        "expected escape to keep the same open path in the document"
    );
    assert!(
        !shape_after.path.closed,
        "expected escape to commit an open path without closing it"
    );
    assert_eq!(
        shape_after.path.anchors.len(),
        anchor_count_before,
        "expected manipulate-mode click after escape to not add anchors"
    );
    assert_eq!(
        shape_after.path.segments.len(),
        seg_count_before,
        "expected manipulate-mode click after escape to not add segments"
    );

    // Keep this test resilient to coordinate system changes by asserting that
    // the first two anchors are still distinct.
    let first_anchor = shape_after
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |anchor| anchor.pos);
    let second_anchor = shape_after
        .path
        .anchors
        .get(1)
        .map_or(Vec2::ZERO, |anchor| anchor.pos);
    assert_ne!(
        first_anchor, second_anchor,
        "expected at least two distinct anchors in the committed open path"
    );
}
