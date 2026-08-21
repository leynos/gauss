//! Structural GPUI coverage for Phase 0 shell style controls.
//!
//! These checks exercise test-only colour application helpers because the GPUI
//! test platform cannot address the colour pickers reliably. They verify style
//! command and undo plumbing rather than user-visible picker interaction.

#[path = "common/gpui_shell_style_controls.rs"]
mod common;

#[path = "common/init_app.rs"]
mod init_app;
#[path = "common/initial_draw.rs"]
mod initial_draw;

use common::{
    anchor_to_canvas_point, click_canvas_and_wait, read_document, read_history_len,
    require_draw_shape, simulate_document_undo, simulate_escape,
};
use gauss::model::{Paint, Rgba, SelItem, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Hsla, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use init_app::init_test_app;
use initial_draw::ensure_initial_draw;
use test_support::{TestSupportError, TestSupportResult};

fn select_first_anchor(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
    position: gpui::Point<gpui::Pixels>,
    shape_id: ShapeId,
) -> TestSupportResult<()> {
    visual_cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    let expected = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    if !selection.contains(&expected) {
        return Err(TestSupportError::expectation(format!(
            "expected first anchor selection; selection={selection:?}"
        )));
    }
    visual_cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    Ok(())
}

fn apply_colour_changes(visual_cx: &mut VisualTestContext, view: &gpui::Entity<Phase0Shell>) {
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.apply_stroke_colour(Some(Hsla::red()));
        });
    });
    visual_cx.run_until_parked();
    visual_cx.update(|_window, app| {
        view.update(app, |shell, _cx| {
            shell.apply_fill_colour(Some(Hsla::blue()));
        });
    });
    visual_cx.run_until_parked();
}

#[gpui::test]
fn style_changes_apply_to_selected_shapes_and_are_undoable(cx: &mut TestAppContext) {
    init_test_app(cx);
    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let bounds = common::canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let first = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let second = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );
    click_canvas_and_wait(visual_cx, first);
    click_canvas_and_wait(visual_cx, second);

    let document = read_document(visual_cx, &view);
    let (shape_id, anchor, original_style) = {
        let shape = require_draw_shape(&document, "after drawing")
            .expect("expected draw shape after drawing");
        (
            shape.id,
            shape
                .path
                .anchors
                .first()
                .map_or(Vec2::ZERO, |item| item.pos),
            shape.style.clone(),
        )
    };
    simulate_escape(visual_cx);
    select_first_anchor(
        visual_cx,
        &view,
        anchor_to_canvas_point(&bounds, anchor, first),
        shape_id,
    )
    .expect("expected first anchor selection");

    let history_len_before_style = read_history_len(visual_cx, &view);
    apply_colour_changes(visual_cx, &view);

    let styled_document = read_document(visual_cx, &view);
    let styled_shape = require_draw_shape(&styled_document, "after applying style")
        .expect("expected draw shape after applying style");
    assert_eq!(
        styled_shape.style.stroke,
        Paint::Solid(Rgba::new(255, 0, 0, 255)),
        "expected stroke to be updated to red"
    );
    assert_eq!(
        styled_shape.style.fill,
        Paint::Solid(Rgba::new(0, 0, 255, 255)),
        "expected fill to be updated to blue"
    );
    assert_eq!(
        read_history_len(visual_cx, &view),
        history_len_before_style + 2,
        "expected two undo entries for stroke and fill changes"
    );

    simulate_document_undo(visual_cx);
    simulate_document_undo(visual_cx);
    let restored_document = read_document(visual_cx, &view);
    let restored_shape = require_draw_shape(&restored_document, "after undo")
        .expect("expected draw shape after undo");
    assert_eq!(
        restored_shape.style, original_style,
        "expected undo to restore the original style"
    );
}
