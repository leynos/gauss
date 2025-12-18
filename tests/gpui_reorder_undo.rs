//! GPUI headless integration tests for Phase 0 z-order reordering.
//!
//! This test draws two identical (overlapping) shapes so that “raise” and
//! “lower” behaviour depends only on z-order, not on choosing distinct hit
//! targets.

use gauss::model::{Document, SelItem, Selection, ShapeId};
use gauss::ui::Phase0Shell;
use gpui::{
    KeyDownEvent, Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
struct LinePoints {
    start: gpui::Point<gpui::Pixels>,
    end: gpui::Point<gpui::Pixels>,
}

fn demo_shape_id() -> ShapeId {
    ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a))
}

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

fn read_document(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> Document {
    visual_cx.read(|app| view.read(app).document().clone())
}

fn read_selection(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> Selection {
    visual_cx.read(|app| view.read(app).selection().clone())
}

fn click_canvas(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
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

fn selected_shape_id(selection: &Selection) -> Option<ShapeId> {
    let item = selection.items.first()?;
    Some(match item {
        SelItem::Shape(shape)
        | SelItem::Anchor { shape, .. }
        | SelItem::HandleIn { shape, .. }
        | SelItem::HandleOut { shape, .. }
        | SelItem::Segment { shape, .. } => *shape,
    })
}

fn require_drawn_shape_ids(doc: &Document) -> (ShapeId, ShapeId) {
    let demo_id = demo_shape_id();
    let mut ids = doc
        .shapes
        .iter()
        .filter(|shape| shape.id != demo_id)
        .map(|shape| shape.id);

    let Some(first) = ids.next() else {
        panic!("expected a first drawn shape");
    };
    let Some(second) = ids.next() else {
        panic!("expected a second drawn shape");
    };
    assert!(ids.next().is_none(), "expected exactly two drawn shapes");
    (first, second)
}

fn require_shape_index(doc: &Document, shape_id: ShapeId, context: &str) -> usize {
    let Some(index) = doc.find_index(shape_id) else {
        panic!("expected {shape_id:?} to exist: {context}");
    };
    index
}

fn ordered_pair(doc: &Document, a: ShapeId, b: ShapeId, context: &str) -> (ShapeId, ShapeId) {
    let a_index = require_shape_index(doc, a, context);
    let b_index = require_shape_index(doc, b, context);
    if a_index <= b_index { (a, b) } else { (b, a) }
}

fn assert_relative_order(doc: &Document, lower: ShapeId, higher: ShapeId, context: &str) {
    let Some(lower_index) = doc.find_index(lower) else {
        panic!("expected {lower:?} to exist: {context}");
    };
    let Some(higher_index) = doc.find_index(higher) else {
        panic!("expected {higher:?} to exist: {context}");
    };
    assert!(
        lower_index < higher_index,
        "expected {lower:?} below {higher:?}: {context}"
    );
}

fn line_points(bounds: &gpui::Bounds<gpui::Pixels>) -> LinePoints {
    let start = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let end = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + px(2.0),
    );
    LinePoints { start, end }
}

fn draw_overlapping_lines(visual_cx: &mut VisualTestContext, points: LinePoints) {
    click_canvas(visual_cx, points.start);
    click_canvas(visual_cx, points.end);
    simulate_key(visual_cx, "escape", Modifiers::none());

    simulate_key(visual_cx, "escape", Modifiers::none());
    click_canvas(visual_cx, points.start);
    click_canvas(visual_cx, points.end);
    simulate_key(visual_cx, "escape", Modifiers::none());
}

#[gpui::test]
fn raise_lower_reorders_overlapping_shapes_with_undo(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };
    let points = line_points(&bounds);

    draw_overlapping_lines(visual_cx, points);

    let doc = read_document(visual_cx, &view);
    let (a, b) = require_drawn_shape_ids(&doc);
    let (lower, higher) = ordered_pair(&doc, a, b, "after drawing");
    assert_relative_order(&doc, lower, higher, "after drawing");

    visual_cx.simulate_mouse_down(points.start, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(points.start, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    let selection = read_selection(visual_cx, &view);
    let Some(selected) = selected_shape_id(&selection) else {
        panic!("expected selection to be non-empty after selecting");
    };
    assert_eq!(
        selected, higher,
        "expected overlapping click to select the top-most shape"
    );

    simulate_key(visual_cx, "[", Modifiers::secondary_key());
    let doc_after_lower = read_document(visual_cx, &view);
    assert_relative_order(
        &doc_after_lower,
        higher,
        lower,
        "after lowering top-most shape",
    );

    simulate_key(visual_cx, "]", Modifiers::secondary_key());
    let doc_after_raise = read_document(visual_cx, &view);
    assert_relative_order(&doc_after_raise, lower, higher, "after raising back to top");

    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    let doc_after_undo_raise = read_document(visual_cx, &view);
    assert_relative_order(&doc_after_undo_raise, higher, lower, "after undoing raise");

    simulate_key(visual_cx, "z", Modifiers::secondary_key());
    let doc_after_undo_lower = read_document(visual_cx, &view);
    assert_relative_order(&doc_after_undo_lower, lower, higher, "after undoing lower");
}
