//! GPUI headless integration tests for Phase 0 draw-mode path closing.
//!
//! Closing a path is a key workflow boundary in Phase 0:
//!
//! - The user clicks near the first anchor (within a snap radius).
//! - The active path becomes closed (and gains a default fill if none was set).
//! - The editor switches to manipulate mode so subsequent clicks do not place
//!   more points.

use gauss::model::{Document, SegmentKind, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Bounds, Modifiers, Pixels, TestAppContext, VisualTestContext, point, px};
use uuid::Uuid;

fn demo_shape_id() -> ShapeId {
    ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a))
}

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

fn canvas_bounds(visual_cx: &mut VisualTestContext) -> Bounds<Pixels> {
    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };
    bounds
}

fn require_draw_shape<'a>(doc: &'a Document, context: &'static str) -> &'a Shape {
    let demo_id = demo_shape_id();
    let Some(shape) = doc.shapes.iter().find(|shape| shape.id != demo_id) else {
        panic!("expected draw shape to exist: {context}");
    };
    shape
}

fn draw_point(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
    visual_cx.run_until_parked();
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

fn triangle_points(
    bounds: &Bounds<Pixels>,
) -> (
    gpui::Point<Pixels>,
    gpui::Point<Pixels>,
    gpui::Point<Pixels>,
) {
    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + px(12.0),
    );
    let p3 = point(
        bounds.origin.x + bounds.size.width - px(12.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );
    (p1, p2, p3)
}

fn assert_open_triangle(doc: &Document) -> ShapeId {
    assert_eq!(
        doc.shapes.len(),
        2,
        "expected demo + one draw shape before close"
    );

    let shape = require_draw_shape(doc, "before close");
    assert!(!shape.path.closed, "expected path to be open before close");
    assert_eq!(
        shape.path.anchors.len(),
        3,
        "expected three anchors before close"
    );
    shape.id
}

fn close_path_by_clicking_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    bounds: &Bounds<Pixels>,
    click_point: gpui::Point<Pixels>,
) -> ShapeId {
    let doc = visual_cx.read(|app| view.read(app).document().clone());
    let shape = require_draw_shape(&doc, "before close");
    let shape_id = shape.id;

    let Some(first_anchor) = shape.path.anchors.first() else {
        panic!("expected first anchor to exist before close");
    };

    let use_local = anchor0_is_local(first_anchor.pos, click_point);
    let close_point = model_to_screen_point(bounds, use_local, first_anchor.pos);
    draw_point(visual_cx, close_point);

    shape_id
}

fn assert_closed_shape(doc: &Document, expected_shape_id: ShapeId) {
    assert_eq!(
        doc.shapes.len(),
        2,
        "closing the path should not create a new shape"
    );

    let shape = require_draw_shape(doc, "after close");
    assert_eq!(
        shape.id, expected_shape_id,
        "expected close to preserve the shape id"
    );
    assert!(shape.path.closed, "expected path to become closed");
    assert!(
        shape.style.fill.is_some(),
        "expected a default fill to be applied on close"
    );
    assert_eq!(
        shape.path.anchors.len(),
        3,
        "expected closing the path not to add anchors"
    );
}

fn assert_click_does_not_place_points(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    click_point: gpui::Point<Pixels>,
) {
    draw_point(visual_cx, click_point);

    let doc_after_click = visual_cx.read(|app| view.read(app).document().clone());
    assert_eq!(
        doc_after_click.shapes.len(),
        2,
        "after closing, additional clicks should not place draw points"
    );
    let shape_after_click = require_draw_shape(&doc_after_click, "after post-close click");
    assert_eq!(
        shape_after_click.path.anchors.len(),
        3,
        "after closing, additional clicks should not mutate the closed path"
    );
}

#[gpui::test]
fn clicking_near_first_anchor_closes_path_and_enters_manipulate(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx);
    let (p1, p2, p3) = triangle_points(&bounds);

    draw_point(visual_cx, p1);
    draw_point(visual_cx, p2);
    draw_point(visual_cx, p3);

    let doc_before_close = visual_cx.read(|app| view.read(app).document().clone());
    let expected_shape_id = assert_open_triangle(&doc_before_close);
    let closed_shape_id = close_path_by_clicking_first_anchor(visual_cx, &view, &bounds, p1);
    assert_eq!(
        closed_shape_id, expected_shape_id,
        "expected close operation to target the drawn shape"
    );

    let doc_after_close = visual_cx.read(|app| view.read(app).document().clone());
    assert_closed_shape(&doc_after_close, expected_shape_id);

    let after_close_click = point(bounds.origin.x + px(20.0), bounds.origin.y + px(20.0));
    assert_click_does_not_place_points(visual_cx, &view, after_close_click);
}

#[gpui::test]
fn closing_in_bezier_mode_uses_cubic_closing_segment(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx);
    let (p1, p2, p3) = triangle_points(&bounds);

    draw_point(visual_cx, p1);
    visual_cx.simulate_keystrokes("tab");
    visual_cx.run_until_parked();

    draw_point(visual_cx, p2);
    draw_point(visual_cx, p3);

    let closed_shape_id = close_path_by_clicking_first_anchor(visual_cx, &view, &bounds, p1);

    let doc_after_close = visual_cx.read(|app| view.read(app).document().clone());
    let shape = require_draw_shape(&doc_after_close, "after bezier close");
    assert_eq!(
        shape.id, closed_shape_id,
        "expected close to preserve the shape id"
    );
    assert!(
        shape.path.closed,
        "expected closing in bezier mode to still close the path"
    );
    assert_eq!(
        shape.path.closing_segment,
        SegmentKind::Cubic,
        "expected bezier mode to use a cubic closing segment"
    );
    assert!(
        shape
            .path
            .anchors
            .first()
            .and_then(|a| a.handle_in)
            .is_some(),
        "expected closing cubic segment to set handle_in on the first anchor"
    );
    assert!(
        shape
            .path
            .anchors
            .last()
            .and_then(|a| a.handle_out)
            .is_some(),
        "expected closing cubic segment to set handle_out on the last anchor"
    );
}
