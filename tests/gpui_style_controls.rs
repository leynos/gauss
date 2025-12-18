//! GPUI headless integration tests for Phase 0 stroke/fill controls.

use gauss::model::{Document, Rgba, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    Hsla, KeyDownEvent, Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext,
    point, px,
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

fn read_document(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> Document {
    visual_cx.read(|app| view.read(app).document().clone())
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

fn click_canvas(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
    visual_cx.run_until_parked();
}

fn anchor0_is_local(anchor0: Vec2, click_point: gpui::Point<gpui::Pixels>) -> bool {
    let expected_local = Vec2::new(2.0, 2.0);
    let expected_abs = Vec2::new(f32::from(click_point.x), f32::from(click_point.y));
    anchor0.distance_squared(expected_local) <= anchor0.distance_squared(expected_abs)
}

fn anchor0_select_point(
    bounds: &gpui::Bounds<gpui::Pixels>,
    anchor0: Vec2,
    click_point: gpui::Point<gpui::Pixels>,
) -> gpui::Point<gpui::Pixels> {
    if anchor0_is_local(anchor0, click_point) {
        point(
            bounds.origin.x + px(anchor0.x),
            bounds.origin.y + px(anchor0.y),
        )
    } else {
        point(px(anchor0.x), px(anchor0.y))
    }
}

fn select_anchor0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    select_point: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) {
    visual_cx.simulate_mouse_down(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    let expected_anchor = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    assert!(
        selection.contains(&expected_anchor),
        "expected anchor selection; selection={selection:?}"
    );

    visual_cx.simulate_mouse_up(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
}

#[gpui::test]
fn style_changes_apply_to_selected_shapes_and_are_undoable(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };
    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );

    click_canvas(visual_cx, p1);
    click_canvas(visual_cx, p2);

    let doc_before = read_document(visual_cx, &view);
    let shape_before = require_draw_shape(&doc_before, "after drawing").clone();
    let anchor0 = shape_before
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |anchor| anchor.pos);

    simulate_key(visual_cx, "escape", Modifiers::none());

    let select_point = anchor0_select_point(&bounds, anchor0, p1);
    select_anchor0(visual_cx, &view, select_point, shape_before.id);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.apply_stroke_colour(Some(Hsla::red()));
            shell.apply_fill_colour(Some(Hsla::blue()));
        });
    });
    visual_cx.run_until_parked();

    let doc_after = read_document(visual_cx, &view);
    let shape_after = require_draw_shape(&doc_after, "after applying style");
    assert_eq!(
        shape_after.style.stroke,
        Some(Rgba::new(255, 0, 0, 255)),
        "expected stroke to be updated to red"
    );
    assert_eq!(
        shape_after.style.fill,
        Some(Rgba::new(0, 0, 255, 255)),
        "expected fill to be updated to blue"
    );

    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    simulate_key(visual_cx, "z", Modifiers::secondary_key());

    let doc_after_undo = read_document(visual_cx, &view);
    let shape_after_undo = require_draw_shape(&doc_after_undo, "after undo");
    assert_eq!(
        shape_after_undo.style, shape_before.style,
        "expected undo to restore the original style"
    );
}
