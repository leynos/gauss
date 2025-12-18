//! GPUI headless integration tests for manipulate-mode multi-select.
//!
//! Phase 0 uses Shift+click to toggle selection items without starting a drag
//! gesture. This keeps multi-select available while Shift remains reserved for
//! selection-history undo/redo shortcuts.

use gauss::model::{Document, SelItem, Selection, Shape, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px};
use uuid::Uuid;

fn demo_shape_id() -> ShapeId {
    ShapeId::from(Uuid::from_u128(0x6d3c_0fb4_43a8_48f1_9f14_623a_70d5_2e1a))
}

fn ensure_initial_draw(visual_cx: &mut VisualTestContext) {
    visual_cx.update(|window, app| drop(window.draw(app)));
    visual_cx.run_until_parked();
}

fn draw_point(visual_cx: &mut VisualTestContext, position: gpui::Point<gpui::Pixels>) {
    visual_cx.simulate_mouse_move(position, None, Modifiers::none());
    visual_cx.simulate_click(position, Modifiers::none());
}

fn require_draw_shape(doc: &Document) -> &Shape {
    let demo_id = demo_shape_id();
    let Some(shape) = doc.shapes.iter().find(|shape| shape.id != demo_id) else {
        panic!("expected a drawn shape to exist");
    };
    shape
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

const fn with_shift(mut modifiers: Modifiers) -> Modifiers {
    modifiers.shift = true;
    modifiers
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
) -> (
    gpui::Bounds<gpui::Pixels>,
    ShapeId,
    gpui::Point<gpui::Pixels>,
    gpui::Point<gpui::Pixels>,
) {
    let Some(bounds) = visual_cx.debug_bounds("#phase0-canvas") else {
        panic!("phase0 canvas should have debug bounds");
    };

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
    assert_eq!(
        doc.shapes.len(),
        2,
        "expected demo + one drawn shape; shapes={:?}",
        doc.shapes.iter().map(|shape| shape.id).collect::<Vec<_>>()
    );
    let shape = require_draw_shape(&doc);

    let Some(anchor0) = shape.path.anchors.first().map(|anchor| anchor.pos) else {
        panic!("expected first anchor after drawing");
    };
    let Some(anchor1) = shape.path.anchors.get(1).map(|anchor| anchor.pos) else {
        panic!("expected second anchor after drawing");
    };

    let use_local = anchor0_is_local(anchor0, p1);
    let anchor0_point = model_to_screen_point(&bounds, use_local, anchor0);
    let anchor1_point = model_to_screen_point(&bounds, use_local, anchor1);

    (bounds, shape.id, anchor0_point, anchor1_point)
}

#[gpui::test]
fn shift_click_toggles_multi_select_without_dragging(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let (_bounds, shape_id, anchor0_point, anchor1_point) =
        draw_two_points_and_anchor_points(visual_cx, &view);
    enter_manipulate_mode(visual_cx, &view);

    let anchor0_item = SelItem::Anchor {
        shape: shape_id,
        anchor: 0,
    };
    let anchor1_item = SelItem::Anchor {
        shape: shape_id,
        anchor: 1,
    };

    mouse_down_left(visual_cx, anchor0_point, Modifiers::none());
    let selection_single = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_single,
        Selection {
            items: vec![anchor0_item.clone()],
        },
        "expected first click to select the first anchor"
    );
    mouse_up_left(visual_cx, anchor0_point, Modifiers::none());

    let shift_mods = with_shift(Modifiers::none());
    mouse_down_left(visual_cx, anchor1_point, shift_mods);
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
    mouse_up_left(visual_cx, anchor1_point, shift_mods);

    mouse_down_left(visual_cx, anchor0_point, shift_mods);
    mouse_up_left(visual_cx, anchor0_point, shift_mods);
    let selection_toggled = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection_toggled,
        Selection {
            items: vec![anchor1_item],
        },
        "expected Shift+click to toggle the clicked item off"
    );
}
