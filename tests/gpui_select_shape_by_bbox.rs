//! GPUI headless integration test for shape selection by bounding box.
//!
//! Phase 0 uses a loose shape bounding-box hit-test as a final fallback after
//! handles, anchors, and segments. Clicking inside a shape’s bounding box (but
//! not near its edges) should select the shape.

use gauss::model::{PaintStyle, Rgba, SegmentKind, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use uuid::Uuid;

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

fn demo_square(id: ShapeId, min: Vec2, max: Vec2) -> Shape {
    Shape {
        id,
        z: 0,
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
    }
}

#[gpui::test]
fn clicking_inside_shape_bbox_selects_shape(cx: &mut TestAppContext) {
    cx.update(gauss::ui::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds after drawing");
    };

    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let shape_id = ShapeId::from(Uuid::from_u128(0x4444_4444_4444_4444_4444_4444_4444_4444));

    // Place a square far enough from the click point that segment hit-testing
    // should not trigger (we click the square's centre, not near edges).
    let min = origin.add(Vec2::new(20.0, 20.0));
    let max = origin.add(Vec2::new(160.0, 160.0));
    let centre = Vec2::new(f32::midpoint(min.x, max.x), f32::midpoint(min.y, max.y));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            let mut doc = shell.document().clone();
            doc.shapes.push(demo_square(shape_id, min, max));

            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(doc);
            shell.replace_selection_for_tests(gauss::model::Selection::empty());
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let click_point = point(px(centre.x), px(centre.y));
    visual_cx.simulate_mouse_down(click_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection.items,
        vec![SelItem::Shape(shape_id)],
        "expected clicking inside bbox to select the shape; selection={selection:?}"
    );

    visual_cx.simulate_mouse_up(click_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
}
