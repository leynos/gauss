//! GPUI headless integration tests for Phase 0 selection history.

use gauss::model::{Document, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    KeyDownEvent, Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px,
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

fn click_left(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
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

const fn shift_secondary(mods: Modifiers) -> Modifiers {
    let mut modifiers = mods;
    modifiers.shift = true;
    modifiers
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

fn draw_point(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
}

#[gpui::test]
fn selection_undo_uses_shift_modified_stack(cx: &mut TestAppContext) {
    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };

    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));

    let dx = (width - 4.0).clamp(1.0, 40.0);
    let dy = (height - 4.0).clamp(1.0, 24.0);
    let p2 = point(p1.x + px(dx), p1.y + px(dy));

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);
    visual_cx.run_until_parked();

    let doc_before = visual_cx.read(|app| view.read(app).document().clone());
    let draw_shape = require_draw_shape(&doc_before, "after drawing");
    let shapes_len_before = doc_before.shapes.len();

    simulate_key(visual_cx, "escape", Modifiers::none());

    let anchor0 = draw_shape
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |anchor| anchor.pos);
    let select_point = select_point_for_anchor0(&bounds, anchor0, p1);

    click_left(visual_cx, select_point);
    visual_cx.run_until_parked();

    let selection_after_select = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        !selection_after_select.items.is_empty(),
        "expected selection to be non-empty; got selection={selection_after_select:?}"
    );
    let selected_snapshot = selection_after_select.clone();

    let clear_x = (width - 12.0).max(1.0);
    let clear_y = (height - 12.0).max(1.0);
    let clear_point = point(bounds.origin.x + px(clear_x), bounds.origin.y + px(clear_y));
    click_left(visual_cx, clear_point);
    visual_cx.run_until_parked();

    let selection_after_clear = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_clear.items.is_empty(),
        "expected selection to be cleared; got selection={selection_after_clear:?}"
    );

    simulate_key(visual_cx, "z", shift_secondary(Modifiers::secondary_key()));
    let selection_after_undo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_undo, selected_snapshot,
        "expected Shift+Undo to restore the selection"
    );

    simulate_key(visual_cx, "y", shift_secondary(Modifiers::secondary_key()));
    let selection_after_redo = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_after_redo, selection_after_clear,
        "expected Shift+Redo to reapply the selection clear"
    );

    let doc_after = visual_cx.read(|app| view.read(app).document().clone());
    assert_eq!(
        doc_after.shapes.len(),
        shapes_len_before,
        "selection undo/redo should not change the document"
    );
}
