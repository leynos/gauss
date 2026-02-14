//! GPUI headless integration tests for Phase 0 draw-mode interactions.

mod common;

use common::{
    canvas_points, ensure_initial_draw, init_test_app, read_document, read_history_len,
    require_canvas_click_changed, require_draw_shape, require_last_canvas_click,
    simulate_document_redo, simulate_document_undo,
};
use gauss::ui::Phase0Shell;
use gpui::TestAppContext;
use test_support::{TestSupportError, TestSupportResult};

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
) -> TestSupportResult<()> {
    if doc.len() != expected.total_shapes {
        return Err(TestSupportError::expectation(format!(
            "unexpected shape count: {context}"
        )));
    }

    let shape = require_draw_shape(doc, context)?;
    if shape.path.anchors.len() != expected.anchors {
        return Err(TestSupportError::expectation(format!(
            "unexpected anchor count: {context}"
        )));
    }
    if shape.path.segments.len() != expected.segments {
        return Err(TestSupportError::expectation(format!(
            "unexpected segment count: {context}"
        )));
    }
    if shape.path.closed != expected.closed {
        return Err(TestSupportError::expectation(format!(
            "unexpected closed state: {context}"
        )));
    }
    Ok(())
}

fn assert_draw_shape_absent(
    doc: &gauss::model::Document,
    expected_total_shapes: usize,
    context: &str,
) -> TestSupportResult<()> {
    if doc.len() != expected_total_shapes {
        return Err(TestSupportError::expectation(format!(
            "unexpected shape count: {context}"
        )));
    }
    if common::find_draw_shape(doc).is_some() {
        return Err(TestSupportError::expectation(format!(
            "draw shape should be absent: {context}"
        )));
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "test helper bundles click + verify for readability"
)]
fn click_and_verify_state(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    position: gpui::Point<gpui::Pixels>,
    expected: ExpectedDrawShapeState,
    context: &str,
) -> TestSupportResult<()> {
    common::click_canvas_and_wait(visual_cx, position);
    let doc = read_document(visual_cx, view);
    assert_draw_shape_state(&doc, expected, context)
}

#[gpui::test]
fn draw_click_adds_points_and_undo_removes(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let (pos1, pos2) = canvas_points(visual_cx).expect("canvas points should be available");

    let len_baseline = read_history_len(visual_cx, &view);

    click_and_verify_state(
        visual_cx,
        &view,
        pos1,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after first click",
    )
    .expect("draw shape should contain one anchor after first click");
    let last_click_after_first = require_last_canvas_click(visual_cx, &view, "after first click")
        .expect("expected Phase0Shell to record the first click");
    assert_eq!(
        read_history_len(visual_cx, &view),
        len_baseline + 1,
        "expected one undo entry after first draw click"
    );

    click_and_verify_state(
        visual_cx,
        &view,
        pos2,
        ExpectedDrawShapeState::new(2, 2, 1, false),
        "after second click",
    )
    .expect("draw shape should contain two anchors after second click");
    let _last_click_after_second = require_canvas_click_changed(
        visual_cx,
        &view,
        last_click_after_first,
        "after second click",
    )
    .expect("expected Phase0Shell to record a second click");
    assert_eq!(
        read_history_len(visual_cx, &view),
        len_baseline + 2,
        "expected two undo entries after second draw click"
    );

    simulate_document_undo(visual_cx);
    let doc_after_undo = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_undo,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after undoing second click",
    )
    .expect("draw shape should have one anchor after undo");

    simulate_document_undo(visual_cx);
    let doc_after_second_undo = read_document(visual_cx, &view);
    assert_draw_shape_absent(&doc_after_second_undo, 1, "after undoing the first click")
        .expect("draw shape should be removed after undo");

    // Redo should re-insert the shape with one anchor.
    simulate_document_redo(visual_cx);
    let doc_after_first_redo = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_first_redo,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after first redo",
    )
    .expect("draw shape should be re-inserted with one anchor after redo");

    // Second redo should restore the second anchor.
    simulate_document_redo(visual_cx);
    let doc_after_second_redo = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_second_redo,
        ExpectedDrawShapeState::new(2, 2, 1, false),
        "after second redo",
    )
    .expect("draw shape should have two anchors after second redo");
}
