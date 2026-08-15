//! Behavioural shell-style-control coverage through `GpuiHarness`.

#[path = "common/gpui_shell_style_controls.rs"]
mod common;

#[path = "common/durable_shell.rs"]
mod durable_shell;
#[path = "shell_bdd/expect_equal.rs"]
mod expect_equal_support;

#[path = "shell_bdd/lifecycle.rs"]
mod lifecycle;

#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "shell_bdd/support.rs"]
mod support;

use std::cell::RefCell;

use common::{
    anchor_to_canvas_point, click_canvas_and_wait, read_document, read_history_len,
    require_draw_shape, simulate_document_undo, simulate_escape,
};
use expect_equal_support::expect_equal;
use gauss::model::{Paint, PaintStyle, Rgba, SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Hsla, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, fresh_shell_with, with_shell};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct StyleState {
    original_style: Option<PaintStyle>,
    history_len_before_style: Option<usize>,
}

thread_local! {
    static STYLE_STATE: RefCell<StyleState> = RefCell::new(StyleState::default());
}

fn select_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    position: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    let expected = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    if !selection.contains(&expected) {
        return Err(TestSupportError::expectation(format!(
            "expected first anchor selection; selection={selection:?}"
        )));
    }
    visual_cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

#[given("a drawn shape with its first anchor selected")]
fn drawn_shape_with_first_anchor_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    STYLE_STATE.with(|cell| *cell.borrow_mut() = StyleState::default());
    fresh_shell_with(cx, Phase0Shell::new)?;
    with_shell(cx, |visual_cx, view| {
        let bounds = common::canvas_bounds(visual_cx)?;
        let first = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
        let second = point(
            bounds.origin.x + bounds.size.width - px(2.0),
            bounds.origin.y + bounds.size.height - px(2.0),
        );
        click_canvas_and_wait(visual_cx, first);
        click_canvas_and_wait(visual_cx, second);

        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after drawing")?;
        let shape_id = shape.id;
        let anchor = shape
            .path
            .anchors
            .first()
            .map_or(Vec2::ZERO, |item| item.pos);
        let original_style = shape.style.clone();
        simulate_escape(visual_cx);
        select_first_anchor(
            visual_cx,
            view,
            anchor_to_canvas_point(&bounds, anchor, first),
            shape_id,
        )?;
        STYLE_STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            state.original_style = Some(original_style);
            state.history_len_before_style = Some(read_history_len(visual_cx, view));
        });
        Ok(())
    })
}

#[when("the stroke is changed to red")]
fn change_stroke_to_red(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                shell.apply_stroke_colour(Some(Hsla::red()));
            });
        });
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[when("the fill is changed to blue")]
fn change_fill_to_blue(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                shell.apply_fill_colour(Some(Hsla::blue()));
            });
        });
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the shape has a red stroke and blue fill")]
fn shape_has_red_stroke_and_blue_fill(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after applying style")?;
        expect_equal(
            &shape.style.stroke,
            &Paint::Solid(Rgba::new(255, 0, 0, 255)),
            "stroke after style changes",
        )?;
        expect_equal(
            &shape.style.fill,
            &Paint::Solid(Rgba::new(0, 0, 255, 255)),
            "fill after style changes",
        )
    })
}

#[then("two document history entries are added")]
fn two_document_history_entries_are_added(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let before = STYLE_STATE
        .with(|cell| cell.borrow().history_len_before_style)
        .ok_or_else(|| TestSupportError::missing("initial history length", "scenario setup"))?;
    with_shell(cx, |visual_cx, view| {
        expect_equal(
            &read_history_len(visual_cx, view),
            &(before + 2),
            "history length after stroke and fill changes",
        )
    })
}

#[when("both style changes are undone")]
fn undo_both_style_changes(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        simulate_document_undo(visual_cx);
        simulate_document_undo(visual_cx);
        Ok(())
    })
}

#[then("the shape has its original style")]
fn shape_has_original_style(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = STYLE_STATE
        .with(|cell| cell.borrow().original_style.clone())
        .ok_or_else(|| TestSupportError::missing("original style", "scenario setup"))?;
    with_shell(cx, |visual_cx, view| {
        let document = read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after undo")?;
        expect_equal(&shape.style, &expected, "style after undo")
    })
}

#[scenario(
    path = "tests/features/shell_style_controls.feature",
    name = "Style changes apply to selected shapes and are undoable",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn style_changes_apply_to_selected_shapes_and_are_undoable(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
