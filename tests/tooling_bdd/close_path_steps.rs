//! Step definitions for draw-mode path-closing scenarios.

use crate::{common, state};
use gauss::model::{Document, SegmentKind, ShapeId};
use gpui::{Bounds, Pixels, Point, TestAppContext, VisualTestContext, point, px};
use rstest_bdd_macros::{given, then, when};
use test_support::{TestSupportError, TestSupportResult};

pub(crate) struct ClosePathState {
    bounds: Bounds<Pixels>,
    points: [Point<Pixels>; 3],
    expected_shape_id: Option<ShapeId>,
}

fn triangle_points(bounds: &Bounds<Pixels>) -> [Point<Pixels>; 3] {
    [
        point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0)),
        point(
            bounds.origin.x + bounds.size.width - px(2.0),
            bounds.origin.y + px(12.0),
        ),
        point(
            bounds.origin.x + bounds.size.width - px(12.0),
            bounds.origin.y + bounds.size.height - px(2.0),
        ),
    ]
}

fn assert_open_triangle(doc: &Document) -> TestSupportResult<ShapeId> {
    if doc.len() != 2 {
        return Err(TestSupportError::expectation(
            "expected demo + one draw shape before close",
        ));
    }
    let shape = common::require_draw_shape(doc, "before close")?;
    if shape.path.closed || shape.path.anchors.len() != 3 {
        return Err(TestSupportError::expectation(
            "expected an open path with three anchors before close",
        ));
    }
    Ok(shape.id)
}

fn close_path(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<gauss::ui::Phase0Shell>,
    data: &ClosePathState,
) -> TestSupportResult<ShapeId> {
    let doc = common::read_document(visual_cx, view);
    let shape = common::require_draw_shape(&doc, "before close")?;
    let first = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "before close"))?;
    let close_point = common::anchor_to_canvas_point(&data.bounds, first.pos, data.points[0]);
    common::draw_point(visual_cx, close_point);
    Ok(shape.id)
}

fn is_expected_closed_shape(shape: &gauss::model::Shape, expected: ShapeId) -> bool {
    shape.id == expected && shape.path.closed && !shape.style.fill.is_none()
}

const fn is_open_three_anchor_path(shape: &gauss::model::Shape) -> bool {
    !shape.path.closed && shape.path.anchors.len() == 3 && shape.path.segments.len() == 2
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, _view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        Ok(ClosePathState {
            points: triangle_points(&bounds),
            bounds,
            expected_shape_id: None,
        })
    })
}

#[when("three triangle anchors are placed")]
fn place_three_triangle_anchors(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut ClosePathState| {
        for point in data.points {
            common::draw_point(visual_cx, point);
        }
        Ok(())
    })
}

#[then("the triangle path is open")]
fn triangle_path_is_open(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut ClosePathState| {
        data.expected_shape_id = Some(assert_open_triangle(&common::read_document(
            visual_cx, view,
        ))?);
        Ok(())
    })
}

#[when("the first triangle anchor is clicked again")]
fn click_first_triangle_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut ClosePathState| {
        let shape_id = close_path(visual_cx, view, data)?;
        if data.expected_shape_id.is_none() {
            data.expected_shape_id = Some(shape_id);
        }
        Ok(())
    })
}

#[then("the triangle path is closed")]
fn triangle_path_is_closed(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut ClosePathState| {
        let expected = data.expected_shape_id.ok_or_else(|| {
            TestSupportError::missing("expected shape id", "recorded before closing")
        })?;
        let doc = common::read_document(visual_cx, view);
        if doc.len() != 2 {
            return Err(TestSupportError::expectation(
                "closing the path should not create a new shape",
            ));
        }
        let shape = common::require_draw_shape(&doc, "after close")?;
        if !is_expected_closed_shape(shape, expected) {
            return Err(TestSupportError::expectation(
                "expected the same closed, filled draw shape after close",
            ));
        }
        if shape.path.anchors.len() != 3 {
            return Err(TestSupportError::expectation(
                "expected closing the path not to add anchors",
            ));
        }
        Ok(())
    })
}

#[then("the closing operation preserves the drawn shape")]
fn closing_preserves_drawn_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut ClosePathState| {
        let expected = data.expected_shape_id.ok_or_else(|| {
            TestSupportError::missing("expected shape id", "recorded before closing")
        })?;
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after close")?;
        if shape.id != expected {
            return Err(TestSupportError::expectation(
                "expected close operation to target the drawn shape",
            ));
        }
        Ok(())
    })
}

#[then("the shell is in manipulate mode")]
fn shell_is_in_manipulate_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut ClosePathState| {
        if !visual_cx.read(|app| view.read(app).is_manipulate_mode()) {
            return Err(TestSupportError::expectation(
                "expected closing the path to enter manipulate mode",
            ));
        }
        Ok(())
    })
}

#[when("the canvas is clicked away from the closed path")]
fn click_away_from_closed_path(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut ClosePathState| {
        let click = point(
            data.bounds.origin.x + px(20.0),
            data.bounds.origin.y + px(20.0),
        );
        common::draw_point(visual_cx, click);
        Ok(())
    })
}

#[then("no point is added to the closed path")]
fn no_point_is_added(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut ClosePathState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after post-close click")?;
        if doc.len() != 2 || shape.path.anchors.len() != 3 {
            return Err(TestSupportError::expectation(
                "after closing, additional clicks must not place draw points",
            ));
        }
        Ok(())
    })
}

#[when("the first triangle anchor is placed")]
fn place_first_triangle_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut ClosePathState| {
        common::draw_point(visual_cx, data.points[0]);
        Ok(())
    })
}

#[when("the remaining triangle anchors are placed")]
fn place_remaining_triangle_anchors(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut ClosePathState| {
        common::draw_point(visual_cx, data.points[1]);
        common::draw_point(visual_cx, data.points[2]);
        Ok(())
    })
}

#[then("the triangle path is closed with a cubic closing segment")]
fn triangle_has_cubic_close(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut ClosePathState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after Bezier close")?;
        if !shape.path.closed || shape.path.closing_segment != SegmentKind::Cubic {
            return Err(TestSupportError::expectation(
                "expected Bezier mode to close with a cubic segment",
            ));
        }
        Ok(())
    })
}

#[then("the first and last anchors have closing handles")]
fn closing_anchors_have_handles(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut ClosePathState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after Bezier close")?;
        let has_first = shape
            .path
            .anchors
            .first()
            .and_then(|a| a.handle_in)
            .is_some();
        let has_last = shape
            .path
            .anchors
            .last()
            .and_then(|a| a.handle_out)
            .is_some();
        if !has_first || !has_last {
            return Err(TestSupportError::expectation(
                "expected closing handles on the first and last anchors",
            ));
        }
        Ok(())
    })
}

#[when("two triangle anchors are placed")]
fn place_two_triangle_anchors(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut ClosePathState| {
        common::draw_point(visual_cx, data.points[0]);
        common::draw_point(visual_cx, data.points[1]);
        Ok(())
    })
}

#[then("the path remains open with 3 anchors and 2 segments")]
fn path_remains_open_with_three_anchors(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut ClosePathState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after early first-anchor click")?;
        if !is_open_three_anchor_path(shape) {
            return Err(TestSupportError::expectation(
                "expected an open path with three anchors and two segments",
            ));
        }
        Ok(())
    })
}

#[then("the open path has no fill")]
fn open_path_has_no_fill(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut ClosePathState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after early first-anchor click")?;
        if !shape.style.fill.is_none() {
            return Err(TestSupportError::expectation(
                "expected no fill while the path remains open",
            ));
        }
        Ok(())
    })
}
