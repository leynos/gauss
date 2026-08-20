//! Step definitions for Escape transitions out of manipulate mode.

use crate::{common, state};
use gauss::model::Anchor;
use gpui::{Modifiers, MouseButton, Pixels, Point, TestAppContext, point, px};
use rstest_bdd_macros::{given, then, when};
use test_support::{TestSupportError, math};

pub(crate) enum EscapeState {
    Click {
        point: Point<Pixels>,
        shapes_before: usize,
    },
    Drag {
        drag_start: Point<Pixels>,
        drag_preview: Point<Pixels>,
        history_before: usize,
        anchors_before: Vec<Anchor>,
    },
}

type DragStateFields<'a> = (
    &'a Point<Pixels>,
    &'a Point<Pixels>,
    &'a usize,
    &'a Vec<Anchor>,
);

impl EscapeState {
    fn as_click(
        &mut self,
        context: &'static str,
    ) -> Result<(&Point<Pixels>, &usize), TestSupportError> {
        let Self::Click {
            point,
            shapes_before,
        } = self
        else {
            return Err(TestSupportError::missing("click state", context));
        };
        Ok((point, shapes_before))
    }

    fn as_drag<'a>(
        &'a mut self,
        context: &'static str,
    ) -> Result<DragStateFields<'a>, TestSupportError> {
        let Self::Drag {
            drag_start,
            drag_preview,
            history_before,
            anchors_before,
        } = self
        else {
            return Err(TestSupportError::missing("drag state", context));
        };
        Ok((drag_start, drag_preview, history_before, anchors_before))
    }
}

fn shape_count(
    visual_cx: &gpui::VisualTestContext,
    view: &gpui::Entity<gauss::ui::Phase0Shell>,
) -> usize {
    visual_cx.read(|app| view.read(app).document().len())
}

fn draw_shape_anchors(
    visual_cx: &gpui::VisualTestContext,
    view: &gpui::Entity<gauss::ui::Phase0Shell>,
    context: &str,
) -> Result<Vec<Anchor>, TestSupportError> {
    let doc = common::read_document(visual_cx, view);
    Ok(common::require_draw_shape(&doc, context)?
        .path
        .anchors
        .clone())
}

fn drag_cancelled_without_changes(
    is_dragging: bool,
    history: (usize, usize),
    anchors: (&[Anchor], &[Anchor]),
) -> bool {
    !is_dragging && history.0 == history.1 && anchors.0 == anchors.1
}

#[given("a fresh Phase 0 shell window in manipulate mode")]
fn fresh_shell_in_manipulate_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        let point = point(bounds.origin.x + px(10.0), bounds.origin.y + px(10.0));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        Ok(EscapeState::Click {
            point,
            shapes_before: shape_count(visual_cx, view),
        })
    })
}

#[when("the canvas is clicked at the test point")]
fn click_test_point(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut EscapeState| {
        let (point, _) = data.as_click("test-point scenario")?;
        common::click_canvas_and_wait(visual_cx, *point);
        Ok(())
    })
}

#[then("no new shape is created")]
fn no_new_shape_is_created(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut EscapeState| {
        let (_, shapes_before) = data.as_click("test-point scenario")?;
        if shape_count(visual_cx, view) != *shapes_before {
            return Err(TestSupportError::expectation(
                "expected manipulate-mode click not to create a shape",
            ));
        }
        Ok(())
    })
}

#[then("one new shape is created")]
fn one_new_shape_is_created(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut EscapeState| {
        let (_, shapes_before) = data.as_click("test-point scenario")?;
        if shape_count(visual_cx, view) != shapes_before.saturating_add(1) {
            return Err(TestSupportError::expectation(
                "expected draw-mode click to create one new shape",
            ));
        }
        Ok(())
    })
}

#[given("a fresh Phase 0 shell window with a two-anchor path in manipulate mode")]
fn shell_with_two_anchor_path(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let scenario = common::canvas_drag_scenario(visual_cx, 20.0, 10.0)?;
        common::draw_point(visual_cx, scenario.first);
        common::draw_point(visual_cx, scenario.second);
        common::simulate_escape(visual_cx);
        let shapes_after_escape = shape_count(visual_cx, view);
        common::click_canvas_and_wait(visual_cx, scenario.first);
        if shape_count(visual_cx, view) != shapes_after_escape {
            return Err(TestSupportError::expectation(
                "expected manipulate mode after committing the open path",
            ));
        }
        let drag_start = point(
            px(math::midpoint(
                f32::from(scenario.first.x),
                f32::from(scenario.second.x),
            )),
            px(math::midpoint(
                f32::from(scenario.first.y),
                f32::from(scenario.second.y),
            )),
        );
        let drag_preview = point(
            drag_start.x + px(scenario.delta.x),
            drag_start.y + px(scenario.delta.y),
        );
        Ok(EscapeState::Drag {
            drag_start,
            drag_preview,
            history_before: common::read_history_len(visual_cx, view),
            anchors_before: draw_shape_anchors(visual_cx, view, "before drag preview")?,
        })
    })
}

#[when("a manipulate drag preview is started")]
fn start_drag_preview(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, data: &mut EscapeState| {
        let (drag_start, drag_preview, _, _) = data.as_drag("drag-preview scenario")?;
        visual_cx.simulate_mouse_down(*drag_start, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        visual_cx.simulate_mouse_move(*drag_preview, MouseButton::Left, Modifiers::none());
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the drag preview is active without a history commit")]
fn drag_preview_is_active_without_commit(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut EscapeState| {
        let (_, _, history_before, _) = data.as_drag("drag-preview scenario")?;
        let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
        if !is_dragging || common::read_history_len(visual_cx, view) != *history_before {
            return Err(TestSupportError::expectation(
                "expected an active preview without a document-history commit",
            ));
        }
        Ok(())
    })
}

#[then("the drag preview is cancelled without history or geometry changes")]
fn drag_preview_is_cancelled_without_changes(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut EscapeState| {
        let (_, _, history_before, anchors_before) = data.as_drag("drag-preview scenario")?;
        let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
        let anchors_after = draw_shape_anchors(visual_cx, view, "after Escape during preview")?;
        let history_after = common::read_history_len(visual_cx, view);
        if !drag_cancelled_without_changes(
            is_dragging,
            (history_after, *history_before),
            (&anchors_after, anchors_before),
        ) {
            return Err(TestSupportError::expectation(
                "expected Escape to cancel the preview without history or geometry changes",
            ));
        }
        Ok(())
    })
}
