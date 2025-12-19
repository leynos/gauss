//! GPUI headless integration tests for bounding-box dragging rules.
//!
//! Phase 0 uses a loose bounding-box hit test as the fallback for selecting a
//! shape in manipulate mode. However, dragging by the bounding box is only
//! allowed once the shape is already selected.
//!
//! This keeps "select" and "move" as separate gestures for bbox hits, while
//! still allowing explicit drags when the user has a selected shape (or a
//! multi-selection).

use gauss::model::{Document, PaintStyle, Rgba, SegmentKind, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, px};
use uuid::Uuid;

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

fn canvas_bounds(visual_cx: &mut VisualTestContext) -> gpui::Bounds<gpui::Pixels> {
    visual_cx
        .debug_bounds("#phase0-canvas")
        .unwrap_or_else(|| panic!("phase0 canvas should have debug bounds"))
}

fn add_square(doc: &mut Document, id: ShapeId, min: Vec2, max: Vec2) {
    doc.shapes.push(Shape {
        id,
        z: i32::try_from(doc.shapes.len()).unwrap_or(i32::MAX),
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

fn require_shape<'a>(doc: &'a Document, id: ShapeId, context: &str) -> &'a Shape {
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

fn assert_shape_unchanged(shape: &Shape, original: &Shape, context: &str) {
    assert_eq!(
        shape.path.anchors.len(),
        original.path.anchors.len(),
        "anchor count mismatch: {context}"
    );

    for (current, start) in shape.path.anchors.iter().zip(original.path.anchors.iter()) {
        let diff = current.pos.sub(start.pos);
        assert!(
            diff.distance_squared(Vec2::ZERO) <= 0.0001,
            "expected shape to remain unchanged ({context}); start={:?} got={:?}",
            start.pos,
            current.pos
        );
    }
}

#[gpui::test]
fn bbox_dragging_requires_shape_to_be_preselected(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx);
    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));

    let shape_id = ShapeId::from(Uuid::from_u128(0x4444_4444_4444_4444_4444_4444_4444_4444));
    let min = origin.add(Vec2::new(10.0, 10.0));
    let max = origin.add(Vec2::new(110.0, 110.0));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            let mut doc = shell.document().clone();
            add_square(&mut doc, shape_id, min, max);
            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(doc);
            shell.replace_selection_for_tests(gauss::model::Selection::empty());
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let viewport = visual_cx.read(|app| view.read(app).viewport());

    let doc_before = visual_cx.read(|app| view.read(app).document().clone());
    let shape_before = require_shape(&doc_before, shape_id, "before drag").clone();
    let start_world = shape_bbox_centre(&shape_before);
    let delta = Vec2::new(25.0, 15.0);

    let start_screen = viewport_to_screen_point(viewport, start_world);
    let end_screen = viewport_to_screen_point(viewport, start_world.add(delta));

    // Act: attempt to drag the shape via its bbox without it being selected
    // before mouse down.
    visual_cx.simulate_mouse_down(start_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    // Assert: the bbox hit should select the shape, but must not start a drag
    // gesture until the shape was already selected.
    let selection_after_down = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_after_down.contains(&SelItem::Shape(shape_id)),
        "expected bbox mouse down to select the shape; selection={selection_after_down:?}"
    );
    let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
    assert!(
        !is_dragging,
        "expected bbox mouse down not to start dragging when shape was not preselected"
    );

    visual_cx.simulate_mouse_move(end_screen, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(end_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let doc_after = visual_cx.read(|app| view.read(app).document().clone());
    let shape_after = require_shape(&doc_after, shape_id, "after attempted drag");
    assert_shape_unchanged(shape_after, &shape_before, "bbox drag without preselection");
}
