//! GPUI headless integration tests for Phase 0 handle dragging.

use gauss::model::{Anchor, Document, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    KeyDownEvent, Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
struct CanvasScenario {
    bounds: gpui::Bounds<gpui::Pixels>,
    first: gpui::Point<gpui::Pixels>,
    second: gpui::Point<gpui::Pixels>,
    delta: Vec2,
}

fn canvas_scenario(visual_cx: &mut VisualTestContext) -> CanvasScenario {
    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds after drawing");
    };

    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    let first = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let second = point(
        bounds.origin.x + px((width - 2.0).max(2.0)),
        bounds.origin.y + px((height - 2.0).max(2.0)),
    );

    let max_horizontal_delta = (width - 4.0).max(1.0);
    let max_vertical_delta = (height - 4.0).max(1.0);
    let horizontal_delta = max_horizontal_delta.min(18.0);
    let vertical_delta = max_vertical_delta.min(10.0);

    CanvasScenario {
        bounds,
        first,
        second,
        delta: Vec2::new(horizontal_delta, vertical_delta),
    }
}

fn demo_shape_id() -> ShapeId {
    ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a))
}

fn find_draw_shape(doc: &Document) -> Option<&Shape> {
    let demo_id = demo_shape_id();
    doc.shapes.iter().find(|shape| shape.id != demo_id)
}

fn require_draw_shape<'a>(doc: &'a Document, context: &str) -> &'a Shape {
    let Some(shape) = find_draw_shape(doc) else {
        panic!("expected draw shape to exist: {context}");
    };
    shape
}

fn read_document(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> Document {
    visual_cx.read(|app| view.read(app).document().clone())
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
}

fn simulate_escape(visual_cx: &mut VisualTestContext) {
    simulate_key(visual_cx, "escape", Modifiers::none());
    visual_cx.run_until_parked();
}

fn simulate_document_undo(visual_cx: &mut VisualTestContext) {
    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    visual_cx.run_until_parked();
}

fn toggle_bezier_auto(visual_cx: &mut VisualTestContext) {
    simulate_key(visual_cx, "tab", Modifiers::none());
    visual_cx.run_until_parked();
}

fn draw_two_point_bezier_path(visual_cx: &mut VisualTestContext, scenario: CanvasScenario) {
    toggle_bezier_auto(visual_cx);
    visual_cx.simulate_mouse_move(scenario.first, None, Modifiers::none());
    visual_cx.simulate_click(scenario.first, Modifiers::none());
    visual_cx.simulate_mouse_move(scenario.second, None, Modifiers::none());
    visual_cx.simulate_click(scenario.second, Modifiers::none());
    visual_cx.run_until_parked();
}

fn assert_vec2_close(actual: Vec2, expected: Vec2, context: &str) {
    let diff = actual.sub(expected);
    assert!(
        diff.distance_squared(Vec2::ZERO) <= 0.0001,
        "{context}: expected={expected:?} got={actual:?}"
    );
}

fn anchor0_is_local_coordinates(anchor0: &Anchor, click_point: gpui::Point<gpui::Pixels>) -> bool {
    let expected_local = Vec2::new(2.0, 2.0);
    let expected_abs = Vec2::new(f32::from(click_point.x), f32::from(click_point.y));
    anchor0.pos.distance_squared(expected_local) <= anchor0.pos.distance_squared(expected_abs)
}

fn model_point_to_canvas_point(
    bounds: gpui::Bounds<gpui::Pixels>,
    use_local_coordinates: bool,
    model: Vec2,
) -> gpui::Point<gpui::Pixels> {
    if use_local_coordinates {
        point(bounds.origin.x + px(model.x), bounds.origin.y + px(model.y))
    } else {
        point(px(model.x), px(model.y))
    }
}

#[gpui::test]
fn dragging_handle_moves_it_and_undo_restores(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let scenario = canvas_scenario(visual_cx);
    draw_two_point_bezier_path(visual_cx, scenario);

    let doc_before = read_document(visual_cx, &view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points").clone();
    let Some(first_anchor) = original_shape.path.anchors.first().cloned() else {
        panic!("expected first anchor");
    };
    let Some(original_handle_out) = first_anchor.handle_out else {
        panic!("expected handle_out on first anchor in bezier mode");
    };

    let use_local_coordinates = anchor0_is_local_coordinates(&first_anchor, scenario.first);
    let handle_start =
        model_point_to_canvas_point(scenario.bounds, use_local_coordinates, original_handle_out);
    let handle_end = point(
        handle_start.x + px(scenario.delta.x),
        handle_start.y + px(scenario.delta.y),
    );

    simulate_escape(visual_cx);

    visual_cx.simulate_mouse_down(handle_start, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let expected_selection = SelItem::HandleOut {
        shape: original_shape.id,
        anchor: 0,
    };
    let did_select = visual_cx.read(|app| view.read(app).selection().contains(&expected_selection));
    assert!(did_select, "expected mouse down to select the handle");

    visual_cx.simulate_mouse_move(handle_end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(handle_end, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let doc_after_drag = read_document(visual_cx, &view);
    let moved_shape = require_draw_shape(&doc_after_drag, "after dragging handle");
    let Some(moved_anchor) = moved_shape.path.anchors.first() else {
        panic!("expected first anchor after drag");
    };

    assert_vec2_close(
        moved_anchor.pos,
        first_anchor.pos,
        "anchor position stable when dragging handle",
    );
    let Some(moved_handle_out) = moved_anchor.handle_out else {
        panic!("expected handle_out to remain present after dragging");
    };
    assert_vec2_close(
        moved_handle_out,
        original_handle_out.add(scenario.delta),
        "handle_out moved by delta",
    );

    simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let restored_shape = require_draw_shape(&doc_after_undo, "after undo");
    let Some(restored_anchor) = restored_shape.path.anchors.first() else {
        panic!("expected first anchor after undo");
    };
    let Some(restored_handle_out) = restored_anchor.handle_out else {
        panic!("expected handle_out after undo");
    };

    assert_vec2_close(restored_anchor.pos, first_anchor.pos, "anchor restored");
    assert_vec2_close(
        restored_handle_out,
        original_handle_out,
        "handle_out restored",
    );
}
