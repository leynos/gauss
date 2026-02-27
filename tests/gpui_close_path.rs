//! GPUI headless integration tests for Phase 0 draw-mode path closing.
//!
//! Closing a path is a key workflow boundary in Phase 0:
//!
//! - The user clicks near the first anchor (within a snap radius).
//! - The active path becomes closed (and gains a default fill if none was set).
//! - The editor switches to manipulate mode so subsequent clicks do not place
//!   more points.

mod common;

use common::{
    anchor_to_canvas_point, canvas_bounds, draw_point, ensure_initial_draw, init_test_app,
    require_draw_shape,
};
use gauss::model::{Document, SegmentKind, ShapeId};
use gauss::ui::Phase0Shell;
use gpui::{Bounds, Pixels, TestAppContext, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult};

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

fn assert_open_triangle(doc: &Document) -> TestSupportResult<ShapeId> {
    if doc.len() != 2 {
        return Err(TestSupportError::expectation(
            "expected demo + one draw shape before close",
        ));
    }

    let shape = require_draw_shape(doc, "before close")?;
    if shape.path.closed {
        return Err(TestSupportError::expectation(
            "expected path to be open before close",
        ));
    }
    if shape.path.anchors.len() != 3 {
        return Err(TestSupportError::expectation(
            "expected three anchors before close",
        ));
    }
    Ok(shape.id)
}

fn close_path_by_clicking_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    bounds: &Bounds<Pixels>,
    click_point: gpui::Point<Pixels>,
) -> TestSupportResult<ShapeId> {
    let doc = visual_cx.read(|app| view.read(app).document().clone());
    let shape = require_draw_shape(&doc, "before close")?;
    let shape_id = shape.id;

    let first_anchor = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "before close"))?;

    let close_point = anchor_to_canvas_point(bounds, first_anchor.pos, click_point);
    draw_point(visual_cx, close_point);

    Ok(shape_id)
}

fn assert_closed_shape(doc: &Document, expected_shape_id: ShapeId) -> TestSupportResult<()> {
    if doc.len() != 2 {
        return Err(TestSupportError::expectation(
            "closing the path should not create a new shape",
        ));
    }

    let shape = require_draw_shape(doc, "after close")?;
    if shape.id != expected_shape_id {
        return Err(TestSupportError::expectation(
            "expected close to preserve the shape id",
        ));
    }
    if !shape.path.closed {
        return Err(TestSupportError::expectation(
            "expected path to become closed",
        ));
    }
    if shape.style.fill.is_none() {
        return Err(TestSupportError::expectation(
            "expected a default fill to be applied on close",
        ));
    }
    if shape.path.anchors.len() != 3 {
        return Err(TestSupportError::expectation(
            "expected closing the path not to add anchors",
        ));
    }
    Ok(())
}

fn assert_click_does_not_place_points(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    click_point: gpui::Point<Pixels>,
) -> TestSupportResult<()> {
    draw_point(visual_cx, click_point);

    let doc_after_click = visual_cx.read(|app| view.read(app).document().clone());
    if doc_after_click.len() != 2 {
        return Err(TestSupportError::expectation(
            "after closing, additional clicks should not place draw points",
        ));
    }
    let shape_after_click = require_draw_shape(&doc_after_click, "after post-close click")?;
    if shape_after_click.path.anchors.len() != 3 {
        return Err(TestSupportError::expectation(
            "after closing, additional clicks should not mutate the closed path",
        ));
    }
    Ok(())
}

#[gpui::test]
fn clicking_near_first_anchor_closes_path_and_enters_manipulate(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let (p1, p2, p3) = triangle_points(&bounds);

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);
    draw_point(visual_cx, p3);

    let doc_before_close = visual_cx.read(|app| view.read(app).document().clone());
    let expected_shape_id =
        assert_open_triangle(&doc_before_close).expect("expected an open triangle before close");
    let closed_shape_id = close_path_by_clicking_first_anchor(visual_cx, &view, &bounds, p1)
        .expect("expected to close the path by clicking the first anchor");
    assert_eq!(
        closed_shape_id, expected_shape_id,
        "expected close operation to target the drawn shape"
    );

    let doc_after_close = visual_cx.read(|app| view.read(app).document().clone());
    assert_closed_shape(&doc_after_close, expected_shape_id).expect("expected shape to be closed");

    let after_close_click = point(bounds.origin.x + px(20.0), bounds.origin.y + px(20.0));
    assert_click_does_not_place_points(visual_cx, &view, after_close_click)
        .expect("expected clicks to avoid adding points after close");
}

#[gpui::test]
fn closing_in_bezier_mode_uses_cubic_closing_segment(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let (p1, p2, p3) = triangle_points(&bounds);

    draw_point(visual_cx, p1);
    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();

    draw_point(visual_cx, p2);
    draw_point(visual_cx, p3);

    let closed_shape_id = close_path_by_clicking_first_anchor(visual_cx, &view, &bounds, p1)
        .expect("expected to close the path by clicking the first anchor");

    let doc_after_close = visual_cx.read(|app| view.read(app).document().clone());
    let shape = require_draw_shape(&doc_after_close, "after bezier close")
        .expect("expected draw shape after bezier close");
    assert_eq!(
        shape.id, closed_shape_id,
        "expected close to preserve the shape id"
    );
    assert!(
        shape.path.closed,
        "expected closing in bezier mode to still close the path"
    );
    assert_eq!(
        shape.path.closing_segment,
        SegmentKind::Cubic,
        "expected bezier mode to use a cubic closing segment"
    );
    assert!(
        shape
            .path
            .anchors
            .first()
            .and_then(|a| a.handle_in)
            .is_some(),
        "expected closing cubic segment to set handle_in on the first anchor"
    );
    assert!(
        shape
            .path
            .anchors
            .last()
            .and_then(|a| a.handle_out)
            .is_some(),
        "expected closing cubic segment to set handle_out on the last anchor"
    );
}

#[gpui::test]
fn clicking_first_anchor_before_third_point_does_not_close_path(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let (p1, p2, _p3) = triangle_points(&bounds);

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);

    // With only two anchors present, clicking the first anchor should append a
    // third point rather than closing the shape.
    draw_point(visual_cx, p1);

    let doc_after_click = visual_cx.read(|app| view.read(app).document().clone());
    let shape = require_draw_shape(
        &doc_after_click,
        "after clicking first anchor with two points",
    )
    .expect("expected draw shape after third click on first anchor");

    assert!(
        !shape.path.closed,
        "expected path to remain open until at least three anchors existed before the close check"
    );
    assert_eq!(
        shape.path.anchors.len(),
        3,
        "expected click to append a third anchor instead of closing"
    );
    assert_eq!(
        shape.path.segments.len(),
        2,
        "expected two segments after three anchor placements"
    );
    assert!(
        shape.style.fill.is_none(),
        "expected no fill while the path remains open"
    );
}
