//! Step definitions for committing an open path with Escape.

use crate::{common, state};
use gauss::model::ShapeId;
use gpui::{Pixels, Point, TestAppContext, point, px};
use rstest_bdd_macros::{given, then, when};
use test_support::TestSupportError;

pub(crate) struct DrawEscapeState {
    points: [Point<Pixels>; 2],
    shape_id: Option<ShapeId>,
    anchor_count: Option<usize>,
    segment_count: Option<usize>,
    document_len: Option<usize>,
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, _view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        Ok(DrawEscapeState {
            points: [
                point(bounds.origin.x + px(10.0), bounds.origin.y + px(10.0)),
                point(bounds.origin.x + px(80.0), bounds.origin.y + px(30.0)),
            ],
            shape_id: None,
            anchor_count: None,
            segment_count: None,
            document_len: None,
        })
    })
}

#[when("two distinct drawing anchors are placed")]
fn place_two_distinct_anchors(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut DrawEscapeState| {
        common::click_canvas_and_wait(visual_cx, data.points[0]);
        common::click_canvas_and_wait(visual_cx, data.points[1]);
        Ok(())
    })
}

#[then("the active path is open with two distinct anchors")]
fn active_path_is_open(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut DrawEscapeState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after drawing two points")?;
        if shape.path.closed {
            return Err(TestSupportError::expectation(
                "expected the newly drawn path to be open",
            ));
        }
        let first = shape
            .path
            .anchors
            .first()
            .ok_or_else(|| TestSupportError::missing("first anchor", "open path"))?;
        let second = shape
            .path
            .anchors
            .get(1)
            .ok_or_else(|| TestSupportError::missing("second anchor", "open path"))?;
        if first.pos == second.pos {
            return Err(TestSupportError::expectation(
                "expected two distinct anchors in the open path",
            ));
        }
        data.shape_id = Some(shape.id);
        data.anchor_count = Some(shape.path.anchors.len());
        data.segment_count = Some(shape.path.segments.len());
        data.document_len = Some(doc.len());
        Ok(())
    })
}

#[when("the canvas is clicked at the second anchor")]
fn click_second_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut DrawEscapeState| {
        common::click_canvas_and_wait(visual_cx, data.points[1]);
        Ok(())
    })
}

#[then("the same open path remains with unchanged anchor and segment counts")]
fn open_path_remains_unchanged(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut DrawEscapeState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after Escape and click")?;
        let unchanged = Some(shape.id) == data.shape_id
            && !shape.path.closed
            && Some(shape.path.anchors.len()) == data.anchor_count
            && Some(shape.path.segments.len()) == data.segment_count
            && Some(doc.len()) == data.document_len
            && visual_cx.read(|app| view.read(app).is_manipulate_mode());
        if !unchanged {
            return Err(TestSupportError::expectation(
                "expected Escape to enter manipulate mode, preserve the same open path and document length, and make the click add nothing",
            ));
        }
        Ok(())
    })
}
