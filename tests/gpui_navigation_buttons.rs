//! GPUI headless integration tests for mouse navigation buttons.
//!
//! Phase 0 maps mouse "back/forward" navigation buttons to undo/redo. Holding
//! Shift switches to the selection history stack.

use gauss::model::{Document, PaintStyle, Rgba, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    Hsla, KeyDownEvent, Keystroke, Modifiers, MouseButton, NavigationDirection, TestAppContext,
    VisualTestContext, point, px,
};
use uuid::Uuid;

fn demo_shape_id() -> ShapeId {
    ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a))
}

fn require_draw_shape<'a>(doc: &'a Document, context: &str) -> &'a Shape {
    let demo_id = demo_shape_id();
    let Some(shape) = doc.shapes.iter().find(|shape| shape.id != demo_id) else {
        panic!("expected draw shape to exist: {context}");
    };
    shape
}

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

fn simulate_key(visual_cx: &mut VisualTestContext, key: &str, modifiers: Modifiers) {
    visual_cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            modifiers,
            key: key.to_owned(),
            key_char: None,
        },
        is_held: false,
    });
    visual_cx.run_until_parked();
}

fn draw_point(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
}

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

const fn with_shift(mut modifiers: Modifiers) -> Modifiers {
    modifiers.shift = true;
    modifiers
}

fn canvas_points(
    visual_cx: &mut VisualTestContext,
) -> (gpui::Bounds<gpui::Pixels>, gpui::Point<gpui::Pixels>) {
    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };

    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    (bounds, p1)
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
    let expected_local = Vec2::new(2.0, 2.0);
    let expected_abs = Vec2::new(f32::from(p1.x), f32::from(p1.y));
    let use_local_coords =
        anchor0.distance_squared(expected_local) <= anchor0.distance_squared(expected_abs);

    if use_local_coords {
        point(
            bounds.origin.x + px(anchor0.x),
            bounds.origin.y + px(anchor0.y),
        )
    } else {
        point(px(anchor0.x), px(anchor0.y))
    }
}

fn draw_two_points_and_select_anchor0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> (
    gpui::Bounds<gpui::Pixels>,
    gpui::Point<gpui::Pixels>,
    PaintStyle,
) {
    let (bounds, p1) = canvas_points(visual_cx);
    let p2 = second_point(&bounds, p1);

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);
    visual_cx.run_until_parked();

    let doc_before = visual_cx.read(|app| view.read(app).document().clone());
    let draw_shape_before = require_draw_shape(&doc_before, "after drawing");
    let initial_style = draw_shape_before.style.clone();

    simulate_key(visual_cx, "escape", Modifiers::none());

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

    (bounds, p1, initial_style)
}

fn apply_red_stroke(visual_cx: &mut VisualTestContext, view: &gpui::Entity<Phase0Shell>) {
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.apply_stroke_colour(Some(Hsla::red()));
        });
    });
    visual_cx.run_until_parked();
}

fn expect_stroke_is_red(doc: &Document, context: &str) {
    let shape = require_draw_shape(doc, context);
    assert_eq!(
        shape.style.stroke,
        Some(Rgba::new(255, 0, 0, 255)),
        "expected stroke to be red ({context})"
    );
}

#[gpui::test]
fn navigation_buttons_undo_redo_document_history(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let (_bounds, p1, initial_style) = draw_two_points_and_select_anchor0(visual_cx, &view);
    apply_red_stroke(visual_cx, &view);

    let doc_after_style = visual_cx.read(|app| view.read(app).document().clone());
    expect_stroke_is_red(&doc_after_style, "after applying style");

    click_button(
        visual_cx,
        p1,
        MouseButton::Navigate(NavigationDirection::Back),
        Modifiers::none(),
    );

    let doc_after_undo = visual_cx.read(|app| view.read(app).document().clone());
    let shape_after_undo = require_draw_shape(&doc_after_undo, "after doc undo");
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
    expect_stroke_is_red(&doc_after_redo, "after doc redo");
}

#[gpui::test]
fn navigation_buttons_undo_redo_selection_history_with_shift(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let (bounds, p1, _initial_style) = draw_two_points_and_select_anchor0(visual_cx, &view);
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
        with_shift(Modifiers::none()),
    );

    let selection_after_undo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_undo, selection_before_clear,
        "expected Shift+navigation back to undo selection changes"
    );

    let doc_after_selection_undo = visual_cx.read(|app| view.read(app).document().clone());
    expect_stroke_is_red(&doc_after_selection_undo, "after selection undo");

    click_button(
        visual_cx,
        p1,
        MouseButton::Navigate(NavigationDirection::Forward),
        with_shift(Modifiers::none()),
    );

    let selection_after_redo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_redo, selection_after_clear,
        "expected Shift+navigation forward to redo selection changes"
    );
}
