//! GPUI integration tests for keybinding registration and action dispatch.
//!
//! These tests verify:
//!
//! - [`ui::init`] correctly registers action bindings from the model layer
//! - [`bind_keymap`] registers shell-specific keybindings (Tab, window controls)
//! - [`bind_model_actions`] correctly wires GPUI actions to shell methods
//! - [`select_all`] and [`deselect_all`] modify selection as expected
//!
//! Note: [`KeyContext::Global`] is implicitly tested by all action dispatch tests.
//! GPUI requires a matching key context for actions to dispatch; if the root element
//! did not have [`KeyContext::Global`] set, `dispatch_action()` calls would silently
//! fail and assertions would not pass.

mod common;

use common::{ensure_initial_draw, init_test_app, read_selection_items};
use gauss::model::{Document, SelItem, ShapeId, Vec2};
use gauss::test_helpers::square_shape;
use gauss::ui::{
    GpuiActivatePenTool, GpuiActivateSelectTool, GpuiDeselectAll, GpuiSelectAll, Phase0Shell,
};
use gpui::TestAppContext;

// === ui::init registration tests ===

#[gpui::test]
fn ui_init_registers_action_bindings(cx: &mut TestAppContext) {
    // ui::init should register action bindings without panicking.
    // If registration fails, the test will panic during init.
    init_test_app(cx);

    // Verify the test context is functional by creating a view.
    let (_view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    // If we got here, ui::init successfully registered bindings.
}

// === bind_model_actions tests ===

#[gpui::test]
fn gpui_select_all_action_selects_all_shapes(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let mut doc = Document::new();
    let first_shape = doc.allocate_shape_id();
    let second_shape = doc.allocate_shape_id();
    doc.append_shape(square_shape(
        first_shape,
        Vec2::new(10.0, 10.0),
        Vec2::new(60.0, 60.0),
    ));
    doc.append_shape(square_shape(
        second_shape,
        Vec2::new(10.0, 10.0),
        Vec2::new(60.0, 60.0),
    ));

    // Add two shapes to the document.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.replace_document_for_tests(doc);
            shell.replace_selection_for_tests(gauss::model::Selection::empty());
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    // Selection should be empty initially.
    let selection_before = read_selection_items(visual_cx, &view);
    assert!(
        selection_before.is_empty(),
        "expected empty selection before GpuiSelectAll"
    );

    // Dispatch GpuiSelectAll action.
    visual_cx.dispatch_action(GpuiSelectAll);
    visual_cx.run_until_parked();

    // All shapes should be selected.
    let selection_after = read_selection_items(visual_cx, &view);
    assert_eq!(
        selection_after.len(),
        2,
        "expected 2 shapes selected after GpuiSelectAll"
    );
    assert!(selection_after.contains(&SelItem::Shape(first_shape)));
    assert!(selection_after.contains(&SelItem::Shape(second_shape)));
}

#[gpui::test]
fn gpui_deselect_all_action_clears_selection(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    let mut doc = Document::new();
    let shape_id = doc.allocate_shape_id();
    doc.append_shape(square_shape(
        shape_id,
        Vec2::new(10.0, 10.0),
        Vec2::new(60.0, 60.0),
    ));

    // Add a shape and select it.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.replace_document_for_tests(doc);
            shell.replace_selection_for_tests(gauss::model::Selection {
                items: vec![SelItem::Shape(shape_id)],
            });
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    // Selection should have the shape.
    let selection_before = read_selection_items(visual_cx, &view);
    assert_eq!(selection_before.len(), 1, "expected 1 shape selected");

    // Dispatch GpuiDeselectAll action.
    visual_cx.dispatch_action(GpuiDeselectAll);
    visual_cx.run_until_parked();

    // Selection should be empty.
    let selection_after = read_selection_items(visual_cx, &view);
    assert!(
        selection_after.is_empty(),
        "expected empty selection after GpuiDeselectAll"
    );
}

#[gpui::test]
fn gpui_activate_pen_tool_switches_to_draw_mode(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    // Start in manipulate mode.
    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let is_manipulate_before = visual_cx.read(|app| view.read(app).is_manipulate_mode());
    assert!(
        is_manipulate_before,
        "expected manipulate mode before GpuiActivatePenTool"
    );

    // Dispatch GpuiActivatePenTool action.
    visual_cx.dispatch_action(GpuiActivatePenTool);
    visual_cx.run_until_parked();

    let is_draw_after = visual_cx.read(|app| view.read(app).is_draw_mode());
    assert!(
        is_draw_after,
        "expected draw mode after GpuiActivatePenTool"
    );
}

#[gpui::test]
fn gpui_activate_select_tool_switches_to_manipulate_mode(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    // Start in draw mode (default).
    let is_draw_before = visual_cx.read(|app| view.read(app).is_draw_mode());
    assert!(
        is_draw_before,
        "expected draw mode before GpuiActivateSelectTool"
    );

    // Dispatch GpuiActivateSelectTool action.
    visual_cx.dispatch_action(GpuiActivateSelectTool);
    visual_cx.run_until_parked();

    let is_manipulate_after = visual_cx.read(|app| view.read(app).is_manipulate_mode());
    assert!(
        is_manipulate_after,
        "expected manipulate mode after GpuiActivateSelectTool"
    );
}

#[gpui::test]
fn gpui_activate_select_tool_clears_active_draw_shape(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.set_draw_active_shape_for_tests(Some(ShapeId::from_accesskit_node_id(42)));
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let active_shape_before = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
    assert_eq!(
        active_shape_before,
        Some(ShapeId::from_accesskit_node_id(42))
    );

    visual_cx.dispatch_action(GpuiActivateSelectTool);
    visual_cx.run_until_parked();

    let active_shape_after = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
    assert!(
        active_shape_after.is_none(),
        "expected active draw shape to be cleared after GpuiActivateSelectTool"
    );
}

// === bind_keymap tests ===

#[gpui::test]
fn tab_key_toggles_edge_mode_in_draw_mode(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    // Start in draw mode with Line edge mode (default).
    let is_line_before = visual_cx.read(|app| view.read(app).is_line_edge_mode());
    assert!(is_line_before, "expected line edge mode before Tab");

    // Simulate Tab key press.
    common::simulate_key(visual_cx, "tab", gpui::Modifiers::none());

    let is_bezier_after = visual_cx.read(|app| view.read(app).is_bezier_edge_mode());
    assert!(is_bezier_after, "expected bezier edge mode after Tab");
}

#[gpui::test]
fn tab_key_in_manipulate_mode_does_not_toggle_draw_edge_mode(cx: &mut TestAppContext) {
    init_test_app(cx);

    let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);

    visual_cx.update(|_window, app| {
        view.update(app, |shell, view_cx| {
            shell.enter_manipulate_mode_for_tests();
            view_cx.notify();
        });
    });
    visual_cx.run_until_parked();

    let is_manipulate_before = visual_cx.read(|app| view.read(app).is_manipulate_mode());
    let is_line_before = visual_cx.read(|app| view.read(app).is_line_edge_mode());
    assert!(
        is_manipulate_before,
        "expected manipulate mode before Tab in manipulate mode test"
    );
    assert!(is_line_before, "expected line edge mode before Tab");

    common::simulate_key(visual_cx, "tab", gpui::Modifiers::none());

    let is_manipulate_after = visual_cx.read(|app| view.read(app).is_manipulate_mode());
    let is_line_after = visual_cx.read(|app| view.read(app).is_line_edge_mode());
    assert!(
        is_manipulate_after,
        "expected manipulate mode after Tab in manipulate mode test"
    );
    assert!(
        is_line_after,
        "expected draw edge mode to remain line after Tab in manipulate mode"
    );
}
