//! Unit tests for AccessKit tree projection and incremental updates.

use accesskit::NodeId;
use rstest::rstest;

use crate::model::{EdgeMode, ShapeId, ToolMode};

use super::tree_builder::build_node_map;
use super::{A11yService, A11yServiceError, A11yShapeSnapshot, A11ySnapshot, A11yUpdateKind};

const SHAPE_LIST_NODE_ID_RAW: u64 = 0x1008;

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

fn snapshot(shape_ids: &[u64], selected_ids: &[u64]) -> A11ySnapshot {
    let shapes = shape_ids
        .iter()
        .enumerate()
        .map(|(index, id)| A11yShapeSnapshot {
            id: shape_id(*id),
            name: Some(format!("Shape {}", index + 1)),
            locked: false,
            hidden: false,
        })
        .collect::<Vec<_>>();
    let selected_shape_ids = selected_ids.iter().map(|id| shape_id(*id)).collect();
    A11ySnapshot {
        tool_mode: ToolMode::Draw,
        edge_mode: EdgeMode::Line,
        can_undo: false,
        can_redo: false,
        is_maximized: false,
        selected_shape_ids,
        shapes,
    }
}

#[rstest]
fn first_sync_builds_initial_tree_and_record() {
    let mut service = A11yService::new();
    let first = snapshot(&[0x2_0000_0001], &[0x2_0000_0001]);
    let queued = service
        .sync_from_shell_like(first)
        .expect("initial sync should not fail");
    assert!(queued);
    assert_eq!(service.pending_updates.len(), 1);
    assert_eq!(service.update_records.len(), 1);
    let first_record = service
        .update_records
        .first()
        .expect("initial update record should be present");
    assert_eq!(first_record.kind, A11yUpdateKind::InitialTree);
    assert!(
        first_record
            .inserted_node_ids
            .contains(&SHAPE_LIST_NODE_ID_RAW)
    );
}

#[rstest]
fn unchanged_snapshot_is_noop() {
    let mut service = A11yService::new();
    let first = snapshot(&[0x2_0000_0001], &[]);
    service
        .sync_from_shell_like(first.clone())
        .expect("initial sync should not fail");
    let queued = service
        .sync_from_shell_like(first)
        .expect("repeat sync should not fail");
    assert!(!queued);
    assert_eq!(service.pending_updates.len(), 1);
    assert_eq!(service.update_records.len(), 1);
}

#[rstest]
fn adding_shape_emits_incremental_insert_and_parent_update() {
    let mut service = A11yService::new();
    service
        .sync_from_shell_like(snapshot(&[0x2_0000_0001], &[]))
        .expect("initial sync should not fail");
    let queued = service
        .sync_from_shell_like(snapshot(&[0x2_0000_0001, 0x2_0000_0002], &[0x2_0000_0002]))
        .expect("second sync should not fail");
    assert!(queued);
    assert_eq!(service.pending_updates.len(), 2);
    let record = service
        .update_records
        .last()
        .expect("incremental record should be present");
    let inserted_shape_node_id = shape_id(0x2_0000_0002).to_accesskit_node_id();
    assert_eq!(record.kind, A11yUpdateKind::Incremental);
    assert!(record.inserted_node_ids.contains(&inserted_shape_node_id));
    assert!(record.updated_node_ids.contains(&SHAPE_LIST_NODE_ID_RAW));
}

#[rstest]
fn removing_shape_emits_removed_node_id() {
    let mut service = A11yService::new();
    service
        .sync_from_shell_like(snapshot(&[0x2_0000_0001, 0x2_0000_0002], &[]))
        .expect("initial sync should not fail");
    let queued = service
        .sync_from_shell_like(snapshot(&[0x2_0000_0001], &[]))
        .expect("second sync should not fail");
    assert!(queued);
    let record = service
        .update_records
        .last()
        .expect("incremental record should be present");
    let removed_shape_node_id = shape_id(0x2_0000_0002).to_accesskit_node_id();
    assert!(record.removed_node_ids.contains(&removed_shape_node_id));
}

#[rstest]
fn duplicate_shape_ids_return_error() {
    let duplicate_id = 0x2_0000_0001;
    let duplicate_shape_node_id = shape_id(duplicate_id).to_accesskit_node_id();
    let result = build_node_map(&snapshot(&[duplicate_id, duplicate_id], &[]));
    assert_eq!(
        result,
        Err(A11yServiceError::DuplicateShapeNodeId {
            node_id: duplicate_shape_node_id
        })
    );
}

#[rstest]
fn node_map_focuses_canvas_when_selected_shape_is_missing() {
    let snapshot = snapshot(&[0x2_0000_0001], &[0x2_0000_0002]);
    let (_, focus) = build_node_map(&snapshot).expect("node map build should succeed");
    assert_eq!(focus, NodeId(0x1007));
}
