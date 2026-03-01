//! Behaviour tests for the Phase 0 accessibility tree service.

use std::collections::BTreeSet;

use gauss::model::{EdgeMode, ShapeId, ToolMode};
use gauss::ui::phase0_shell::{
    A11yService, A11yServiceError, A11yShapeSnapshot, A11ySnapshot, A11yUpdateKind,
};
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use test_support::{TestSupportError, TestSupportResult};

const TITLEBAR_NODE_ID: u64 = 0x1006;
const MINIMIZE_NODE_ID: u64 = 0x1001;
const MAXIMIZE_NODE_ID: u64 = 0x1002;
const CLOSE_NODE_ID: u64 = 0x1003;

struct A11yWorld {
    service: A11yService,
    snapshot: A11ySnapshot,
    appended_shape_id: Option<u64>,
    last_publish_result: Option<Result<bool, A11yServiceError>>,
    update_count_before: usize,
}

#[fixture]
fn world() -> A11yWorld {
    A11yWorld {
        service: A11yService::new(),
        snapshot: empty_snapshot(),
        appended_shape_id: None,
        last_publish_result: None,
        update_count_before: 0,
    }
}

const fn empty_snapshot() -> A11ySnapshot {
    A11ySnapshot {
        tool_mode: ToolMode::Draw,
        edge_mode: EdgeMode::Line,
        can_undo: false,
        can_redo: false,
        is_maximized: false,
        selected_shape_ids: BTreeSet::new(),
        shapes: Vec::new(),
    }
}

fn shape_snapshot(raw_id: u64) -> A11yShapeSnapshot {
    A11yShapeSnapshot {
        id: ShapeId::from_accesskit_node_id(raw_id),
        name: Some(format!("Shape {raw_id:#x}")),
        locked: false,
        hidden: false,
    }
}

#[given("a fresh accessibility service snapshot")]
fn fresh_accessibility_service_snapshot(world: &mut A11yWorld) {
    world.service = A11yService::new();
    world.snapshot = empty_snapshot();
    world.last_publish_result = None;
    world.appended_shape_id = None;
}

#[when("I publish the initial accessibility snapshot")]
fn publish_initial_accessibility_snapshot(world: &mut A11yWorld) {
    world.last_publish_result = Some(world.service.sync_snapshot(world.snapshot.clone()));
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

#[then("the update includes titlebar and window control node IDs")]
fn update_includes_chrome_node_ids(world: &A11yWorld) -> TestSupportResult<()> {
    let inserted = &world
        .service
        .update_records()
        .first()
        .ok_or_else(|| TestSupportError::missing("update record", "chrome id assertions"))?
        .inserted_node_ids;
    for expected_id in [
        TITLEBAR_NODE_ID,
        MINIMIZE_NODE_ID,
        MAXIMIZE_NODE_ID,
        CLOSE_NODE_ID,
    ] {
        if !inserted.contains(&expected_id) {
            return Err(TestSupportError::expectation(format!(
                "expected inserted node IDs to contain {expected_id:#x}, got {inserted:?}"
            )));
        }
    }
    Ok(())
}

#[given("an initialized accessibility service baseline")]
fn initialized_accessibility_service_baseline(world: &mut A11yWorld) -> TestSupportResult<()> {
    world.service = A11yService::new();
    world.snapshot = empty_snapshot();
    world
        .service
        .sync_snapshot(world.snapshot.clone())
        .map_err(|error| {
            TestSupportError::expectation(format!("baseline initialization failed: {error}"))
        })?;
    world.service.clear_update_records();
    world.update_count_before = world.service.update_records().len();
    Ok(())
}

#[when("I append one shape and publish an incremental snapshot")]
fn append_shape_and_publish_incremental_snapshot(world: &mut A11yWorld) {
    let appended_shape_id = 0x2_0000_0001;
    world
        .snapshot
        .shapes
        .push(shape_snapshot(appended_shape_id));
    world.appended_shape_id = Some(appended_shape_id);
    world.last_publish_result = Some(world.service.sync_snapshot(world.snapshot.clone()));
}

#[then("one incremental accessibility update is queued")]
fn one_incremental_accessibility_update_is_queued(world: &A11yWorld) -> TestSupportResult<()> {
    let records = world.service.update_records();
    if records.len() != world.update_count_before + 1 {
        return Err(TestSupportError::expectation(format!(
            "expected one incremental update, got {} total records (before {})",
            records.len(),
            world.update_count_before
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
    let expected_shape_id = world
        .appended_shape_id
        .ok_or_else(|| TestSupportError::missing("appended shape id", "insert assertion"))?;
    let last = world
        .service
        .update_records()
        .last()
        .ok_or_else(|| TestSupportError::missing("incremental update", "insert assertion"))?;
    if !last.inserted_node_ids.contains(&expected_shape_id) {
        return Err(TestSupportError::expectation(format!(
            "expected inserted node IDs {:?} to contain {expected_shape_id:#x}",
            last.inserted_node_ids
        )));
    }
    Ok(())
}

#[when("I publish the same snapshot again")]
fn publish_same_snapshot_again(world: &mut A11yWorld) {
    world.last_publish_result = Some(world.service.sync_snapshot(world.snapshot.clone()));
}

#[then("no new accessibility update is queued")]
fn no_new_accessibility_update_is_queued(world: &A11yWorld) -> TestSupportResult<()> {
    let current_len = world.service.update_records().len();
    if current_len != world.update_count_before {
        return Err(TestSupportError::expectation(format!(
            "expected no new records, before {}, after {}",
            world.update_count_before, current_len
        )));
    }
    if let Some(Ok(queued)) = world.last_publish_result.as_ref()
        && *queued
    {
        return Err(TestSupportError::expectation(
            "expected sync_snapshot to return false for unchanged input",
        ));
    }
    Ok(())
}

#[given("an accessibility snapshot with duplicate shape node IDs")]
fn accessibility_snapshot_with_duplicate_shape_node_ids(world: &mut A11yWorld) {
    world.service = A11yService::new();
    world.snapshot = empty_snapshot();
    let duplicate_raw_id = 0x2_0000_0001;
    world.snapshot.shapes.push(shape_snapshot(duplicate_raw_id));
    world.snapshot.shapes.push(shape_snapshot(duplicate_raw_id));
}

#[when("I publish the duplicate-node accessibility snapshot")]
fn publish_duplicate_node_accessibility_snapshot(world: &mut A11yWorld) {
    world.last_publish_result = Some(world.service.sync_snapshot(world.snapshot.clone()));
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
