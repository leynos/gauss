//! Step definitions for Bezier-auto drawing scenarios.

use crate::{common, state};
use gauss::model::{SegmentKind, Shape, Vec2};
use gpui::{Point, TestAppContext, point, px};
use rstest_bdd_macros::{given, then, when};
use test_support::{TestSupportError, TestSupportResult};

const CATMULL_ROM_TENSION: f32 = 1.0;

struct BezierAutoState {
    points: [Point<gpui::Pixels>; 4],
}

#[expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]
fn catmull_rom_controls(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> (Vec2, Vec2) {
    let tension = CATMULL_ROM_TENSION / 6.0;
    (
        p1.add(p2.sub(p0).mul(tension)),
        p2.sub(p3.sub(p1).mul(tension)),
    )
}

fn require_anchor_pos(shape: &Shape, index: usize) -> TestSupportResult<Vec2> {
    shape
        .path
        .anchors
        .get(index)
        .map(|anchor| anchor.pos)
        .ok_or_else(|| TestSupportError::missing(format!("anchor {index}"), "Bezier path"))
}

fn require_handle_out(shape: &Shape, index: usize) -> TestSupportResult<Vec2> {
    shape
        .path
        .anchors
        .get(index)
        .and_then(|anchor| anchor.handle_out)
        .ok_or_else(|| TestSupportError::missing(format!("handle_out {index}"), "Bezier path"))
}

fn require_handle_in(shape: &Shape, index: usize) -> TestSupportResult<Vec2> {
    shape
        .path
        .anchors
        .get(index)
        .and_then(|anchor| anchor.handle_in)
        .ok_or_else(|| TestSupportError::missing(format!("handle_in {index}"), "Bezier path"))
}

fn has_four_anchors_and_three_cubic_segments(shape: &Shape) -> bool {
    shape.path.anchors.len() == 4
        && shape.path.segments.len() == 3
        && shape
            .path
            .segments
            .iter()
            .all(|kind| *kind == SegmentKind::Cubic)
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, _view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        Ok(BezierAutoState {
            points: [
                point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0)),
                point(
                    bounds.origin.x + bounds.size.width - px(2.0),
                    bounds.origin.y + px(12.0),
                ),
                point(
                    bounds.origin.x + bounds.size.width - px(12.0),
                    bounds.origin.y + bounds.size.height - px(2.0),
                ),
                point(
                    bounds.origin.x + px(12.0),
                    bounds.origin.y + bounds.size.height - px(12.0),
                ),
            ],
        })
    })
}

#[when("the first of four drawing anchors is placed")]
fn place_first_anchor(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut BezierAutoState| {
        common::draw_point(visual_cx, data.points[0]);
        Ok(())
    })
}

#[when("the draw edge mode is switched to Bezier auto")]
fn switch_to_bezier_auto(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut BezierAutoState| {
        visual_cx.simulate_keystrokes("tab");
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[when("the remaining three drawing anchors are placed")]
fn place_remaining_anchors(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut BezierAutoState| {
        for point in &data.points[1..] {
            common::draw_point(visual_cx, *point);
        }
        Ok(())
    })
}

#[then("the path has 4 anchors and 3 cubic segments")]
fn path_has_cubic_segments(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut BezierAutoState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after drawing Bezier auto points")?;
        if !has_four_anchors_and_three_cubic_segments(shape) {
            return Err(TestSupportError::expectation(format!(
                "expected four anchors and three cubic segments; path={:?}",
                shape.path
            )));
        }
        Ok(())
    })
}

#[then("the middle cubic segment handles match the Catmull-Rom controls")]
fn middle_handles_match_catmull_rom(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut BezierAutoState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after drawing Bezier auto points")?;
        let a0 = require_anchor_pos(shape, 0)?;
        let a1 = require_anchor_pos(shape, 1)?;
        let a2 = require_anchor_pos(shape, 2)?;
        let a3 = require_anchor_pos(shape, 3)?;
        let (expected_out, expected_in) = catmull_rom_controls(a0, a1, a2, a3);
        common::assert_vec2_close(
            require_handle_out(shape, 1)?,
            expected_out,
            "segment1 handle_out",
        )?;
        common::assert_vec2_close(
            require_handle_in(shape, 2)?,
            expected_in,
            "segment1 handle_in",
        )
    })
}
