//! GPUI headless integration tests for mouse navigation buttons.
//!
//! Phase 0 maps mouse "back/forward" navigation buttons to undo/redo. Holding
//! Shift switches to the selection history stack.
#![expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]

mod common;

use common::{
    anchor_to_canvas_point, canvas_bounds, draw_point, ensure_initial_draw, init_test_app,
    require_draw_shape, shift_secondary, simulate_escape,
};
use gauss::model::{Document, Paint, PaintStyle, Rgba, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    Hsla, Modifiers, MouseButton, NavigationDirection, TestAppContext, VisualTestContext, point, px,
};
use test_support::{TestSupportError, TestSupportResult};

fn click_button(
    visual_cx: &mut VisualTestContext,
    position: gpui::Point<gpui::Pixels>,
    button: MouseButton,
    modifiers: Modifiers,
) {
    visual_cx.simulate_mouse_down(position, button, modifiers);
    visual_cx.simulate_mouse_up(position, button, modifiers);
    visual_cx.run_until_parked();
}

fn canvas_points(
    visual_cx: &mut VisualTestContext,
) -> TestSupportResult<(gpui::Bounds<gpui::Pixels>, gpui::Point<gpui::Pixels>)> {
    let bounds = canvas_bounds(visual_cx)?;
    let p1 = point(
        bounds.origin.x + px(common::CANVAS_PADDING_PX),
        bounds.origin.y + px(common::CANVAS_PADDING_PX),
    );
    Ok((bounds, p1))
}

fn second_point(
    bounds: &gpui::Bounds<gpui::Pixels>,
    p1: gpui::Point<gpui::Pixels>,
) -> gpui::Point<gpui::Pixels> {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    let dx = (width - 4.0).clamp(1.0, 40.0);
    let dy = (height - 4.0).clamp(1.0, 24.0);
    point(p1.x + px(dx), p1.y + px(dy))
}

fn clear_point(bounds: &gpui::Bounds<gpui::Pixels>) -> gpui::Point<gpui::Pixels> {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    let clear_x = (width - 12.0).max(1.0);
    let clear_y = (height - 12.0).max(1.0);
    point(bounds.origin.x + px(clear_x), bounds.origin.y + px(clear_y))
}

fn select_point_for_anchor0(
    bounds: &gpui::Bounds<gpui::Pixels>,
    anchor0: Vec2,
    p1: gpui::Point<gpui::Pixels>,
) -> gpui::Point<gpui::Pixels> {
    anchor_to_canvas_point(bounds, anchor0, p1)
}

fn draw_two_points_and_select_anchor0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> TestSupportResult<(
    gpui::Bounds<gpui::Pixels>,
    gpui::Point<gpui::Pixels>,
    PaintStyle,
)> {
    let (bounds, p1) = canvas_points(visual_cx)?;
    let p2 = second_point(&bounds, p1);

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);
    visual_cx.run_until_parked();

    let doc_before = visual_cx.read(|app| view.read(app).document().clone());
    let draw_shape_before = require_draw_shape(&doc_before, "after drawing")?;
    let initial_style = draw_shape_before.style.clone();

    simulate_escape(visual_cx);

    let anchor0 = draw_shape_before
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |anchor| anchor.pos);
    let select_point = select_point_for_anchor0(&bounds, anchor0, p1);
    click_button(
        visual_cx,
        select_point,
        MouseButton::Left,
        Modifiers::none(),
    );

    Ok((bounds, p1, initial_style))
}

fn apply_red_stroke(visual_cx: &mut VisualTestContext, view: &gpui::Entity<Phase0Shell>) {
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.apply_stroke_colour(Some(Hsla::red()));
        });
    });
    visual_cx.run_until_parked();
}

fn expect_stroke_is_red(doc: &Document, context: &str) -> TestSupportResult<()> {
    let shape = require_draw_shape(doc, context)?;
    if shape.style.stroke != Paint::Solid(Rgba::new(255, 0, 0, 255)) {
        return Err(TestSupportError::expectation(format!(
            "expected stroke to be red ({context})"
        )));
    }
    Ok(())
}

#[gpui::test]
fn navigation_buttons_undo_redo_document_history(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let (_bounds, p1, initial_style) = draw_two_points_and_select_anchor0(visual_cx, &view)
        .expect("expected to draw and select the first anchor");
    apply_red_stroke(visual_cx, &view);

    let doc_after_style = visual_cx.read(|app| view.read(app).document().clone());
    expect_stroke_is_red(&doc_after_style, "after applying style")
        .expect("expected red stroke after applying style");

    click_button(
        visual_cx,
        p1,
        MouseButton::Navigate(NavigationDirection::Back),
        Modifiers::none(),
    );

    let doc_after_undo = visual_cx.read(|app| view.read(app).document().clone());
    let shape_after_undo = require_draw_shape(&doc_after_undo, "after doc undo")
        .expect("expected draw shape after undo");
    assert_eq!(
        shape_after_undo.style, initial_style,
        "expected navigation back to undo the last document edit"
    );

    click_button(
        visual_cx,
        p1,
        MouseButton::Navigate(NavigationDirection::Forward),
        Modifiers::none(),
    );

    let doc_after_redo = visual_cx.read(|app| view.read(app).document().clone());
    expect_stroke_is_red(&doc_after_redo, "after doc redo")
        .expect("expected stroke to be red after redo");
}

#[gpui::test]
fn navigation_buttons_undo_redo_selection_history_with_shift(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let (bounds, p1, _initial_style) = draw_two_points_and_select_anchor0(visual_cx, &view)
        .expect("expected to draw and select the first anchor");
    apply_red_stroke(visual_cx, &view);

    let selection_before_clear = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        !selection_before_clear.items.is_empty(),
        "expected selection to be non-empty after selecting anchor"
    );

    let clear_point = clear_point(&bounds);
    click_button(visual_cx, clear_point, MouseButton::Left, Modifiers::none());

    let selection_after_clear = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_clear.items.is_empty(),
        "expected selection to be cleared"
    );

    click_button(
        visual_cx,
        p1,
        MouseButton::Navigate(NavigationDirection::Back),
        shift_secondary(Modifiers::none()),
    );

    let selection_after_undo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_undo, selection_before_clear,
        "expected Shift+navigation back to undo selection changes"
    );

    let doc_after_selection_undo = visual_cx.read(|app| view.read(app).document().clone());
    expect_stroke_is_red(&doc_after_selection_undo, "after selection undo")
        .expect("expected stroke to remain red after selection undo");

    click_button(
        visual_cx,
        p1,
        MouseButton::Navigate(NavigationDirection::Forward),
        shift_secondary(Modifiers::none()),
    );

    let selection_after_redo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_redo, selection_after_clear,
        "expected Shift+navigation forward to redo selection changes"
    );
}
