//! GPUI headless integration tests for Phase 0 draw-mode interactions.

mod common;

use common::{
    canvas_bounds, canvas_points, ensure_initial_draw, init_test_app, read_document,
    read_history_len, require_canvas_click_changed, require_draw_shape, require_last_canvas_click,
    simulate_document_redo, simulate_document_undo,
};
use gauss::model::ShapeId;
use gauss::ui::{GpuiActivatePenTool, Phase0Shell};
use gpui::{Modifiers, TestAppContext, point, px};
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

/// Encapsulates a click-and-verify scenario for draw state testing.
#[derive(Clone, Copy)]
struct ClickVerification {
    position: gpui::Point<gpui::Pixels>,
    expected: ExpectedDrawShapeState,
    context: &'static str,
}

impl ClickVerification {
    const fn new(
        position: gpui::Point<gpui::Pixels>,
        expected: ExpectedDrawShapeState,
        context: &'static str,
    ) -> Self {
        Self {
            position,
            expected,
            context,
        }
    }
}

fn click_and_verify_state(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    verification: ClickVerification,
) -> TestSupportResult<()> {
    common::click_canvas_and_wait(visual_cx, verification.position);
    let doc = read_document(visual_cx, view);
    assert_draw_shape_state(&doc, verification.expected, verification.context)
}

/// Apply a document action (undo/redo), read the document, and verify the result.
fn apply_action_and_verify<F, V>(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    action: F,
    verify: V,
) -> TestSupportResult<()>
where
    F: FnOnce(&mut gpui::VisualTestContext),
    V: FnOnce(&gauss::model::Document) -> TestSupportResult<()>,
{
    action(visual_cx);
    let doc = read_document(visual_cx, view);
    verify(&doc)
}

fn undo_and_verify_state(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    expected: ExpectedDrawShapeState,
    context: &str,
) -> TestSupportResult<()> {
    apply_action_and_verify(visual_cx, view, simulate_document_undo, |doc| {
        assert_draw_shape_state(doc, expected, context)
    })
}

fn undo_and_verify_absent(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    expected_total_shapes: usize,
    context: &str,
) -> TestSupportResult<()> {
    apply_action_and_verify(visual_cx, view, simulate_document_undo, |doc| {
        assert_draw_shape_absent(doc, expected_total_shapes, context)
    })
}

fn redo_and_verify_state(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    expected: ExpectedDrawShapeState,
    context: &str,
) -> TestSupportResult<()> {
    apply_action_and_verify(visual_cx, view, simulate_document_redo, |doc| {
        assert_draw_shape_state(doc, expected, context)
    })
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
        ClickVerification::new(
            pos1,
            ExpectedDrawShapeState::new(2, 1, 0, false),
            "after first click",
        ),
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
        ClickVerification::new(
            pos2,
            ExpectedDrawShapeState::new(2, 2, 1, false),
            "after second click",
        ),
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

    undo_and_verify_state(
        visual_cx,
        &view,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after undoing second click",
    )
    .expect("draw shape should have one anchor after undo");

    undo_and_verify_absent(visual_cx, &view, 1, "after undoing the first click")
        .expect("draw shape should be removed after undo");

    // Redo should re-insert the shape with one anchor.
    redo_and_verify_state(
        visual_cx,
        &view,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after first redo",
    )
    .expect("draw shape should be re-inserted with one anchor after redo");

    // Second redo should restore the second anchor.
    redo_and_verify_state(
        visual_cx,
        &view,
        ExpectedDrawShapeState::new(2, 2, 1, false),
        "after second redo",
    )
    .expect("draw shape should have two anchors after second redo");
}

#[gpui::test]
fn activate_pen_tool_from_manipulate_allows_drawing(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let click_in_manipulate = point(bounds.origin.x + px(8.0), bounds.origin.y + px(8.0));
    let click_in_draw = point(bounds.origin.x + px(24.0), bounds.origin.y + px(24.0));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let shape_count_before = read_document(visual_cx, &view).len();

    visual_cx.simulate_mouse_move(click_in_manipulate, None, Modifiers::none());
    visual_cx.simulate_click(click_in_manipulate, Modifiers::none());
    visual_cx.run_until_parked();

    let shape_count_after_manipulate_click = read_document(visual_cx, &view).len();
    assert_eq!(
        shape_count_after_manipulate_click, shape_count_before,
        "expected manipulate-mode click to keep shape count unchanged"
    );

    visual_cx.dispatch_action(GpuiActivatePenTool);
    visual_cx.run_until_parked();

    visual_cx.simulate_mouse_move(click_in_draw, None, Modifiers::none());
    visual_cx.simulate_click(click_in_draw, Modifiers::none());
    visual_cx.run_until_parked();

    let shape_count_after_draw_click = read_document(visual_cx, &view).len();
    assert_eq!(
        shape_count_after_draw_click,
        shape_count_before.saturating_add(1),
        "expected pen tool activation to restore draw-click shape insertion"
    );
}

#[gpui::test]
fn stale_active_path_is_recovered_when_pen_draws_new_shape(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let click_point = point(bounds.origin.x + px(12.0), bounds.origin.y + px(12.0));
    let stale_path = ShapeId::from_accesskit_node_id(9_999);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.set_draw_active_shape_for_tests(Some(stale_path));
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let active_before_click = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
    assert_eq!(
        active_before_click,
        Some(stale_path),
        "expected test setup to install a stale active path"
    );

    common::click_canvas_and_wait(visual_cx, click_point);

    let doc_after_click = read_document(visual_cx, &view);
    let shape = require_draw_shape(&doc_after_click, "after stale active path click")
        .expect("expected draw shape after recovering from stale active path");
    assert_eq!(
        shape.path.anchors.len(),
        1,
        "expected stale active path recovery to start a new single-anchor shape"
    );
    assert_eq!(
        shape.path.segments.len(),
        0,
        "expected no segments after first anchor in recovered draw flow"
    );

    let active_after_click = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
    assert_eq!(
        active_after_click,
        Some(shape.id),
        "expected active path to track the newly created shape after recovery"
    );
    assert_ne!(
        active_after_click,
        Some(stale_path),
        "expected stale active path id to be replaced during recovery"
    );

    let history_error = visual_cx.read(|app| {
        view.read(app)
            .last_history_error_for_tests()
            .map(str::to_owned)
    });
    assert!(
        history_error.is_none(),
        "expected stale active path recovery to avoid surfacing history errors"
    );
}
