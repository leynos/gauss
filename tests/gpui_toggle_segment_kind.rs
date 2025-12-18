//! GPUI headless integration tests for Phase 0 segment toggling.

use gauss::model::{Document, SegmentKind, SelItem, Shape, ShapeId, Vec2};
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

fn canvas_points(
    bounds: &gpui::Bounds<gpui::Pixels>,
) -> (gpui::Point<gpui::Pixels>, gpui::Point<gpui::Pixels>) {
    let first = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let second = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );
    (first, second)
}

fn anchor0_is_local(anchor0: Vec2, click_point: gpui::Point<gpui::Pixels>) -> bool {
    let expected_local = Vec2::new(2.0, 2.0);
    let expected_abs = Vec2::new(f32::from(click_point.x), f32::from(click_point.y));
    anchor0.distance_squared(expected_local) <= anchor0.distance_squared(expected_abs)
}

fn model_to_screen_point(
    bounds: &gpui::Bounds<gpui::Pixels>,
    use_local: bool,
    model: Vec2,
) -> gpui::Point<gpui::Pixels> {
    if use_local {
        point(bounds.origin.x + px(model.x), bounds.origin.y + px(model.y))
    } else {
        point(px(model.x), px(model.y))
    }
}

fn assert_vec2_close(actual: Vec2, expected: Vec2, context: &str) {
    let diff = actual.sub(expected);
    assert!(
        diff.distance_squared(Vec2::ZERO) <= 0.0001,
        "{context}: expected={expected:?} got={actual:?}"
    );
}

fn select_segment0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    select_point: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) {
    visual_cx.simulate_mouse_down(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    let expected_segment = SelItem::Segment {
        shape: shape_id,
        seg: 0,
    };
    assert!(
        selection.contains(&expected_segment),
        "expected segment selection; selection={selection:?}"
    );

    visual_cx.simulate_mouse_up(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
}

fn assert_segment0_is_cubic_with_initial_handles(shape: &Shape, start_pos: Vec2, end_pos: Vec2) {
    let Some(kind) = shape.path.segments.first().copied() else {
        panic!("expected segment after toggle");
    };
    assert_eq!(kind, SegmentKind::Cubic, "expected segment to become cubic");

    let Some(start_anchor) = shape.path.anchors.first() else {
        panic!("expected first anchor after toggle");
    };
    let Some(end_anchor) = shape.path.anchors.get(1) else {
        panic!("expected second anchor after toggle");
    };

    let Some(handle_out) = start_anchor.handle_out else {
        panic!("expected handle_out to be set after line->cubic toggle");
    };
    let Some(handle_in) = end_anchor.handle_in else {
        panic!("expected handle_in to be set after line->cubic toggle");
    };

    let delta = end_pos.sub(start_pos);
    let third = delta.mul(1.0 / 3.0);
    assert_vec2_close(
        handle_out,
        start_pos.add(third),
        "handle_out should be one third along the line",
    );
    assert_vec2_close(
        handle_in,
        end_pos.sub(third),
        "handle_in should be two thirds along the line",
    );
}

fn assert_segment0_restores_line_without_handles(shape: &Shape) {
    let Some(kind) = shape.path.segments.first().copied() else {
        panic!("expected segment after undo");
    };
    assert_eq!(kind, SegmentKind::Line, "expected undo to restore line");

    let Some(start_anchor) = shape.path.anchors.first() else {
        panic!("expected first anchor after undo");
    };
    let Some(end_anchor) = shape.path.anchors.get(1) else {
        panic!("expected second anchor after undo");
    };

    assert!(
        start_anchor.handle_out.is_none(),
        "expected handle_out cleared on undo"
    );
    assert!(
        end_anchor.handle_in.is_none(),
        "expected handle_in cleared on undo"
    );
}

#[gpui::test]
fn tab_toggles_selected_segment_kind_and_undo_restores(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };
    let (p1, p2) = canvas_points(&bounds);

    click_canvas(visual_cx, p1);
    click_canvas(visual_cx, p2);

    let doc_before = read_document(visual_cx, &view);
    let shape_before = require_draw_shape(&doc_before, "after drawing").clone();
    let Some(start_anchor) = shape_before.path.anchors.first() else {
        panic!("expected first anchor after drawing");
    };
    let Some(end_anchor) = shape_before.path.anchors.get(1) else {
        panic!("expected second anchor after drawing");
    };

    simulate_key(visual_cx, "escape", Modifiers::none());

    let use_local = anchor0_is_local(start_anchor.pos, p1);
    let midpoint = Vec2::new(
        f32::midpoint(start_anchor.pos.x, end_anchor.pos.x),
        f32::midpoint(start_anchor.pos.y, end_anchor.pos.y),
    );
    let select_point = model_to_screen_point(&bounds, use_local, midpoint);

    select_segment0(visual_cx, &view, select_point, shape_before.id);

    simulate_key(visual_cx, "tab", Modifiers::none());

    let doc_after_toggle = read_document(visual_cx, &view);
    let shape_after_toggle = require_draw_shape(&doc_after_toggle, "after toggle");
    assert_segment0_is_cubic_with_initial_handles(
        shape_after_toggle,
        start_anchor.pos,
        end_anchor.pos,
    );

    simulate_key(visual_cx, "z", Modifiers::secondary_key());

    let doc_after_undo = read_document(visual_cx, &view);
    let shape_after_undo = require_draw_shape(&doc_after_undo, "after undo");
    assert_segment0_restores_line_without_handles(shape_after_undo);
}
