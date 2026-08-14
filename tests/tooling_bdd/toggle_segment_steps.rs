//! Step definitions for toggling a selected segment and undoing the change.

use crate::{common, state};
use gauss::model::{SegmentKind, SelItem, Shape, ShapeId, Vec2};
use gpui::{Modifiers, MouseButton, TestAppContext, point, px};
use rstest_bdd_macros::{given, then, when};
use test_support::{TestSupportError, TestSupportResult, math};

struct ToggleSegmentState {
    start: Vec2,
    end: Vec2,
    history_before: usize,
}

fn select_first_segment(
    visual_cx: &mut gpui::VisualTestContext,
    view: &gpui::Entity<gauss::ui::Phase0Shell>,
    point: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    let segment = SelItem::Segment {
        shape: shape_id,
        seg: 0,
    };
    if !selection.contains(&SelItem::Shape(shape_id)) || !selection.contains(&segment) {
        return Err(TestSupportError::expectation(format!(
            "expected shape and first segment selected; got {selection:?}"
        )));
    }
    visual_cx.simulate_mouse_up(point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

#[expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]
fn assert_cubic_with_initial_handles(
    shape: &Shape,
    start: Vec2,
    end: Vec2,
) -> TestSupportResult<()> {
    if shape.path.segments.first() != Some(&SegmentKind::Cubic) {
        return Err(TestSupportError::expectation(
            "expected the selected segment to become cubic",
        ));
    }
    let start_anchor = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "after toggle"))?;
    let end_anchor = shape
        .path
        .anchors
        .get(1)
        .ok_or_else(|| TestSupportError::missing("anchor 1", "after toggle"))?;
    let handle_out = start_anchor
        .handle_out
        .ok_or_else(|| TestSupportError::missing("handle_out", "after line-to-cubic toggle"))?;
    let handle_in = end_anchor
        .handle_in
        .ok_or_else(|| TestSupportError::missing("handle_in", "after line-to-cubic toggle"))?;
    let third = end.sub(start).mul(1.0 / 3.0);
    common::assert_vec2_close(
        handle_out,
        start.add(third),
        "handle_out should be one third along the line",
    )?;
    common::assert_vec2_close(
        handle_in,
        end.sub(third),
        "handle_in should be two thirds along the line",
    )
}

fn assert_line_without_handles(shape: &Shape) -> TestSupportResult<()> {
    if shape.path.segments.first() != Some(&SegmentKind::Line) {
        return Err(TestSupportError::expectation(
            "expected undo to restore the line segment",
        ));
    }
    let start = shape
        .path
        .anchors
        .first()
        .ok_or_else(|| TestSupportError::missing("anchor 0", "after undo"))?;
    let end = shape
        .path
        .anchors
        .get(1)
        .ok_or_else(|| TestSupportError::missing("anchor 1", "after undo"))?;
    if start.handle_out.is_some() || end.handle_in.is_some() {
        return Err(TestSupportError::expectation(
            "expected undo to clear the synthesized segment handles",
        ));
    }
    Ok(())
}

#[given("a fresh Phase 0 shell window with a selected line segment")]
fn shell_with_selected_line_segment(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        let first = point(
            bounds.origin.x + px(common::CANVAS_PADDING_PX),
            bounds.origin.y + px(common::CANVAS_PADDING_PX),
        );
        let second = point(
            bounds.origin.x + bounds.size.width - px(common::CANVAS_PADDING_PX),
            bounds.origin.y + bounds.size.height - px(common::CANVAS_PADDING_PX),
        );
        common::click_canvas_and_wait(visual_cx, first);
        common::click_canvas_and_wait(visual_cx, second);
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after drawing")?.clone();
        let start = shape
            .path
            .anchors
            .first()
            .ok_or_else(|| TestSupportError::missing("first anchor", "after drawing"))?
            .pos;
        let end = shape
            .path
            .anchors
            .get(1)
            .ok_or_else(|| TestSupportError::missing("second anchor", "after drawing"))?
            .pos;
        common::simulate_escape(visual_cx);
        let midpoint = Vec2::new(
            math::midpoint(start.x, end.x),
            math::midpoint(start.y, end.y),
        );
        let select_point = common::anchor_to_canvas_point(&bounds, midpoint, first);
        select_first_segment(visual_cx, view, select_point, shape.id)?;
        Ok(ToggleSegmentState {
            start,
            end,
            history_before: common::read_history_len(visual_cx, view),
        })
    })
}

#[when("Tab is pressed")]
fn press_tab(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut ToggleSegmentState| {
        common::simulate_key(visual_cx, "tab", Modifiers::none());
        Ok(())
    })
}

#[then("the selected segment is cubic with initial handles")]
fn segment_is_cubic_with_handles(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut ToggleSegmentState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after segment toggle")?;
        assert_cubic_with_initial_handles(shape, data.start, data.end)
    })
}

#[then("one segment-toggle history entry is added")]
fn history_entry_is_added(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut ToggleSegmentState| {
        if common::read_history_len(visual_cx, view) != data.history_before + 1 {
            return Err(TestSupportError::expectation(
                "expected one undo entry for the segment toggle",
            ));
        }
        Ok(())
    })
}

#[when("the last document change is undone")]
fn undo_last_document_change(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut ToggleSegmentState| {
        common::simulate_document_undo(visual_cx);
        Ok(())
    })
}

#[then("the selected segment is a line without handles")]
fn segment_is_line_without_handles(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut ToggleSegmentState| {
        let doc = common::read_document(visual_cx, view);
        let shape = common::require_draw_shape(&doc, "after undo")?;
        assert_line_without_handles(shape)
    })
}
