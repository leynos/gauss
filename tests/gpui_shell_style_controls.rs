//! GPUI headless integration tests for Phase 0 stroke/fill controls.

mod common;

use common::{
    anchor_to_canvas_point, click_canvas_and_wait, ensure_initial_draw, init_test_app,
    read_document, read_history_len, require_draw_shape, simulate_escape,
};
use gauss::model::{Paint, Rgba, SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Hsla, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult};

fn select_anchor0(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    select_point: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    let expected_anchor = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    if !selection.contains(&expected_anchor) {
        return Err(TestSupportError::expectation(format!(
            "expected anchor selection; selection={selection:?}"
        )));
    }

    visual_cx.simulate_mouse_up(select_point, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

#[gpui::test]
fn style_changes_apply_to_selected_shapes_and_are_undoable(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = common::canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );

    click_canvas_and_wait(visual_cx, p1);
    click_canvas_and_wait(visual_cx, p2);

    let doc_before = read_document(visual_cx, &view);
    let shape_before = require_draw_shape(&doc_before, "after drawing")
        .expect("expected draw shape after drawing")
        .clone();
    let anchor0 = shape_before
        .path
        .anchors
        .first()
        .map_or(Vec2::ZERO, |anchor| anchor.pos);

    simulate_escape(visual_cx);

    let select_point = anchor_to_canvas_point(&bounds, anchor0, p1);
    select_anchor0(visual_cx, &view, select_point, shape_before.id)
        .expect("expected anchor selection");

    let len_before_style = read_history_len(visual_cx, &view);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.apply_stroke_colour(Some(Hsla::red()));
            shell.apply_fill_colour(Some(Hsla::blue()));
        });
    });
    visual_cx.run_until_parked();
    assert_eq!(
        read_history_len(visual_cx, &view),
        len_before_style + 2,
        "expected two undo entries for stroke + fill changes"
    );

    let doc_after = read_document(visual_cx, &view);
    let shape_after = require_draw_shape(&doc_after, "after applying style")
        .expect("expected draw shape after applying style");
    assert_eq!(
        shape_after.style.stroke,
        Paint::Solid(Rgba::new(255, 0, 0, 255)),
        "expected stroke to be updated to red"
    );
    assert_eq!(
        shape_after.style.fill,
        Paint::Solid(Rgba::new(0, 0, 255, 255)),
        "expected fill to be updated to blue"
    );

    common::simulate_document_undo(visual_cx);
    common::simulate_document_undo(visual_cx);

    let doc_after_undo = read_document(visual_cx, &view);
    let shape_after_undo =
        require_draw_shape(&doc_after_undo, "after undo").expect("expected draw shape after undo");
    assert_eq!(
        shape_after_undo.style, shape_before.style,
        "expected undo to restore the original style"
    );
}
