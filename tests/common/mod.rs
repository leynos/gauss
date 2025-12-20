//! Shared helpers for Phase 0 GPUI integration tests.
#![allow(
    dead_code,
    reason = "integration tests each use a subset of the shared helper module"
)]
// Each integration test pulls in this module but only uses a subset of the
// helpers, so allow dead code to keep the shared test surface in one place.

use gauss::model::{Document, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    Bounds, KeyDownEvent, Keystroke, Modifiers, MouseButton, Pixels, Point, TestAppContext,
    VisualTestContext, point, px,
};
use uuid::Uuid;

pub const CANVAS_PADDING_PX: f32 = 2.0;
pub const MIN_CANVAS_HEIGHT_PX: f32 = 200.0;

pub fn init_test_app(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);
}

pub fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| {
        let _draw = window.draw(app);
    });
    visual_cx.run_until_parked();
}

pub fn demo_shape_id() -> ShapeId {
    ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a))
}

pub fn canvas_bounds(visual_cx: &mut VisualTestContext) -> Bounds<Pixels> {
    visual_cx
        .debug_bounds("#phase0-canvas")
        .unwrap_or_else(|| panic!("phase0 canvas should have debug bounds after drawing"))
}

pub fn canvas_points(visual_cx: &mut VisualTestContext) -> (Point<Pixels>, Point<Pixels>) {
    let bounds = canvas_bounds(visual_cx);
    let first = point(
        bounds.origin.x + px(CANVAS_PADDING_PX),
        bounds.origin.y + px(CANVAS_PADDING_PX),
    );
    let second = point(
        bounds.origin.x + bounds.size.width - px(CANVAS_PADDING_PX),
        bounds.origin.y + bounds.size.height - px(CANVAS_PADDING_PX),
    );
    (first, second)
}

pub fn click_canvas(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
}

pub fn click_canvas_and_wait(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    click_canvas(visual_cx, position);
    visual_cx.run_until_parked();
}

pub fn click_left(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    visual_cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
}

pub fn click_left_and_wait(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    click_left(visual_cx, position);
    visual_cx.run_until_parked();
}

pub fn draw_point(visual_cx: &mut VisualTestContext, position: Point<Pixels>) {
    click_canvas_and_wait(visual_cx, position);
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasDragScenario {
    pub bounds: Bounds<Pixels>,
    pub first: Point<Pixels>,
    pub second: Point<Pixels>,
    pub drag_end: Point<Pixels>,
    pub delta: Vec2,
}

pub fn canvas_drag_scenario(
    visual_cx: &mut VisualTestContext,
    horizontal_limit: f32,
    vertical_limit: f32,
) -> CanvasDragScenario {
    let bounds = canvas_bounds(visual_cx);

    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    let first = point(
        bounds.origin.x + px(CANVAS_PADDING_PX),
        bounds.origin.y + px(CANVAS_PADDING_PX),
    );
    let second = point(
        bounds.origin.x + px((width - CANVAS_PADDING_PX).max(CANVAS_PADDING_PX)),
        bounds.origin.y + px((height - CANVAS_PADDING_PX).max(CANVAS_PADDING_PX)),
    );

    let max_horizontal_delta = (width - (2.0 * CANVAS_PADDING_PX)).max(1.0);
    let max_vertical_delta = (height - (2.0 * CANVAS_PADDING_PX)).max(1.0);
    let horizontal_delta = max_horizontal_delta.min(horizontal_limit);
    let vertical_delta = max_vertical_delta.min(vertical_limit);
    let drag_end = point(first.x + px(horizontal_delta), first.y + px(vertical_delta));

    CanvasDragScenario {
        bounds,
        first,
        second,
        drag_end,
        delta: Vec2::new(horizontal_delta, vertical_delta),
    }
}

pub fn read_document(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> Document {
    visual_cx.read(|app| view.read(app).document().clone())
}

pub fn find_draw_shape(doc: &Document) -> Option<&Shape> {
    let demo_id = demo_shape_id();
    doc.shapes.iter().find(|shape| shape.id != demo_id)
}

pub fn require_draw_shape<'a>(doc: &'a Document, context: &str) -> &'a Shape {
    find_draw_shape(doc).unwrap_or_else(|| panic!("expected draw shape to exist: {context}"))
}

pub fn require_last_canvas_click(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    context: &str,
) -> Vec2 {
    visual_cx
        .read(|app| view.read(app).last_canvas_click_screen())
        .unwrap_or_else(|| panic!("expected Phase0Shell to observe a canvas click: {context}"))
}

pub fn require_canvas_click_changed(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    previous: Vec2,
    context: &str,
) -> Vec2 {
    let next = require_last_canvas_click(visual_cx, view, context);
    assert_ne!(
        next, previous,
        "expected a distinct canvas click: {context}"
    );
    next
}

pub fn simulate_key(visual_cx: &mut VisualTestContext, key: &str, modifiers: Modifiers) {
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

pub fn simulate_escape(visual_cx: &mut VisualTestContext) {
    simulate_key(visual_cx, "escape", Modifiers::none());
}

pub const fn shift_secondary(modifiers: Modifiers) -> Modifiers {
    let mut next = modifiers;
    next.shift = true;
    next
}

pub fn simulate_document_undo(visual_cx: &mut VisualTestContext) {
    simulate_key(visual_cx, "z", Modifiers::secondary_key());
}

pub fn assert_vec2_close(actual: Vec2, expected: Vec2, context: &str) {
    let diff = actual.sub(expected);
    assert!(
        diff.distance_squared(Vec2::ZERO) <= 0.0001,
        "{context}: expected={expected:?} got={actual:?}"
    );
}

pub fn anchor_to_canvas_point(
    bounds: &Bounds<Pixels>,
    anchor: Vec2,
    reference: Point<Pixels>,
) -> Point<Pixels> {
    let expected_local = Vec2::new(CANVAS_PADDING_PX, CANVAS_PADDING_PX);
    let expected_abs = Vec2::new(f32::from(reference.x), f32::from(reference.y));
    let use_local =
        anchor.distance_squared(expected_local) <= anchor.distance_squared(expected_abs);
    if use_local {
        point(
            bounds.origin.x + px(anchor.x),
            bounds.origin.y + px(anchor.y),
        )
    } else {
        point(px(anchor.x), px(anchor.y))
    }
}
