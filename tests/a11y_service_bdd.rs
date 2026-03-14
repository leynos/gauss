//! Behaviour tests for the Phase 0 accessibility tree service.

#[path = "a11y_service_bdd/support.rs"]
mod a11y_service_bdd_support;

use accesskit::{Action, Role};
use gauss::model::ShapeId;
use gauss::ui::phase0_shell::{A11yServiceError, A11yShapeSnapshot, A11yUpdateKind, accessibility};
use rstest_bdd_macros::{given, scenario, then, when};

use a11y_service_bdd_support::*;

const APPENDED_SHAPE_ID: u64 = 0x2_0000_0001;

fn shape_snapshot(raw_id: u64) -> A11yShapeSnapshot {
    A11yShapeSnapshot {
        id: ShapeId::from_accesskit_node_id(raw_id),
        name: Some(format!("Shape {raw_id:#x}")),
        locked: false,
        hidden: false,
    }
}

fn chrome_node<'a>(
    world: &'a A11yWorld,
    node_id: u64,
    context: &str,
) -> TestSupportResult<&'a accesskit::Node> {
    accessibility::chrome_node_from_map(&world.last_emitted_nodes, node_id)
        .ok_or_else(|| TestSupportError::missing("emitted accessibility node", context))
}

fn publish_snapshot(world: &mut A11yWorld, context: &str) -> TestSupportResult<()> {
    world.last_publish_result = Some(world.service.sync_snapshot(world.snapshot.clone()));
    if let Some(Err(error)) = world.last_publish_result.as_ref() {
        return Err(TestSupportError::expectation(format!(
            "{context} failed: {error}"
        )));
    }
    capture_last_emitted_nodes(world);
    Ok(())
}

#[given("an initialized accessibility service baseline")]
fn initialized_accessibility_service_baseline(world: &mut A11yWorld) -> TestSupportResult<()> {
    fresh_accessibility_service_snapshot(world);
    world
        .service
        .sync_snapshot(world.snapshot.clone())
        .map_err(|error| {
            TestSupportError::expectation(format!("baseline initialization failed: {error}"))
        })?;
    let _ = world.service.drain_pending_updates();
    world.service.clear_update_records();
    Ok(())
}

#[given("the snapshot marks the window as maximized")]
fn snapshot_marks_window_as_maximized(world: &mut A11yWorld) {
    world.snapshot.is_maximized = true;
}

#[when("I publish the initial accessibility snapshot")]
fn publish_initial_accessibility_snapshot(world: &mut A11yWorld) -> TestSupportResult<()> {
    publish_snapshot(world, "initial accessibility snapshot publish")
}

#[then("one initial accessibility update is queued")]
fn one_initial_accessibility_update_is_queued(world: &A11yWorld) -> TestSupportResult<()> {
    let records = world.service.update_records();
    let first = records
        .first()
        .ok_or_else(|| TestSupportError::missing("update record", "initial queue assertion"))?;
    if records.len() != 1 {
        return Err(TestSupportError::expectation(format!(
            "expected exactly one update record, got {}",
            records.len()
        )));
    }
    if first.kind != A11yUpdateKind::InitialTree {
        return Err(TestSupportError::expectation(format!(
            "expected initial update kind, got {:?}",
            first.kind
        )));
    }
    Ok(())
}

#[then("the maximize node uses the restore label and maximize shortcut hint")]
fn maximize_node_uses_restore_label_and_maximize_shortcut_hint(
    world: &A11yWorld,
) -> TestSupportResult<()> {
    let maximize = chrome_node(
        world,
        accessibility::node_ids::MAXIMIZE_BUTTON,
        "maximize semantics",
    )?;
    if maximize.label() != Some(accessibility::accessible_names::RESTORE) {
        return Err(TestSupportError::expectation(format!(
            "expected maximize node label {:?}, got {:?}",
            accessibility::accessible_names::RESTORE,
            maximize.label()
        )));
    }
    if maximize.keyboard_shortcut() != Some(accessibility::shortcut_hints::MAXIMIZE) {
        return Err(TestSupportError::expectation(format!(
            "expected maximize node keyboard shortcut {:?}, got {:?}",
            accessibility::shortcut_hints::MAXIMIZE,
            maximize.keyboard_shortcut()
        )));
    }
    Ok(())
}

#[then("the update includes titlebar and window control node IDs")]
fn update_includes_chrome_node_ids(world: &A11yWorld) -> TestSupportResult<()> {
    let inserted = &world
        .service
        .update_records()
        .first()
        .ok_or_else(|| TestSupportError::missing("update record", "chrome id assertions"))?
        .inserted_node_ids;
    for expected_id in [
        accessibility::node_ids::TITLEBAR,
        accessibility::node_ids::WINDOW_MENU,
        accessibility::node_ids::MINIMIZE_BUTTON,
        accessibility::node_ids::MAXIMIZE_BUTTON,
        accessibility::node_ids::FULLSCREEN_BUTTON,
        accessibility::node_ids::CLOSE_BUTTON,
    ] {
        if !inserted.contains(&expected_id) {
            return Err(TestSupportError::expectation(format!(
                "expected inserted node IDs to contain {expected_id:#x}, got {inserted:?}"
            )));
        }
    }
    Ok(())
}

fn assert_titlebar_semantics(world: &A11yWorld) -> TestSupportResult<()> {
    let titlebar = chrome_node(
        world,
        accessibility::node_ids::TITLEBAR,
        "titlebar semantics",
    )?;
    if titlebar.role() != Role::TitleBar {
        return Err(TestSupportError::expectation(format!(
            "expected titlebar role {:?}, got {:?}",
            Role::TitleBar,
            titlebar.role()
        )));
    }
    if titlebar.label() != Some(accessibility::accessible_names::TITLEBAR) {
        return Err(TestSupportError::expectation(format!(
            "expected titlebar label {:?}, got {:?}",
            accessibility::accessible_names::TITLEBAR,
            titlebar.label()
        )));
    }
    Ok(())
}

fn assert_chrome_button(
    world: &A11yWorld,
    expected: &accessibility::ChromeButtonSemantics,
) -> TestSupportResult<()> {
    let node = chrome_node(world, expected.node_id, "chrome button semantics")?;
    if node.role() != Role::Button {
        return Err(TestSupportError::expectation(format!(
            "expected node {:#x} role {:?}, got {:?}",
            expected.node_id,
            Role::Button,
            node.role()
        )));
    }
    if node.label() != Some(expected.label) {
        return Err(TestSupportError::expectation(format!(
            "expected node {:#x} label {:?}, got {:?}",
            expected.node_id,
            expected.label,
            node.label()
        )));
    }
    if node.description() != Some(expected.shortcut_hint) {
        return Err(TestSupportError::expectation(format!(
            "expected node {:#x} description {:?}, got {:?}",
            expected.node_id,
            expected.shortcut_hint,
            node.description()
        )));
    }
    if node.keyboard_shortcut() != Some(expected.shortcut_hint) {
        return Err(TestSupportError::expectation(format!(
            "expected node {:#x} keyboard shortcut {:?}, got {:?}",
            expected.node_id,
            expected.shortcut_hint,
            node.keyboard_shortcut()
        )));
    }
    if !node.supports_action(Action::Click) {
        return Err(TestSupportError::expectation(format!(
            "expected node {:#x} to support click action",
            expected.node_id
        )));
    }
    Ok(())
}

#[then("the chrome nodes expose expected roles, labels, and shortcut hints")]
fn chrome_nodes_expose_expected_roles_labels_and_shortcut_hints(
    world: &A11yWorld,
) -> TestSupportResult<()> {
    assert_titlebar_semantics(world)?;
    for expected in accessibility::chrome_button_semantics(world.snapshot.is_maximized) {
        assert_chrome_button(world, &expected)?;
    }
    Ok(())
}

#[when("I append one shape and publish an incremental snapshot")]
fn append_shape_and_publish_incremental_snapshot(world: &mut A11yWorld) -> TestSupportResult<()> {
    world
        .snapshot
        .shapes
        .push(shape_snapshot(APPENDED_SHAPE_ID));
    publish_snapshot(world, "incremental accessibility snapshot publish")
}

#[then("one incremental accessibility update is queued")]
fn one_incremental_accessibility_update_is_queued(world: &A11yWorld) -> TestSupportResult<()> {
    let records = world.service.update_records();
    if records.len() != 1 {
        return Err(TestSupportError::expectation(format!(
            "expected one incremental update, got {} total records",
            records.len()
        )));
    }
    let last = records
        .last()
        .ok_or_else(|| TestSupportError::missing("incremental update", "record check"))?;
    if last.kind != A11yUpdateKind::Incremental {
        return Err(TestSupportError::expectation(format!(
            "expected incremental update kind, got {:?}",
            last.kind
        )));
    }
    Ok(())
}

#[then("the inserted node list contains the appended shape node ID")]
fn inserted_node_list_contains_appended_shape_id(world: &A11yWorld) -> TestSupportResult<()> {
    let last = world
        .service
        .update_records()
        .last()
        .ok_or_else(|| TestSupportError::missing("incremental update", "insert assertion"))?;
    if !last.inserted_node_ids.contains(&APPENDED_SHAPE_ID) {
        return Err(TestSupportError::expectation(format!(
            "expected inserted node IDs {:?} to contain {APPENDED_SHAPE_ID:#x}",
            last.inserted_node_ids
        )));
    }
    Ok(())
}

fn assert_unchanged_snapshot_publish_result(
    result: &Result<bool, A11yServiceError>,
) -> TestSupportResult<()> {
    match result {
        Ok(false) => Ok(()),
        Ok(true) => Err(TestSupportError::expectation(
            "expected sync_snapshot to return false for unchanged input",
        )),
        Err(error) => Err(TestSupportError::expectation(format!(
            "expected sync_snapshot to return false for unchanged input, got error {error}"
        ))),
    }
}

#[when("I publish the same snapshot again")]
fn publish_same_snapshot_again(world: &mut A11yWorld) -> TestSupportResult<()> {
    publish_snapshot(world, "unchanged accessibility snapshot publish")
}

#[then("no new accessibility update is queued")]
fn no_new_accessibility_update_is_queued(world: &A11yWorld) -> TestSupportResult<()> {
    let current_len = world.service.update_records().len();
    if current_len != 0 {
        return Err(TestSupportError::expectation(format!(
            "expected no new records, got {current_len}"
        )));
    }
    let result = world
        .last_publish_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("publish result", "unchanged-input assertion"))?;
    assert_unchanged_snapshot_publish_result(result)
}

#[given("an accessibility snapshot with duplicate shape node IDs")]
fn accessibility_snapshot_with_duplicate_shape_node_ids(world: &mut A11yWorld) {
    fresh_accessibility_service_snapshot(world);
    world
        .snapshot
        .shapes
        .push(shape_snapshot(APPENDED_SHAPE_ID));
    world
        .snapshot
        .shapes
        .push(shape_snapshot(APPENDED_SHAPE_ID));
}

#[when("I publish the duplicate-node accessibility snapshot")]
fn publish_duplicate_node_accessibility_snapshot(world: &mut A11yWorld) {
    world.last_publish_result = Some(world.service.sync_snapshot(world.snapshot.clone()));
    capture_last_emitted_nodes(world);
}

#[then("publishing fails with a duplicate shape node ID error")]
fn publishing_fails_with_duplicate_shape_node_id_error(world: &A11yWorld) -> TestSupportResult<()> {
    let result = world
        .last_publish_result
        .as_ref()
        .ok_or_else(|| TestSupportError::missing("publish result", "collision assertion"))?;
    match result {
        Err(A11yServiceError::DuplicateShapeNodeId { .. }) => Ok(()),
        Ok(value) => Err(TestSupportError::expectation(format!(
            "expected duplicate-node error, got Ok({value})"
        ))),
        Err(other) => Err(TestSupportError::expectation(format!(
            "expected duplicate-node error, got {other}"
        ))),
    }
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Initial accessibility snapshot includes window chrome nodes with stable IDs"
)]
fn initial_accessibility_snapshot_includes_window_chrome_nodes_with_stable_ids(world: A11yWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Maximized window exposes restore semantics on the maximize node"
)]
fn maximized_window_exposes_restore_semantics_on_the_maximize_node(world: A11yWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Adding a shape emits one inserted accessibility node update"
)]
fn adding_a_shape_emits_one_inserted_accessibility_node_update(world: A11yWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Unchanged state emits no accessibility updates"
)]
fn unchanged_state_emits_no_accessibility_updates(world: A11yWorld) {
    let _ = world;
}

#[scenario(
    path = "tests/features/a11y_service.feature",
    name = "Duplicate node ID is reported and update is aborted"
)]
fn duplicate_node_id_is_reported_and_update_is_aborted(world: A11yWorld) {
    let _ = world;
}
