//! GPUI headless integration tests for Phase 0 manipulate-mode interactions.

use gauss::model::{Document, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    KeyDownEvent, Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
struct CanvasDragScenario {
    first: gpui::Point<gpui::Pixels>,
    second: gpui::Point<gpui::Pixels>,
    delta: Vec2,
}

fn canvas_drag_scenario(visual_cx: &mut VisualTestContext) -> CanvasDragScenario {
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

    let horizontal_delta = max_horizontal_delta.min(20.0);
    let vertical_delta = max_vertical_delta.min(10.0);

    CanvasDragScenario {
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

fn simulate_document_undo(visual_cx: &mut VisualTestContext) {
    let undo = KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers::secondary_key(),
            key: "z".to_owned(),
            key_char: None,
        },
        is_held: false,
    };

    visual_cx.simulate_event(undo);
}

fn simulate_escape(visual_cx: &mut VisualTestContext) {
    let escape = KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers::none(),
            key: "escape".to_owned(),
            key_char: None,
        },
        is_held: false,
    };

    visual_cx.simulate_event(escape);
}

fn assert_shape_translated_by_delta(shape: &Shape, original: &Shape, delta: Vec2, context: &str) {
    assert_eq!(
        shape.path.anchors.len(),
        original.path.anchors.len(),
        "anchor count mismatch: {context}"
    );

    for (current, start) in shape.path.anchors.iter().zip(original.path.anchors.iter()) {
        let expected = start.pos.add(delta);
        let diff = current.pos.sub(expected);
        assert!(
            diff.distance_squared(Vec2::ZERO) <= 0.0001,
            "anchor did not move by expected delta: {context}; start={:?} expected={:?} got={:?} delta={:?}",
            start.pos,
            expected,
            current.pos,
            delta
        );
    }
}

#[gpui::test]
fn dragging_demo_shape_moves_it_and_undo_restores(cx: &mut TestAppContext) {
    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();

    // Create a new shape in draw mode at deterministic in-canvas coordinates so
    // the hit-testing and drag logic can operate on it.
    let scenario = canvas_drag_scenario(visual_cx);
    visual_cx.simulate_mouse_move(scenario.first, None, Modifiers::none());
    visual_cx.simulate_click(scenario.first, Modifiers::none());
    visual_cx.simulate_mouse_move(scenario.second, None, Modifiers::none());
    visual_cx.simulate_click(scenario.second, Modifiers::none());
    visual_cx.run_until_parked();

    let doc_before = read_document(visual_cx, &view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points").clone();

    // Switch to manipulate mode (Phase0Shell defaults to draw mode).
    simulate_escape(visual_cx);
    visual_cx.run_until_parked();

    let shapes_after_escape = read_document(visual_cx, &view).shapes.len();
    visual_cx.simulate_mouse_move(scenario.first, None, Modifiers::none());
    visual_cx.simulate_click(scenario.first, Modifiers::none());
    visual_cx.run_until_parked();
    let shapes_after_escape_click = read_document(visual_cx, &view).shapes.len();
    assert_eq!(
        shapes_after_escape_click, shapes_after_escape,
        "escape should switch to manipulate mode, where clicks do not add points"
    );

    let drag_start = {
        let start_x = f32::midpoint(f32::from(scenario.first.x), f32::from(scenario.second.x));
        let start_y = f32::midpoint(f32::from(scenario.first.y), f32::from(scenario.second.y));
        point(px(start_x), px(start_y))
    };
    let drag_end = point(
        drag_start.x + px(scenario.delta.x),
        drag_start.y + px(scenario.delta.y),
    );

    visual_cx.simulate_mouse_down(drag_start, MouseButton::Left, Modifiers::none());
    let selection_after_down = visual_cx.read(|app| view.read(app).selection().clone());
    let did_select = selection_after_down.items.iter().any(|item| match item {
        SelItem::Shape(id) => *id == original_shape.id,
        SelItem::Segment { shape, .. } => *shape == original_shape.id,
        _ => false,
    });
    assert!(
        did_select,
        "expected mouse down to select the draw shape; selection={selection_after_down:?}"
    );

    let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
    assert!(is_dragging, "expected mouse down to start a drag gesture");

    visual_cx.simulate_mouse_move(drag_end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(drag_end, MouseButton::Left, Modifiers::none());

    let stopped_dragging = visual_cx.read(|app| !view.read(app).is_dragging());
    assert!(
        stopped_dragging,
        "expected mouse up to end the active drag gesture"
    );

    let doc_after_drag = read_document(visual_cx, &view);
    let moved_shape = require_draw_shape(&doc_after_drag, "after dragging draw shape");
    assert_shape_translated_by_delta(moved_shape, &original_shape, scenario.delta, "after drag");

    simulate_document_undo(visual_cx);
    let doc_after_undo = read_document(visual_cx, &view);
    let restored_shape = require_draw_shape(&doc_after_undo, "after undo");
    assert_shape_translated_by_delta(restored_shape, &original_shape, Vec2::ZERO, "after undo");
}
