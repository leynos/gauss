//! GPUI headless integration tests for Phase 0 anchor insertion/deletion.

use gauss::model::{Document, SelItem, Shape, ShapeId, Vec2};
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

fn require_anchor_pos(shape: &Shape, index: usize, context: &str) -> Vec2 {
    shape.path.anchors.get(index).map_or_else(
        || panic!("missing anchor {index}: {context}"),
        |anchor| anchor.pos,
    )
}

fn require_anchor_len(shape: &Shape, expected: usize, context: &str) {
    assert_eq!(
        shape.path.anchors.len(),
        expected,
        "{context}: expected {expected} anchors, got {}",
        shape.path.anchors.len()
    );
}

fn require_segment_len(shape: &Shape, expected: usize, context: &str) {
    assert_eq!(
        shape.path.segments.len(),
        expected,
        "{context}: expected {expected} segments, got {}",
        shape.path.segments.len()
    );
}

fn require_path_counts(shape: &Shape, anchors: usize, segments: usize, context: &str) {
    require_anchor_len(shape, anchors, context);
    require_segment_len(shape, segments, context);
}

fn assert_vec2_close(actual: Vec2, expected: Vec2, context: &str) {
    let diff = actual.sub(expected);
    assert!(
        diff.distance_squared(Vec2::ZERO) <= 0.0001,
        "{context}: expected={expected:?} got={actual:?}"
    );
}

fn read_draw_shape(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    context: &str,
) -> Shape {
    let doc = read_document(visual_cx, view);
    require_draw_shape(&doc, context).clone()
}

#[gpui::test]
fn insert_and_delete_anchor_are_doc_undoable(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };
    let (p1, p2) = canvas_points(&bounds);
    click_canvas(visual_cx, p1);
    click_canvas(visual_cx, p2);

    let shape_before = read_draw_shape(visual_cx, &view, "after drawing");
    require_path_counts(&shape_before, 2, 1, "after drawing");

    let start_pos = require_anchor_pos(&shape_before, 0, "after drawing");
    let end_pos = require_anchor_pos(&shape_before, 1, "after drawing");

    simulate_key(visual_cx, "escape", Modifiers::none());

    let use_local = anchor0_is_local(start_pos, p1);
    let midpoint = Vec2::new(
        f32::midpoint(start_pos.x, end_pos.x),
        f32::midpoint(start_pos.y, end_pos.y),
    );
    let select_point = model_to_screen_point(&bounds, use_local, midpoint);
    select_segment0(visual_cx, &view, select_point, shape_before.id);

    simulate_key(visual_cx, "i", Modifiers::none());
    let shape_after_insert = read_draw_shape(visual_cx, &view, "after insert");
    require_path_counts(&shape_after_insert, 3, 2, "after insert");

    let inserted_pos = require_anchor_pos(&shape_after_insert, 1, "after insert");
    assert_vec2_close(
        inserted_pos,
        midpoint,
        "inserted anchor should be at midpoint",
    );

    let selection_after_insert = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_insert.contains(&SelItem::Anchor {
            shape: shape_before.id,
            anchor: 1
        }),
        "expected inserted anchor to become selected; selection={selection_after_insert:?}"
    );

    simulate_key(visual_cx, "backspace", Modifiers::none());
    let shape_after_delete = read_draw_shape(visual_cx, &view, "after delete");
    require_path_counts(&shape_after_delete, 2, 1, "after delete");

    let selection_after_delete = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_delete.items.is_empty(),
        "expected delete to clear selection; selection={selection_after_delete:?}"
    );

    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    let shape_after_undo_delete = read_draw_shape(visual_cx, &view, "after undo delete");
    require_path_counts(&shape_after_undo_delete, 3, 2, "after undo delete");

    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    let shape_after_undo_insert = read_draw_shape(visual_cx, &view, "after undo insert");
    require_path_counts(&shape_after_undo_insert, 2, 1, "after undo insert");

    simulate_key(visual_cx, "y", Modifiers::secondary_key());
    let shape_after_redo_insert = read_draw_shape(visual_cx, &view, "after redo insert");
    require_path_counts(&shape_after_redo_insert, 3, 2, "after redo insert");

    simulate_key(visual_cx, "y", Modifiers::secondary_key());
    let shape_after_redo_delete = read_draw_shape(visual_cx, &view, "after redo delete");
    require_path_counts(&shape_after_redo_delete, 2, 1, "after redo delete");
}
