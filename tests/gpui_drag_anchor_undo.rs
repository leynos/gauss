//! GPUI headless integration tests for Phase 0 anchor dragging.

use gauss::model::{Anchor, Document, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{
    KeyDownEvent, Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
struct CanvasDragScenario {
    first: gpui::Point<gpui::Pixels>,
    second: gpui::Point<gpui::Pixels>,
    drag_end: gpui::Point<gpui::Pixels>,
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

    let horizontal_delta = max_horizontal_delta.min(24.0);
    let vertical_delta = max_vertical_delta.min(12.0);

    let drag_end = point(first.x + px(horizontal_delta), first.y + px(vertical_delta));

    CanvasDragScenario {
        first,
        second,
        drag_end,
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

fn assert_vec2_close(actual: Vec2, expected: Vec2, context: &str) {
    let diff = actual.sub(expected);
    assert!(
        diff.distance_squared(Vec2::ZERO) <= 0.0001,
        "{context}: expected={expected:?} got={actual:?}"
    );
}

fn draw_two_point_line_path(visual_cx: &mut VisualTestContext, scenario: CanvasDragScenario) {
    visual_cx.simulate_mouse_move(scenario.first, None, Modifiers::none());
    visual_cx.simulate_click(scenario.first, Modifiers::none());
    visual_cx.simulate_mouse_move(scenario.second, None, Modifiers::none());
    visual_cx.simulate_click(scenario.second, Modifiers::none());
    visual_cx.run_until_parked();
}

fn first_two_anchors(shape: &Shape) -> (Anchor, Anchor) {
    let Some(first) = shape.path.anchors.first().cloned() else {
        panic!("expected first anchor");
    };
    let Some(second) = shape.path.anchors.get(1).cloned() else {
        panic!("expected second anchor");
    };
    (first, second)
}

fn drag_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    shape_id: ShapeId,
    scenario: CanvasDragScenario,
) {
    visual_cx.simulate_mouse_down(scenario.first, MouseButton::Left, Modifiers::none());

    let selected_anchor = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection.contains(&SelItem::Shape(shape_id)),
        "expected anchor interaction to keep the shape selected; selection={selection:?}"
    );
    assert!(
        selection.contains(&selected_anchor),
        "expected mouse down to select the first anchor; selection={selection:?}"
    );

    visual_cx.simulate_mouse_move(scenario.drag_end, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(scenario.drag_end, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
}

#[gpui::test]
fn dragging_anchor_moves_it_and_undo_restores(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let scenario = canvas_drag_scenario(visual_cx);
    draw_two_point_line_path(visual_cx, scenario);

    let doc_before = read_document(visual_cx, &view);
    let original_shape = require_draw_shape(&doc_before, "after drawing two points").clone();
    let (original_first_anchor, original_second_anchor) = first_two_anchors(&original_shape);

    simulate_escape(visual_cx);
    drag_first_anchor(visual_cx, &view, original_shape.id, scenario);

    let doc_after_drag = read_document(visual_cx, &view);
    let moved_shape = require_draw_shape(&doc_after_drag, "after dragging anchor");
    let (moved_first_anchor, moved_second_anchor) = first_two_anchors(moved_shape);

    assert_vec2_close(
        moved_first_anchor.pos,
        original_first_anchor.pos.add(scenario.delta),
        "first anchor moved",
    );
    assert_vec2_close(
        moved_second_anchor.pos,
        original_second_anchor.pos,
        "second anchor stable",
    );

    if let Some(handle_out) = original_first_anchor.handle_out {
        let Some(moved_handle_out) = moved_first_anchor.handle_out else {
            panic!("expected moved handle_out to remain present");
        };
        assert_vec2_close(
            moved_handle_out,
            handle_out.add(scenario.delta),
            "first anchor handle_out moved",
        );
    }

    simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let restored_shape = require_draw_shape(&doc_after_undo, "after undo");
    let (restored_first_anchor, restored_second_anchor) = first_two_anchors(restored_shape);

    assert_vec2_close(
        restored_first_anchor.pos,
        original_first_anchor.pos,
        "first anchor restored",
    );
    assert_vec2_close(
        restored_second_anchor.pos,
        original_second_anchor.pos,
        "second anchor still stable after undo",
    );
}
