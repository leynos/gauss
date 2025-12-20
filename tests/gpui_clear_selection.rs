//! GPUI headless integration tests for selection behaviour in manipulate mode.
//!
//! Phase 0 uses `on_mouse_down` for selection. This test verifies that clicking
//! empty canvas space clears the current selection.
#![expect(
    clippy::float_arithmetic,
    reason = "integration tests use floating point geometry inputs"
)]

mod common;

use common::{click_left_and_wait, ensure_initial_draw, init_test_app};
use gauss::model::{PaintStyle, Rgba, SegmentKind, SelItem, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, VisualTestContext, point, px};
use uuid::Uuid;

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

fn read_selection(visual_cx: &VisualTestContext, view: &gpui::Entity<Phase0Shell>) -> Vec<SelItem> {
    visual_cx.read(|app| view.read(app).selection().items.clone())
}

#[gpui::test]
fn clicking_empty_space_clears_selection(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = common::canvas_bounds(visual_cx).expect("canvas bounds should be available");

    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);

    let shape_id = ShapeId::from(Uuid::from_u128(0x3333_3333_3333_3333_3333_3333_3333_3333));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            let mut doc = shell.document().clone();
            doc.shapes.push(demo_square(
                shape_id,
                origin.add(Vec2::new(10.0, 10.0)),
                origin.add(Vec2::new(60.0, 60.0)),
            ));

            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(doc);
            shell.replace_selection_for_tests(gauss::model::Selection {
                items: vec![SelItem::Shape(shape_id)],
            });
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let selection_before = read_selection(visual_cx, &view);
    assert_eq!(
        selection_before,
        vec![SelItem::Shape(shape_id)],
        "expected selection to start with the demo square selected"
    );

    let click_x = (width - 2.0).max(2.0);
    let click_y = (height - 2.0).max(2.0);
    let empty_point = point(bounds.origin.x + px(click_x), bounds.origin.y + px(click_y));

    click_left_and_wait(visual_cx, empty_point);

    let selection_after = read_selection(visual_cx, &view);
    assert!(
        selection_after.is_empty(),
        "expected clicking empty space to clear selection; got {selection_after:?}"
    );
}
