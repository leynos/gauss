//! GPUI headless integration tests for manipulate-mode multi-shape dragging.
//!
//! When multiple shapes are selected, dragging any selected shape should:
//!
//! - keep the selection intact, and
//! - translate all selected shapes by the same delta.

mod common;

use common::{canvas_bounds, ensure_initial_draw, init_test_app, read_document};
use gauss::model::{Document, PaintStyle, Rgba, SegmentKind, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, px};
use uuid::Uuid;

fn find_shape<'a>(doc: &'a Document, id: ShapeId, context: &str) -> &'a Shape {
    doc.shapes
        .iter()
        .find(|shape| shape.id == id)
        .unwrap_or_else(|| panic!("expected shape {id:?} to exist: {context}"))
}

fn shape_bbox_centre(shape: &Shape) -> Vec2 {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for anchor in &shape.path.anchors {
        min_x = min_x.min(anchor.pos.x);
        min_y = min_y.min(anchor.pos.y);
        max_x = max_x.max(anchor.pos.x);
        max_y = max_y.max(anchor.pos.y);
    }
    Vec2::new(f32::midpoint(min_x, max_x), f32::midpoint(min_y, max_y))
}

const fn viewport_to_screen_point(
    viewport: gauss::model::Viewport,
    world: Vec2,
) -> gpui::Point<gpui::Pixels> {
    let screen = viewport.world_to_screen(world);
    gpui::point(px(screen.x), px(screen.y))
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

fn add_square(doc: &mut Document, id: ShapeId, min: Vec2, max: Vec2) {
    doc.shapes.push(Shape {
        id,
        z: i32::try_from(doc.shapes.len())
            .unwrap_or_else(|_| panic!("expected shape count to fit in i32 for z-ordering")),
        style: PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 2.0, None),
        path: gauss::model::PathGeom {
            anchors: vec![
                gauss::model::Anchor::new(min),
                gauss::model::Anchor::new(Vec2::new(max.x, min.y)),
                gauss::model::Anchor::new(max),
                gauss::model::Anchor::new(Vec2::new(min.x, max.y)),
            ],
            segments: vec![SegmentKind::Line, SegmentKind::Line, SegmentKind::Line],
            closed: true,
            closing_segment: SegmentKind::Line,
        },
    });
}

#[derive(Clone, Copy, Debug)]
struct ShapePair {
    first: ShapeId,
    second: ShapeId,
}

fn arrange_multi_shape_selection(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    origin: Vec2,
    shapes: ShapePair,
) {
    let min1 = origin.add(Vec2::new(10.0, 10.0));
    let max1 = origin.add(Vec2::new(110.0, 110.0));
    let min2 = origin.add(Vec2::new(160.0, 10.0));
    let max2 = origin.add(Vec2::new(260.0, 110.0));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            let mut doc = shell.document().clone();
            add_square(&mut doc, shapes.first, min1, max1);
            add_square(&mut doc, shapes.second, min2, max2);
            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(doc);
            shell.replace_selection_for_tests(gauss::model::Selection {
                items: vec![SelItem::Shape(shapes.first), SelItem::Shape(shapes.second)],
            });
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();
}

fn assert_selection_contains_shapes(
    visual_cx: &VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    expected_shape_ids: [ShapeId; 2],
    context: &str,
) {
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection.items.len(),
        2,
        "expected selection to remain a 2-shape multi-select ({context}); selection={selection:?}"
    );

    let [shape1_id, shape2_id] = expected_shape_ids;
    assert!(
        selection.contains(&SelItem::Shape(shape1_id))
            && selection.contains(&SelItem::Shape(shape2_id)),
        "expected selection to contain both shapes ({context}); selection={selection:?}"
    );
}

#[gpui::test]
fn dragging_a_selected_shape_moves_all_selected_shapes_and_preserves_selection(
    cx: &mut TestAppContext,
) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx);
    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));

    // Arrange: add two extra shapes so we can multi-select and move them.
    let shape1_id = ShapeId::from(Uuid::from_u128(0x1111_1111_1111_1111_1111_1111_1111_1111));
    let shape2_id = ShapeId::from(Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222));
    arrange_multi_shape_selection(
        visual_cx,
        &view,
        origin,
        ShapePair {
            first: shape1_id,
            second: shape2_id,
        },
    );

    let viewport = visual_cx.read(|app| view.read(app).viewport());

    let doc_before = read_document(visual_cx, &view);
    let shape1_before = find_shape(&doc_before, shape1_id, "before drag shape1").clone();
    let shape2_before = find_shape(&doc_before, shape2_id, "before drag shape2").clone();

    let start_model = shape_bbox_centre(&shape1_before);
    let delta = Vec2::new(20.0, 10.0);

    let start_screen = viewport_to_screen_point(viewport, start_model);
    let end_screen = viewport_to_screen_point(viewport, start_model.add(delta));

    // Act: drag shape1 with no modifiers.
    visual_cx.simulate_mouse_down(start_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    // Assert: selection remains intact on mouse down (no collapse to a single shape).
    assert_selection_contains_shapes(visual_cx, &view, [shape1_id, shape2_id], "after mouse down");

    visual_cx.simulate_mouse_move(end_screen, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(end_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    // Assert: both shapes translate by the same delta and selection stays intact.
    let doc_after = read_document(visual_cx, &view);
    let shape1_after = find_shape(&doc_after, shape1_id, "after drag shape1");
    let shape2_after = find_shape(&doc_after, shape2_id, "after drag shape2");

    assert_shape_translated_by_delta(shape1_after, &shape1_before, delta, "shape1 after drag");
    assert_shape_translated_by_delta(shape2_after, &shape2_before, delta, "shape2 after drag");

    assert_selection_contains_shapes(visual_cx, &view, [shape1_id, shape2_id], "after drag");

    // Keep this test resilient: if we accidentally fall back to draw mode, fail
    // loudly rather than flaking.
    let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
    assert!(!is_dragging, "expected drag gesture to have ended");
}
