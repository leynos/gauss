//! GPUI headless integration tests for Phase 0 draw-mode interactions.

mod common;

use common::{
    canvas_points, ensure_initial_draw, init_test_app, read_document, require_canvas_click_changed,
    require_draw_shape, require_last_canvas_click, simulate_document_undo,
};
use gauss::ui::Phase0Shell;
use gpui::TestAppContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExpectedDrawShapeState {
    total_shapes: usize,
    anchors: usize,
    segments: usize,
    closed: bool,
}

impl ExpectedDrawShapeState {
    const fn new(total_shapes: usize, anchors: usize, segments: usize, closed: bool) -> Self {
        Self {
            total_shapes,
            anchors,
            segments,
            closed,
        }
    }
}

fn assert_draw_shape_state(
    doc: &gauss::model::Document,
    expected: ExpectedDrawShapeState,
    context: &str,
) {
    assert_eq!(
        doc.shapes.len(),
        expected.total_shapes,
        "unexpected shape count: {context}"
    );

    let shape = require_draw_shape(doc, context);
    assert_eq!(
        shape.path.anchors.len(),
        expected.anchors,
        "unexpected anchor count: {context}"
    );
    assert_eq!(
        shape.path.segments.len(),
        expected.segments,
        "unexpected segment count: {context}"
    );
    assert_eq!(
        shape.path.closed, expected.closed,
        "unexpected closed state: {context}"
    );
}

fn assert_draw_shape_absent(
    doc: &gauss::model::Document,
    expected_total_shapes: usize,
    context: &str,
) {
    assert_eq!(
        doc.shapes.len(),
        expected_total_shapes,
        "unexpected shape count: {context}"
    );
    assert!(
        common::find_draw_shape(doc).is_none(),
        "draw shape should be absent: {context}"
    );
}

#[gpui::test]
fn draw_click_adds_points_and_undo_removes(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let (pos1, pos2) = canvas_points(visual_cx);

    common::click_canvas_and_wait(visual_cx, pos1);
    let last_click_after_first = require_last_canvas_click(visual_cx, &view, "after first click");

    let doc_after_first = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_first,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after first click",
    );

    common::click_canvas_and_wait(visual_cx, pos2);
    let _last_click_after_second = require_canvas_click_changed(
        visual_cx,
        &view,
        last_click_after_first,
        "after second click",
    );

    let doc_after_second = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_second,
        ExpectedDrawShapeState::new(2, 2, 1, false),
        "after second click",
    );

    simulate_document_undo(visual_cx);
    let doc_after_undo = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_undo,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after undoing second click",
    );

    simulate_document_undo(visual_cx);
    let doc_after_second_undo = read_document(visual_cx, &view);
    assert_draw_shape_absent(&doc_after_second_undo, 1, "after undoing the first click");
}
