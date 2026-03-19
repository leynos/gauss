//! GPUI headless integration tests for manipulate-mode multi-select.
//!
//! Phase 0 uses Shift+click to toggle selection items without starting a drag
//! gesture. This keeps multi-select available while Shift remains reserved for
//! selection-history undo/redo shortcuts.

mod common;

use common::{
    anchor_to_canvas_point, draw_point, ensure_initial_draw, init_test_app, require_draw_shape,
    shift_secondary,
};
use gauss::model::{SelItem, Selection, ShapeId};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Clone, Copy, Debug)]
struct AnchorPointSelection {
    shape_id: ShapeId,
    anchor0_point: gpui::Point<gpui::Pixels>,
    anchor1_point: gpui::Point<gpui::Pixels>,
}

fn mouse_down_left(
    visual_cx: &mut VisualTestContext,
    position: gpui::Point<gpui::Pixels>,
    modifiers: Modifiers,
) {
    visual_cx.simulate_mouse_down(position, MouseButton::Left, modifiers);
    visual_cx.run_until_parked();
}

fn mouse_up_left(
    visual_cx: &mut VisualTestContext,
    position: gpui::Point<gpui::Pixels>,
    modifiers: Modifiers,
) {
    visual_cx.simulate_mouse_up(position, MouseButton::Left, modifiers);
    visual_cx.run_until_parked();
}

fn enter_manipulate_mode(visual_cx: &mut VisualTestContext, view: &gpui::Entity<Phase0Shell>) {
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();
}

fn draw_two_points_and_anchor_points(
    visual_cx: &mut VisualTestContext,
    view: &gpui::Entity<Phase0Shell>,
) -> TestSupportResult<AnchorPointSelection> {
    let bounds = common::canvas_bounds(visual_cx)?;

    let p1 = point(bounds.origin.x + px(2.0), bounds.origin.y + px(2.0));
    let p2 = point(
        bounds.origin.x + bounds.size.width - px(2.0),
        bounds.origin.y + bounds.size.height - px(2.0),
    );

    draw_point(visual_cx, p1);
    visual_cx.run_until_parked();
    draw_point(visual_cx, p2);
    visual_cx.run_until_parked();

    let doc = visual_cx.read(|app| view.read(app).document().clone());
    if doc.len() != 2 {
        return Err(TestSupportError::expectation(format!(
            "expected demo + one drawn shape; shapes={:?}",
            doc.iter_ids_in_draw_order().collect::<Vec<_>>()
        )));
    }
    let shape = require_draw_shape(&doc, "after drawing two points")?;

    let anchor0 = shape
        .path
        .anchors
        .first()
        .map(|anchor| anchor.pos)
        .ok_or_else(|| TestSupportError::missing("anchor 0", "after drawing"))?;
    let anchor1 = shape
        .path
        .anchors
        .get(1)
        .map(|anchor| anchor.pos)
        .ok_or_else(|| TestSupportError::missing("anchor 1", "after drawing"))?;

    let anchor0_point = anchor_to_canvas_point(&bounds, anchor0, p1);
    let anchor1_point = anchor_to_canvas_point(&bounds, anchor1, p1);

    Ok(AnchorPointSelection {
        shape_id: shape.id,
        anchor0_point,
        anchor1_point,
    })
}

#[gpui::test]
fn shift_click_toggles_multi_select_without_dragging(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let selection_setup = draw_two_points_and_anchor_points(visual_cx, &view)
        .expect("expected two points and anchor positions");
    enter_manipulate_mode(visual_cx, &view);

    let anchor0_item = SelItem::Anchor {
        shape: selection_setup.shape_id,
        anchor: 0,
    };
    let anchor1_item = SelItem::Anchor {
        shape: selection_setup.shape_id,
        anchor: 1,
    };

    mouse_down_left(visual_cx, selection_setup.anchor0_point, Modifiers::none());
    let selection_single = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_single,
        Selection {
            items: vec![
                SelItem::Shape(selection_setup.shape_id),
                anchor0_item.clone(),
            ],
        },
        "expected first click to select the first anchor"
    );
    mouse_up_left(visual_cx, selection_setup.anchor0_point, Modifiers::none());

    let shift_mods = shift_secondary(Modifiers::none());
    mouse_down_left(visual_cx, selection_setup.anchor1_point, shift_mods);
    let selection_multi = visual_cx.read(|app| view.read(app).selection().clone());
    assert!(
        selection_multi.contains(&anchor0_item) && selection_multi.contains(&anchor1_item),
        "expected Shift+click to add a second selected anchor; selection={selection_multi:?}"
    );

    let is_dragging = visual_cx.read(|app| view.read(app).is_dragging());
    assert!(
        !is_dragging,
        "Shift+click should not start a drag gesture (it is selection-only)"
    );
    mouse_up_left(visual_cx, selection_setup.anchor1_point, shift_mods);

    mouse_down_left(visual_cx, selection_setup.anchor0_point, shift_mods);
    mouse_up_left(visual_cx, selection_setup.anchor0_point, shift_mods);
    let selection_toggled = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_toggled,
        Selection {
            items: vec![SelItem::Shape(selection_setup.shape_id), anchor1_item],
        },
        "expected Shift+click to toggle the clicked item off"
    );
}
