//! Step definitions for keybinding registration and action dispatch.

use crate::{common, state};
use gauss::model::{Document, SelItem, ShapeId, Vec2};
use gauss::ui::{GpuiActivatePenTool, GpuiActivateSelectTool, GpuiDeselectAll, GpuiSelectAll};
use gauss_core::test_helpers::square_shape;
use gpui::TestAppContext;
use rstest_bdd_macros::{given, then, when};
use test_support::TestSupportError;

enum KeybindingState {
    Registration,
    SelectAll { first: ShapeId, second: ShapeId },
    DeselectAll,
    Mode,
    ActiveShape,
}

fn contains_both_shapes(selected: &[SelItem], first: ShapeId, second: ShapeId) -> bool {
    selected.len() == 2
        && selected.contains(&SelItem::Shape(first))
        && selected.contains(&SelItem::Shape(second))
}

#[given("the test application initializes its UI action bindings")]
fn initialize_ui_bindings(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |_visual_cx, _view| Ok(KeybindingState::Registration))
}

#[when("a Phase 0 shell window is opened")]
fn open_phase0_shell(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut KeybindingState| {
        common::ensure_initial_draw(visual_cx);
        Ok(())
    })
}

#[then("the shell completes its initial draw")]
fn shell_completes_initial_draw(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |_visual_cx, _view, data: &mut KeybindingState| {
        if !matches!(data, KeybindingState::Registration) {
            return Err(TestSupportError::missing(
                "registration state",
                "UI initialization scenario",
            ));
        }
        Ok(())
    })
}

#[given("a fresh Phase 0 shell window with two unselected shapes")]
fn shell_with_two_unselected_shapes(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let mut document = Document::new();
        let first = document.allocate_shape_id();
        let second = document.allocate_shape_id();
        let _first = document.append_shape(square_shape(
            first,
            Vec2::new(10.0, 10.0),
            Vec2::new(60.0, 60.0),
        ));
        let _second = document.append_shape(square_shape(
            second,
            Vec2::new(10.0, 10.0),
            Vec2::new(60.0, 60.0),
        ));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.replace_document_for_tests(document);
                shell.replace_selection_for_tests(gauss::model::Selection::empty());
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        if !common::read_selection_items(visual_cx, view).is_empty() {
            return Err(TestSupportError::expectation(
                "expected an empty selection before select all",
            ));
        }
        Ok(KeybindingState::SelectAll { first, second })
    })
}

#[when("the select-all action is dispatched")]
fn dispatch_select_all(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut KeybindingState| {
        visual_cx.dispatch_action(GpuiSelectAll);
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("both shapes are selected")]
fn both_shapes_are_selected(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, data: &mut KeybindingState| {
        let KeybindingState::SelectAll { first, second } = data else {
            return Err(TestSupportError::missing(
                "shape ids",
                "select-all scenario",
            ));
        };
        let selected = common::read_selection_items(visual_cx, view);
        if !contains_both_shapes(&selected, *first, *second) {
            return Err(TestSupportError::expectation(format!(
                "expected both shapes selected; got {selected:?}"
            )));
        }
        Ok(())
    })
}

#[given("a fresh Phase 0 shell window with one selected shape")]
fn shell_with_one_selected_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let mut document = Document::new();
        let shape_id = document.allocate_shape_id();
        let _shape = document.append_shape(square_shape(
            shape_id,
            Vec2::new(10.0, 10.0),
            Vec2::new(60.0, 60.0),
        ));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.replace_document_for_tests(document);
                shell.replace_selection_for_tests(gauss::model::Selection {
                    items: vec![SelItem::Shape(shape_id)],
                });
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        if common::read_selection_items(visual_cx, view).len() != 1 {
            return Err(TestSupportError::expectation(
                "expected one selected shape before deselect all",
            ));
        }
        Ok(KeybindingState::DeselectAll)
    })
}

#[when("the deselect-all action is dispatched")]
fn dispatch_deselect_all(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut KeybindingState| {
        visual_cx.dispatch_action(GpuiDeselectAll);
        visual_cx.run_until_parked();
        Ok(())
    })
}

#[then("the selection is empty")]
fn selection_is_empty(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut KeybindingState| {
        if !common::read_selection_items(visual_cx, view).is_empty() {
            return Err(TestSupportError::expectation(
                "expected an empty selection after deselect all",
            ));
        }
        Ok(())
    })
}

#[given("a fresh Phase 0 shell window in manipulate mode")]
fn shell_in_manipulate_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        if !visual_cx.read(|app| view.read(app).is_manipulate_mode()) {
            return Err(TestSupportError::expectation(
                "expected manipulate mode during setup",
            ));
        }
        Ok(KeybindingState::Mode)
    })
}

#[when("the pen tool is activated")]
fn activate_pen_tool(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    dispatch_action(cx, GpuiActivatePenTool)
}

#[then("draw mode is active")]
fn draw_mode_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_mode(
        cx,
        gauss::ui::Phase0Shell::is_draw_mode,
        "expected draw mode",
    )
}

#[given("a fresh Phase 0 shell window in draw mode")]
fn shell_in_draw_mode(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        if !visual_cx.read(|app| view.read(app).is_draw_mode()) {
            return Err(TestSupportError::expectation("expected initial draw mode"));
        }
        Ok(KeybindingState::Mode)
    })
}

#[when("the select tool is activated")]
fn activate_select_tool(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    dispatch_action(cx, GpuiActivateSelectTool)
}

#[then("manipulate mode is active")]
fn manipulate_mode_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_mode(
        cx,
        gauss::ui::Phase0Shell::is_manipulate_mode,
        "expected manipulate mode",
    )
}

#[given("a fresh Phase 0 shell window with an active draw shape")]
fn shell_with_active_draw_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let active = ShapeId::from_accesskit_node_id(42);
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.set_draw_active_shape_for_tests(Some(active));
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        let actual = visual_cx.read(|app| view.read(app).draw_active_shape_for_tests());
        if actual != Some(active) {
            return Err(TestSupportError::expectation("expected active draw shape"));
        }
        Ok(KeybindingState::ActiveShape)
    })
}

#[then("the active draw shape is clear")]
fn active_draw_shape_is_clear(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut KeybindingState| {
        if visual_cx
            .read(|app| view.read(app).draw_active_shape_for_tests())
            .is_some()
        {
            return Err(TestSupportError::expectation(
                "expected select tool to clear the active draw shape",
            ));
        }
        Ok(())
    })
}

#[given("a fresh Phase 0 shell window in draw mode with line edge mode")]
fn shell_in_draw_mode_with_line_edges(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        let is_ready = visual_cx.read(|app| {
            let shell = view.read(app);
            shell.is_draw_mode() && shell.is_line_edge_mode()
        });
        if !is_ready {
            return Err(TestSupportError::expectation(
                "expected draw mode with line edge mode before Tab",
            ));
        }
        Ok(KeybindingState::Mode)
    })
}

#[given("a fresh Phase 0 shell window in manipulate mode with line edge mode")]
fn shell_in_manipulate_mode_with_line_edges(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::initialize(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.enter_manipulate_mode_for_tests();
                view_cx.notify();
            });
        });
        visual_cx.run_until_parked();
        let is_ready = visual_cx.read(|app| {
            let shell = view.read(app);
            shell.is_manipulate_mode() && shell.is_line_edge_mode()
        });
        if !is_ready {
            return Err(TestSupportError::expectation(
                "expected manipulate mode with line edge mode before Tab",
            ));
        }
        Ok(KeybindingState::Mode)
    })
}

#[when("Tab is pressed")]
fn press_tab(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut KeybindingState| {
        common::simulate_key(visual_cx, "tab", gpui::Modifiers::none());
        Ok(())
    })
}

#[then("Bezier auto edge mode is active")]
fn bezier_auto_is_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_mode(
        cx,
        gauss::ui::Phase0Shell::is_bezier_edge_mode,
        "expected Bezier edge mode",
    )
}

#[then("manipulate mode with line edge mode remains active")]
fn manipulate_with_line_edges_remains_active(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    assert_mode(
        cx,
        |shell| shell.is_manipulate_mode() && shell.is_line_edge_mode(),
        "expected manipulate mode with unchanged line edge mode",
    )
}

fn dispatch_action(
    cx: &mut TestAppContext,
    action: impl gpui::Action,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, _view, _data: &mut KeybindingState| {
        visual_cx.dispatch_action(action);
        visual_cx.run_until_parked();
        Ok(())
    })
}

fn assert_mode(
    cx: &mut TestAppContext,
    predicate: impl FnOnce(&gauss::ui::Phase0Shell) -> bool,
    message: &'static str,
) -> Result<(), TestSupportError> {
    state::with_visual_cx(cx, |visual_cx, view, _data: &mut KeybindingState| {
        if !visual_cx.read(|app| predicate(view.read(app))) {
            return Err(TestSupportError::expectation(message));
        }
        Ok(())
    })
}
