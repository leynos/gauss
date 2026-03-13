//! GPUI integration tests for shared hit-test service wiring.

mod common;

use common::{canvas_bounds, ensure_initial_draw, init_test_app};
use gauss::model::{Document, SelItem, SelectPointerHit, ShapeId, Vec2};
use gauss::test_helpers::square_shape_with_out_handle;
use gauss::ui::{GpuiActivatePenTool, Phase0Shell};
use gpui::{Modifiers, MouseButton, TestAppContext, point, px};

#[gpui::test]
fn hovering_handle_prefers_handle_hit(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let min = origin.add(Vec2::new(20.0, 20.0));
    let max = origin.add(Vec2::new(120.0, 120.0));

    let mut document = Document::new();
    let shape_id = ShapeId::from_accesskit_node_id(301);
    let _inserted = document.append_shape(square_shape_with_out_handle(
        shape_id,
        min,
        max,
        Vec2::new(6.0, 0.0),
    ));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(document);
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let viewport = visual_cx.read(|app| view.read(app).viewport());
    let handle_world = min.add(Vec2::new(6.0, 0.0));
    let handle_screen = {
        let screen = viewport.world_to_screen(handle_world);
        point(px(screen.x), px(screen.y))
    };
    visual_cx.simulate_mouse_move(handle_screen, None, Modifiers::none());
    visual_cx.run_until_parked();

    let hover_hit = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
    assert!(matches!(
        hover_hit,
        SelectPointerHit::Handle(handle_hit)
            if handle_hit.shape_id == shape_id && handle_hit.anchor_index == 0
    ));
}

#[gpui::test]
fn hover_hit_clears_when_cursor_moves_to_empty_space(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let min = origin.add(Vec2::new(20.0, 20.0));
    let max = origin.add(Vec2::new(120.0, 120.0));

    let mut document = Document::new();
    let shape_id = ShapeId::from_accesskit_node_id(302);
    let _inserted = document.append_shape(square_shape_with_out_handle(
        shape_id,
        min,
        max,
        Vec2::new(6.0, 0.0),
    ));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(document);
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let viewport = visual_cx.read(|app| view.read(app).viewport());
    let segment_world = min.add(Vec2::new(40.0, 0.0));
    let segment_screen = {
        let screen = viewport.world_to_screen(segment_world);
        point(px(screen.x), px(screen.y))
    };
    visual_cx.simulate_mouse_move(segment_screen, None, Modifiers::none());
    visual_cx.run_until_parked();

    let hover_hit = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
    assert!(matches!(hover_hit, SelectPointerHit::Segment(_)));

    let empty_screen = {
        let screen = viewport.world_to_screen(max.add(Vec2::new(200.0, 200.0)));
        point(px(screen.x), px(screen.y))
    };
    visual_cx.simulate_mouse_move(empty_screen, None, Modifiers::none());
    visual_cx.run_until_parked();

    let cleared_hover_hit = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
    assert_eq!(cleared_hover_hit, SelectPointerHit::None);
}

#[gpui::test]
fn hover_hit_clears_when_leaving_manipulate_mode(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let min = origin.add(Vec2::new(20.0, 20.0));
    let max = origin.add(Vec2::new(120.0, 120.0));

    let mut document = Document::new();
    let shape_id = ShapeId::from_accesskit_node_id(303);
    let _inserted = document.append_shape(square_shape_with_out_handle(
        shape_id,
        min,
        max,
        Vec2::new(6.0, 0.0),
    ));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(document);
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let viewport = visual_cx.read(|app| view.read(app).viewport());
    let handle_world = min.add(Vec2::new(6.0, 0.0));
    let handle_screen = {
        let screen = viewport.world_to_screen(handle_world);
        point(px(screen.x), px(screen.y))
    };
    visual_cx.simulate_mouse_move(handle_screen, None, Modifiers::none());
    visual_cx.run_until_parked();

    let hover_before = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
    assert!(
        !matches!(hover_before, SelectPointerHit::None),
        "expected hover hit before leaving manipulate mode; got {hover_before:?}"
    );

    visual_cx.dispatch_action(GpuiActivatePenTool);
    visual_cx.run_until_parked();

    let hover_after = visual_cx.read(|app| view.read(app).hover_hit_for_tests());
    assert_eq!(
        hover_after,
        SelectPointerHit::None,
        "expected hover hit to clear when leaving manipulate mode"
    );
}

#[gpui::test]
fn clicking_overlapping_shapes_selects_topmost_shape(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let bounds = canvas_bounds(visual_cx).expect("canvas bounds should be available");
    let origin = Vec2::new(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let min = origin.add(Vec2::new(20.0, 20.0));
    let max = origin.add(Vec2::new(120.0, 120.0));
    let centre = Vec2::new(f32::midpoint(min.x, max.x), f32::midpoint(min.y, max.y));

    let mut document = Document::new();
    let bottom = ShapeId::from_accesskit_node_id(401);
    let top = ShapeId::from_accesskit_node_id(402);
    let _bottom = document.append_shape(square_shape_with_out_handle(
        bottom,
        min,
        max,
        Vec2::new(6.0, 0.0),
    ));
    let _top = document.append_shape(square_shape_with_out_handle(
        top,
        min,
        max,
        Vec2::new(6.0, 0.0),
    ));

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            shell.replace_document_for_tests(document);
            shell.replace_selection_for_tests(gauss::model::Selection::empty());
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let viewport = visual_cx.read(|app| view.read(app).viewport());
    let centre_screen = {
        let screen = viewport.world_to_screen(centre);
        point(px(screen.x), px(screen.y))
    };
    visual_cx.simulate_mouse_down(centre_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();
    visual_cx.simulate_mouse_up(centre_screen, MouseButton::Left, Modifiers::none());
    visual_cx.run_until_parked();

    let selection = visual_cx.read(|app| view.read(app).selection().clone());
    assert_eq!(
        selection.items,
        vec![SelItem::Shape(top)],
        "expected topmost overlapping shape to be selected; selection={selection:?}"
    );
}
