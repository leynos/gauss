//! GPUI headless integration tests for Draw-mode `Escape` behaviour.
//!
//! In Phase 0, pressing `Escape` while drawing should:
//! - keep the current open path in the document, and
//! - switch to manipulate mode (so clicks no longer place points).

use gauss::model::{Document, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{KeyDownEvent, Keystroke, Modifiers, TestAppContext, VisualTestContext, point, px};
use uuid::Uuid;

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

fn require_draw_shape<'a>(doc: &'a Document, context: &str) -> &'a Shape {
    let demo_id = demo_shape_id();
    let Some(shape) = doc.shapes.iter().find(|shape| shape.id != demo_id) else {
        panic!("expected draw shape to exist: {context}");
    };
    shape
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

#[gpui::test]
fn escape_commits_open_path_and_enters_manipulate(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };

    let p1 = point(bounds.origin.x + px(10.0), bounds.origin.y + px(10.0));
    let p2 = point(bounds.origin.x + px(80.0), bounds.origin.y + px(30.0));

    click_canvas(visual_cx, p1);
    click_canvas(visual_cx, p2);

    let doc_before_escape = read_document(visual_cx, &view);
    let shape_before_escape = require_draw_shape(&doc_before_escape, "after drawing two points");
    assert!(
        !shape_before_escape.path.closed,
        "expected newly drawn path to be open before closing; shape={shape_before_escape:?}"
    );

    let anchor_count_before = shape_before_escape.path.anchors.len();
    let seg_count_before = shape_before_escape.path.segments.len();

    simulate_key(visual_cx, "escape", Modifiers::none());

    // Clicking after escape should not add points (we should be in manipulate
    // mode), and the open path should still exist in the document.
    click_canvas(visual_cx, p2);

    let doc_after = read_document(visual_cx, &view);
    let shape_after = require_draw_shape(&doc_after, "after escape and click");

    assert_eq!(
        shape_after.id, shape_before_escape.id,
        "expected escape to keep the same open path in the document"
    );
    assert!(
        !shape_after.path.closed,
        "expected escape to commit an open path without closing it"
    );
    assert_eq!(
        shape_after.path.anchors.len(),
        anchor_count_before,
        "expected manipulate-mode click after escape to not add anchors"
    );
    assert_eq!(
        shape_after.path.segments.len(),
        seg_count_before,
        "expected manipulate-mode click after escape to not add segments"
    );

    // Keep this test resilient to coordinate system changes by asserting that
    // the first two anchors are still distinct.
    let first_anchor = shape_after
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |anchor| anchor.pos);
    let second_anchor = shape_after
        .path
        .anchors
        .get(1)
        .map_or(Vec2::ZERO, |anchor| anchor.pos);
    assert_ne!(
        first_anchor, second_anchor,
        "expected at least two distinct anchors in the committed open path"
    );
}
