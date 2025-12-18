//! GPUI headless integration tests for Phase 0 draw-mode interactions.

use gauss::model::ShapeId;
use gauss::ui::Phase0Shell;
use gpui::{KeyDownEvent, Keystroke, Modifiers, TestAppContext, VisualTestContext, point, px};
use uuid::Uuid;

fn demo_shape_id() -> ShapeId {
    ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a))
}

fn canvas_points(
    visual_cx: &mut gpui::VisualTestContext,
) -> (gpui::Point<gpui::Pixels>, gpui::Point<gpui::Pixels>) {
    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds after drawing");
    };
    let first = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let second = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );
    (first, second)
}

fn find_draw_shape(doc: &gauss::model::Document) -> Option<&gauss::model::Shape> {
    let demo_id = demo_shape_id();
    doc.shapes.iter().find(|shape| shape.id != demo_id)
}

fn require_draw_shape<'a>(
    doc: &'a gauss::model::Document,
    context: &str,
) -> &'a gauss::model::Shape {
    let Some(shape) = find_draw_shape(doc) else {
        panic!("expected draw shape to exist: {context}");
    };
    shape
}

fn read_document(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> gauss::model::Document {
    visual_cx.read(|app| view.read(app).document().clone())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExpectedDrawShapeState {
    total_shapes: usize,
    anchors: usize,
    segments: usize,
    closed: bool,
}

impl ExpectedDrawShapeState {
    const fn new(total_shapes: usize, anchors: usize, segments: usize, closed: bool) -> Self {
        Self {
            total_shapes,
            anchors,
            segments,
            closed,
        }
    }
}

fn assert_draw_shape_state(
    doc: &gauss::model::Document,
    expected: ExpectedDrawShapeState,
    context: &str,
) {
    assert_eq!(
        doc.shapes.len(),
        expected.total_shapes,
        "unexpected shape count: {context}"
    );

    let shape = require_draw_shape(doc, context);
    assert_eq!(
        shape.path.anchors.len(),
        expected.anchors,
        "unexpected anchor count: {context}"
    );
    assert_eq!(
        shape.path.segments.len(),
        expected.segments,
        "unexpected segment count: {context}"
    );
    assert_eq!(
        shape.path.closed, expected.closed,
        "unexpected closed state: {context}"
    );
}

fn assert_draw_shape_absent(
    doc: &gauss::model::Document,
    expected_total_shapes: usize,
    context: &str,
) {
    assert_eq!(
        doc.shapes.len(),
        expected_total_shapes,
        "unexpected shape count: {context}"
    );
    assert!(
        find_draw_shape(doc).is_none(),
        "draw shape should be absent: {context}"
    );
}

fn require_last_canvas_click(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    context: &str,
) -> gauss::model::Vec2 {
    let Some(last) = visual_cx.read(|app| view.read(app).last_canvas_click_screen()) else {
        panic!("expected Phase0Shell to observe a canvas click: {context}");
    };
    last
}

fn require_canvas_click_changed(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    previous: gauss::model::Vec2,
    context: &str,
) -> gauss::model::Vec2 {
    let next = require_last_canvas_click(visual_cx, view, context);
    assert_ne!(
        next, previous,
        "expected a distinct canvas click: {context}"
    );
    next
}

fn click_canvas(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
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

#[gpui::test]
fn draw_click_adds_points_and_undo_removes(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();

    let (pos1, pos2) = canvas_points(visual_cx);

    click_canvas(visual_cx, pos1);
    let last_click_after_first = require_last_canvas_click(visual_cx, &view, "after first click");

    let doc_after_first = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_first,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after first click",
    );

    click_canvas(visual_cx, pos2);
    let _last_click_after_second = require_canvas_click_changed(
        visual_cx,
        &view,
        last_click_after_first,
        "after second click",
    );

    let doc_after_second = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_second,
        ExpectedDrawShapeState::new(2, 2, 1, false),
        "after second click",
    );

    simulate_document_undo(visual_cx);
    let doc_after_undo = read_document(visual_cx, &view);
    assert_draw_shape_state(
        &doc_after_undo,
        ExpectedDrawShapeState::new(2, 1, 0, false),
        "after undoing second click",
    );

    simulate_document_undo(visual_cx);
    let doc_after_second_undo = read_document(visual_cx, &view);
    assert_draw_shape_absent(&doc_after_second_undo, 1, "after undoing the first click");
}
