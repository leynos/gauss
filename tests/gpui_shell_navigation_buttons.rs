//! Behavioural mouse-navigation coverage through `GpuiHarness`.

#[path = "common/gpui_shell_navigation_buttons.rs"]
mod common;
#[path = "shell_bdd/expect_equal.rs"]
mod expect_equal_support;
#[path = "shell_bdd/expect_true.rs"]
mod expect_true_support;
#[path = "shell_bdd/support.rs"]
mod support;

use std::cell::RefCell;

use common::{
    anchor_to_canvas_point, canvas_bounds, draw_point, require_draw_shape, shift_secondary,
    simulate_escape,
};
use expect_equal_support::expect_equal;
use expect_true_support::expect_true;
use gauss::model::{Document, Paint, PaintStyle, Rgba, Selection, Vec2};
use gpui::{
    Bounds, Hsla, Modifiers, MouseButton, NavigationDirection, Pixels, Point, TestAppContext,
    VisualTestContext, point, px,
};
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, fresh_shell_with, with_shell};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct NavigationState {
    bounds: Option<Bounds<Pixels>>,
    click_point: Option<Point<Pixels>>,
    initial_style: Option<PaintStyle>,
    selection_before_clear: Option<Selection>,
    selection_after_clear: Option<Selection>,
}

thread_local! {
    static NAVIGATION_STATE: RefCell<NavigationState> = RefCell::new(NavigationState::default());
}

fn click_button(
    visual_cx: &mut VisualTestContext,
    position: Point<Pixels>,
    button: MouseButton,
    modifiers: Modifiers,
) {
    visual_cx.simulate_mouse_down(position, button, modifiers);
    visual_cx.simulate_mouse_up(position, button, modifiers);
    visual_cx.run_until_parked();
}

#[expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]
fn second_point(bounds: &Bounds<Pixels>, first: Point<Pixels>) -> Point<Pixels> {
    let dx = (f32::from(bounds.size.width) - 4.0).clamp(1.0, 40.0);
    let dy = (f32::from(bounds.size.height) - 4.0).clamp(1.0, 24.0);
    point(first.x + px(dx), first.y + px(dy))
}

#[expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]
fn clear_point(bounds: &Bounds<Pixels>) -> Point<Pixels> {
    let clear_x = (f32::from(bounds.size.width) - 12.0).max(1.0);
    let clear_y = (f32::from(bounds.size.height) - 12.0).max(1.0);
    point(bounds.origin.x + px(clear_x), bounds.origin.y + px(clear_y))
}

fn draw_and_select_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<gauss::ui::Phase0Shell>,
) -> TestSupportResult<(Bounds<Pixels>, Point<Pixels>, PaintStyle)> {
    let bounds = canvas_bounds(visual_cx)?;
    let first = point(
        bounds.origin.x + px(common::CANVAS_PADDING_PX),
        bounds.origin.y + px(common::CANVAS_PADDING_PX),
    );
    draw_point(visual_cx, first);
    draw_point(visual_cx, second_point(&bounds, first));
    visual_cx.run_until_parked();

    let document = common::read_document(visual_cx, view);
    let shape = require_draw_shape(&document, "after drawing")?;
    let initial_style = shape.style.clone();
    let anchor = shape
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |item| item.pos);
    simulate_escape(visual_cx);
    click_button(
        visual_cx,
        anchor_to_canvas_point(&bounds, anchor, first),
        MouseButton::Left,
        Modifiers::none(),
    );
    Ok((bounds, first, initial_style))
}

fn expect_stroke_is_red(document: &Document, context: &str) -> TestSupportResult<()> {
    let shape = require_draw_shape(document, context)?;
    expect_equal(
        &shape.style.stroke,
        &Paint::Solid(Rgba::new(255, 0, 0, 255)),
        format!("stroke colour {context}"),
    )
}

fn navigation_click(
    cx: &mut TestAppContext,
    direction: NavigationDirection,
    modifiers: Modifiers,
) -> TestSupportResult<()> {
    let position = NAVIGATION_STATE
        .with(|cell| cell.borrow().click_point)
        .ok_or_else(|| TestSupportError::missing("navigation click point", "scenario setup"))?;
    with_shell(cx, |visual_cx, _view| {
        click_button(
            visual_cx,
            position,
            MouseButton::Navigate(direction),
            modifiers,
        );
        Ok(())
    })
}

#[given("a selected anchor with a red stroke")]
fn selected_anchor_with_red_stroke(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    NAVIGATION_STATE.with(|cell| *cell.borrow_mut() = NavigationState::default());
    fresh_shell_with(cx, gauss::ui::Phase0Shell::new);
    with_shell(cx, |visual_cx, view| {
        let (bounds, click_point, initial_style) = draw_and_select_first_anchor(visual_cx, view)?;
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                shell.apply_stroke_colour(Some(Hsla::red()));
            });
        });
        visual_cx.run_until_parked();
        let document = common::read_document(visual_cx, view);
        expect_stroke_is_red(&document, "after applying style")?;
        NAVIGATION_STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            state.bounds = Some(bounds);
            state.click_point = Some(click_point);
            state.initial_style = Some(initial_style);
        });
        Ok(())
    })
}

#[when("navigation Back is clicked")]
fn click_navigation_back(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    navigation_click(cx, NavigationDirection::Back, Modifiers::none())
}

#[then("the original stroke is restored")]
fn original_stroke_is_restored(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = NAVIGATION_STATE
        .with(|cell| cell.borrow().initial_style.clone())
        .ok_or_else(|| TestSupportError::missing("initial style", "scenario setup"))?;
    with_shell(cx, |visual_cx, view| {
        let document = common::read_document(visual_cx, view);
        let shape = require_draw_shape(&document, "after document undo")?;
        expect_equal(&shape.style, &expected, "style after navigation Back")
    })
}

#[when("navigation Forward is clicked")]
fn click_navigation_forward(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    navigation_click(cx, NavigationDirection::Forward, Modifiers::none())
}

#[then("the stroke is red")]
fn stroke_is_red(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        expect_stroke_is_red(
            &common::read_document(visual_cx, view),
            "after document redo",
        )
    })
}

#[when("the selection is cleared")]
fn clear_selection(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let bounds = NAVIGATION_STATE
        .with(|cell| cell.borrow().bounds)
        .ok_or_else(|| TestSupportError::missing("canvas bounds", "scenario setup"))?;
    with_shell(cx, |visual_cx, view| {
        let before = visual_cx.read(|app| view.read(app).selection().clone());
        expect_true(
            !before.items.is_empty(),
            "expected selection before clearing",
        )?;
        click_button(
            visual_cx,
            clear_point(&bounds),
            MouseButton::Left,
            Modifiers::none(),
        );
        let after = visual_cx.read(|app| view.read(app).selection().clone());
        expect_true(after.items.is_empty(), "expected selection to be cleared")?;
        NAVIGATION_STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            state.selection_before_clear = Some(before);
            state.selection_after_clear = Some(after);
        });
        Ok(())
    })
}

#[when("Shift-navigation Back is clicked")]
fn click_shift_navigation_back(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    navigation_click(
        cx,
        NavigationDirection::Back,
        shift_secondary(Modifiers::none()),
    )
}

#[then("the previous selection is restored")]
fn previous_selection_is_restored(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = NAVIGATION_STATE
        .with(|cell| cell.borrow().selection_before_clear.clone())
        .ok_or_else(|| TestSupportError::missing("selection before clearing", "clear step"))?;
    with_shell(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).selection().clone());
        expect_equal(&actual, &expected, "selection after Shift-navigation Back")
    })
}

#[then("the stroke remains red")]
fn stroke_remains_red(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        expect_stroke_is_red(
            &common::read_document(visual_cx, view),
            "after selection undo",
        )
    })
}

#[when("Shift-navigation Forward is clicked")]
fn click_shift_navigation_forward(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    navigation_click(
        cx,
        NavigationDirection::Forward,
        shift_secondary(Modifiers::none()),
    )
}

#[then("the selection is cleared")]
fn selection_is_cleared(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let expected = NAVIGATION_STATE
        .with(|cell| cell.borrow().selection_after_clear.clone())
        .ok_or_else(|| TestSupportError::missing("cleared selection", "clear step"))?;
    with_shell(cx, |visual_cx, view| {
        let actual = visual_cx.read(|app| view.read(app).selection().clone());
        expect_equal(
            &actual,
            &expected,
            "selection after Shift-navigation Forward",
        )
    })
}

#[scenario(
    path = "tests/features/shell_navigation.feature",
    name = "Navigation buttons undo and redo document history",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn navigation_buttons_undo_and_redo_document_history(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}

#[scenario(
    path = "tests/features/shell_navigation.feature",
    name = "Shift navigation buttons undo and redo selection history",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn shift_navigation_buttons_undo_and_redo_selection_history(
    #[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
