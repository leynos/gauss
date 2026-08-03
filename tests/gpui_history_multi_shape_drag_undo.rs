//! GPUI headless integration test verifying that multi-shape drag produces
//! exactly one undo entry.
//!
//! When multiple shapes are selected, dragging moves all of them but must
//! create only a single history entry — not one entry per shape.

mod common;

use common::{
    add_square, assert_shape_translated_by_delta, canvas_bounds, ensure_initial_draw,
    init_test_app, read_document, read_history_len, simulate_document_undo,
    viewport_to_screen_point,
};
use gauss::model::{SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};
use test_support::TestSupportResult;
use test_support::selection::{require_shape, shape_bbox_centre};

fn arrange_multi_shape_selection(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    origin: Vec2,
) -> TestSupportResult<(ShapeId, ShapeId)> {
    let min1 = origin.add(Vec2::new(10.0, 10.0));
    let max1 = origin.add(Vec2::new(110.0, 110.0));
    let min2 = origin.add(Vec2::new(160.0, 10.0));
    let max2 = origin.add(Vec2::new(260.0, 110.0));

    let mut doc = visual_cx.read(|app| view.read(app).document().clone());
    let first = add_square(&mut doc, min1, max1)?;
    let second = add_square(&mut doc, min2, max2)?;

    visual_cx.update(move |_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(doc);
            shell.replace_selection_for_tests(gauss::model::Selection {
                items: vec![SelItem::Shape(first), SelItem::Shape(second)],
            });
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();
    Ok((first, second))
}

#[gpui::test]
fn multi_shape_drag_creates_exactly_one_undo_entry(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));

    let (shape1_id, shape2_id) = arrange_multi_shape_selection(visual_cx, &view, origin)
        .expect("expected to arrange multi-shape selection");

    let viewport = visual_cx.read(|app| view.read(app).viewport());

    let doc_before = read_document(visual_cx, &view);
    let shape1_before = require_shape(&doc_before, shape1_id, "before drag")
        .expect("expected shape1")
        .clone();
    let shape2_before = require_shape(&doc_before, shape2_id, "before drag")
        .expect("expected shape2")
        .clone();

    let len_before = read_history_len(visual_cx, &view);

    let start_model = shape_bbox_centre(&shape1_before).expect("shape1 should have anchors");
    let delta = Vec2::new(20.0, 10.0);
    let start_screen = viewport_to_screen_point(viewport, start_model);
    let end_screen = viewport_to_screen_point(viewport, start_model.add(delta));

    visual_cx.simulate_mouse_down(start_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    visual_cx.simulate_mouse_move(end_screen, MouseButton::Left, Modifiers::none());
    visual_cx.simulate_mouse_up(end_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let len_after = read_history_len(visual_cx, &view);
    assert_eq!(
        len_after,
        len_before + 1,
        "expected exactly one undo entry for multi-shape drag (not one per shape)"
    );

    let doc_after = read_document(visual_cx, &view);
    let shape1_after =
        require_shape(&doc_after, shape1_id, "after drag").expect("expected shape1 after drag");
    let shape2_after =
        require_shape(&doc_after, shape2_id, "after drag").expect("expected shape2 after drag");
    assert_shape_translated_by_delta(shape1_after, &shape1_before, delta, "shape1 after drag")
        .expect("expected shape1 to translate");
    assert_shape_translated_by_delta(shape2_after, &shape2_before, delta, "shape2 after drag")
        .expect("expected shape2 to translate");

    simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let shape1_restored = require_shape(&doc_after_undo, shape1_id, "after undo")
        .expect("expected shape1 after undo");
    let shape2_restored = require_shape(&doc_after_undo, shape2_id, "after undo")
        .expect("expected shape2 after undo");
    assert_shape_translated_by_delta(shape1_restored, &shape1_before, Vec2::ZERO, "shape1 undo")
        .expect("expected shape1 to restore");
    assert_shape_translated_by_delta(shape2_restored, &shape2_before, Vec2::ZERO, "shape2 undo")
        .expect("expected shape2 to restore");
}
