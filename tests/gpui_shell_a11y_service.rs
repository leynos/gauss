//! Behavioural coverage for the shell accessibility service through `GpuiHarness`.

#[path = "common/gpui_shell_a11y_service.rs"]
mod common;

#[path = "common/durable_shell.rs"]
mod durable_shell;

#[path = "shell_bdd/lifecycle.rs"]
mod lifecycle;

#[path = "common/scenario_state.rs"]
mod scenario_state;
#[path = "shell_bdd/expect_equal.rs"]
mod expect_equal_support;

#[path = "shell_bdd/expect_true.rs"]
mod expect_true_support;

#[path = "shell_bdd/support.rs"]
mod support;

use std::cell::RefCell;

use accesskit::{Action, ActionRequest, Role, TreeId, TreeUpdate};
use expect_equal_support::expect_equal;
use expect_true_support::expect_true;
use gauss::model::{SelItem, Selection, ShapeId, Vec2};
use gauss::ui::Phase0Shell;
use gauss::ui::phase0_shell::{
    A11yActionRequestError, A11yRequestedAction, A11yUpdateKind, A11yWindowAction, CloseWindow,
    accessibility,
};
use gpui::TestAppContext;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use support::{ScenarioStateCleanup, fresh_shell_with, with_shell};
use test_support::{TestSupportError, TestSupportResult};

#[derive(Default)]
struct A11yState {
    inserted_shape_id: Option<u64>,
    routed: Option<Result<A11yRequestedAction, A11yActionRequestError>>,
}

thread_local! {
    static A11Y_STATE: RefCell<A11yState> = RefCell::new(A11yState::default());
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    A11Y_STATE.with(|cell| *cell.borrow_mut() = A11yState::default());
    fresh_shell_with(cx, Phase0Shell::new_for_tests)
}

#[when("a shape is inserted")]
fn insert_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| shell.clear_a11y_updates_for_tests());
        });
        let inserted_id = visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                let shape_id = common::add_square(
                    shell.document_mut_for_tests(),
                    Vec2::new(10.0, 10.0),
                    Vec2::new(40.0, 40.0),
                )?;
                view_cx.notify();
                Ok::<u64, TestSupportError>(shape_id.to_accesskit_node_id())
            })
        })?;
        A11Y_STATE.with(|cell| cell.borrow_mut().inserted_shape_id = Some(inserted_id));
        common::ensure_initial_draw(visual_cx);
        Ok(())
    })
}

#[when("accessibility updates are cleared")]
fn clear_accessibility_updates(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| shell.clear_a11y_updates_for_tests());
        });
        Ok(())
    })
}

#[when("the shell is rendered without state changes")]
fn render_without_state_changes(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        common::ensure_initial_draw(visual_cx);
        Ok(())
    })
}

#[when("a stale shape is selected")]
fn select_stale_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        visual_cx.update(|_window, app| {
            view.update(app, |shell, view_cx| {
                shell.clear_a11y_updates_for_tests();
                let stale_id = ShapeId::from_accesskit_node_id(0x2_ffff_ffff);
                shell.replace_selection_for_tests(Selection {
                    items: vec![SelItem::Shape(stale_id)],
                });
                view_cx.notify();
            });
        });
        common::ensure_initial_draw(visual_cx);
        Ok(())
    })
}

#[when("the close-window action is dispatched")]
fn dispatch_close_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, _view| {
        visual_cx.dispatch_action(CloseWindow);
        visual_cx.run_until_parked();
        Ok(())
    })
}

fn dispatch_accessibility_action(
    cx: &mut TestAppContext,
    action: Action,
    target_node: u64,
) -> TestSupportResult<()> {
    with_shell(cx, |visual_cx, view| {
        let routed = visual_cx.update(|window, app| {
            view.update(app, |shell, view_cx| {
                shell.handle_accesskit_action_request_for_tests(
                    &ActionRequest {
                        action,
                        target_tree: TreeId::ROOT,
                        target_node: accesskit::NodeId(target_node),
                        data: None,
                    },
                    window,
                    view_cx,
                )
            })
        });
        visual_cx.run_until_parked();
        A11Y_STATE.with(|cell| cell.borrow_mut().routed = Some(routed));
        Ok(())
    })
}

#[when("the close button is clicked through accessibility")]
fn click_accessibility_close(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    dispatch_accessibility_action(cx, Action::Click, accessibility::node_ids::CLOSE_BUTTON)
}

#[when("the close button receives an unsupported accessibility action")]
fn send_unsupported_accessibility_action(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    dispatch_accessibility_action(cx, Action::Focus, accessibility::node_ids::CLOSE_BUTTON)
}

#[when("an unknown accessibility node is clicked")]
fn click_unknown_accessibility_node(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    dispatch_accessibility_action(cx, Action::Click, 0xbeef)
}

fn expect_chrome_button(
    update: &TreeUpdate,
    expected: &accessibility::ChromeButtonSemantics,
) -> TestSupportResult<()> {
    let node =
        accessibility::chrome_node_from_update(update, expected.node_id).ok_or_else(|| {
            TestSupportError::missing(
                "accessibility node",
                format!("node {:#x}", expected.node_id),
            )
        })?;
    expect_equal(&node.role(), &Role::Button, "chrome button role")?;
    expect_equal(&node.label(), &Some(expected.label), "chrome button label")?;
    expect_equal(
        &node.description(),
        &Some(expected.shortcut_hint),
        "chrome button description",
    )?;
    expect_equal(
        &node.keyboard_shortcut(),
        &Some(expected.shortcut_hint),
        "chrome button shortcut",
    )?;
    expect_true(node.supports_action(Action::Click), "button supports click")
}

#[then("one initial accessibility tree update is emitted")]
fn one_initial_tree_update_is_emitted(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let records = visual_cx.read(|app| view.read(app).a11y_update_records_for_tests().to_vec());
        let updates = visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| shell.drain_a11y_tree_updates_for_tests())
        });
        expect_equal(&records.len(), &1, "initial accessibility record count")?;
        let record = records
            .first()
            .ok_or_else(|| TestSupportError::missing("initial record", "first draw"))?;
        expect_equal(
            &record.kind,
            &A11yUpdateKind::InitialTree,
            "initial record kind",
        )?;
        expect_true(
            record
                .inserted_node_ids
                .contains(&accessibility::node_ids::TITLEBAR),
            "initial record includes titlebar",
        )?;
        expect_equal(&updates.len(), &1, "serialized initial update count")?;
        let update = updates
            .first()
            .ok_or_else(|| TestSupportError::missing("serialized update", "first draw"))?;
        expect_true(
            update.tree.is_some(),
            "initial update includes tree metadata",
        )?;
        let titlebar =
            accessibility::chrome_node_from_update(update, accessibility::node_ids::TITLEBAR)
                .ok_or_else(|| TestSupportError::missing("titlebar node", "initial update"))?;
        expect_equal(&titlebar.role(), &Role::TitleBar, "titlebar role")?;
        expect_equal(
            &titlebar.label(),
            &Some(accessibility::accessible_names::TITLEBAR),
            "titlebar label",
        )?;
        for expected in accessibility::chrome_button_semantics(false) {
            expect_chrome_button(update, &expected)?;
        }
        Ok(())
    })
}

#[then("one incremental accessibility update includes the inserted shape")]
fn incremental_update_includes_shape(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    let inserted_id = A11Y_STATE
        .with(|cell| cell.borrow().inserted_shape_id)
        .ok_or_else(|| TestSupportError::missing("inserted shape id", "shape insertion"))?;
    with_shell(cx, |visual_cx, view| {
        let records = visual_cx.read(|app| view.read(app).a11y_update_records_for_tests().to_vec());
        expect_equal(&records.len(), &1, "incremental update count")?;
        let record = records
            .first()
            .ok_or_else(|| TestSupportError::missing("incremental record", "shape insertion"))?;
        expect_equal(
            &record.kind,
            &A11yUpdateKind::Incremental,
            "incremental record kind",
        )?;
        expect_true(
            record.inserted_node_ids.contains(&inserted_id),
            "incremental record includes inserted shape",
        )
    })
}

#[then("no accessibility update is emitted")]
fn no_accessibility_update_is_emitted(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    with_shell(cx, |visual_cx, view| {
        let count = visual_cx.read(|app| view.read(app).a11y_pending_update_count_for_tests());
        expect_equal(&count, &0, "pending accessibility update count")
    })
}

#[then("the shell requests quit")]
fn shell_requests_quit(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    expect_quit(cx, true)
}

#[then("the shell does not request quit")]
fn shell_does_not_request_quit(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) -> Result<(), TestSupportError> {
    expect_quit(cx, false)
}

fn expect_quit(cx: &mut TestAppContext, expected: bool) -> TestSupportResult<()> {
    let actual = shell_did_request_quit!(cx)?;
    expect_equal(&actual, &expected, "shell quit request")
}

#[then("the accessibility request routes to close the window")]
fn request_routes_to_close_window() -> Result<(), TestSupportError> {
    expect_routed(&Ok(A11yRequestedAction::Window(
        A11yWindowAction::CloseWindow,
    )))
}

#[then("the accessibility request is rejected as unsupported")]
fn request_is_rejected_as_unsupported() -> Result<(), TestSupportError> {
    expect_routed(&Err(A11yActionRequestError::UnsupportedAction {
        target_node: accessibility::node_ids::CLOSE_BUTTON,
        action: Action::Focus,
    }))
}

#[then("the accessibility request is rejected as an unknown node")]
fn request_is_rejected_as_unknown_node() -> Result<(), TestSupportError> {
    expect_routed(&Err(A11yActionRequestError::UnknownNode {
        target_node: 0xbeef,
    }))
}

fn expect_routed(
    expected: &Result<A11yRequestedAction, A11yActionRequestError>,
) -> TestSupportResult<()> {
    A11Y_STATE.with(|cell| {
        let state = cell.borrow();
        let actual = state.routed.as_ref().ok_or_else(|| {
            TestSupportError::missing("routed accessibility result", "action dispatch")
        })?;
        expect_equal(actual, expected, "accessibility route")
    })
}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Initial accessibility tree is emitted",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn initial_tree(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Shape insertion emits an incremental accessibility update",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn shape_insert(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Idle rendering emits no accessibility update",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn idle_render(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Stale shape selection emits no accessibility update",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn stale_selection(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Close-window action requests quit",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn close_action(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Accessibility close click requests quit",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn a11y_close(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Unsupported accessibility action is rejected",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn unsupported_action(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}

#[scenario(
    path = "tests/features/shell_a11y_service.feature",
    name = "Unknown accessibility node is rejected",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn unknown_node(#[from(support::scenario_state_cleanup)] _cleanup: ScenarioStateCleanup) {}
