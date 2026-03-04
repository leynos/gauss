//! Unit tests for AccessKit tree projection and incremental updates.

use accesskit::{Action, NodeId, Role};
use rstest::rstest;

use crate::model::{EdgeMode, ShapeId, ToolMode};
use crate::ui::phase0_shell::accessibility;

use super::tree_builder::{CANVAS_NODE_ID, SHAPE_LIST_NODE_ID, build_node_map};
use super::{A11yService, A11yServiceError, A11yShapeSnapshot, A11ySnapshot, A11yUpdateKind};

fn shape_id(raw: u64) -> ShapeId {
    ShapeId::from_accesskit_node_id(raw)
}

fn snapshot(shape_ids: &[u64], selected_ids: &[u64]) -> A11ySnapshot {
    snapshot_with_state(shape_ids, selected_ids, false)
}

fn snapshot_with_state(
    shape_ids: &[u64],
    selected_ids: &[u64],
    is_maximized: bool,
) -> A11ySnapshot {
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
        is_maximized,
        selected_shape_ids,
        shapes,
    }
}

#[rstest]
fn titlebar_node_uses_stable_role_label_and_children_order() {
    let (nodes, _) = build_node_map(&snapshot(&[], &[])).expect("node map build should succeed");
    let Some(titlebar) =
        accessibility::chrome_node_from_map(&nodes, accessibility::node_ids::TITLEBAR)
    else {
        panic!(
            "expected node map to contain chrome node id {:#x}",
            accessibility::node_ids::TITLEBAR
        );
    };
    let expected_children = accessibility::chrome_button_semantics(false)
        .into_iter()
        .map(|button| NodeId(button.node_id))
        .collect::<Vec<_>>();
    assert_eq!(titlebar.role(), Role::TitleBar);
    assert_eq!(
        titlebar.label(),
        Some(accessibility::accessible_names::TITLEBAR)
    );
    assert_eq!(titlebar.children().to_vec(), expected_children);
}

#[rstest]
fn chrome_button_nodes_expose_role_label_hint_and_click_action() {
    let (nodes, _) = build_node_map(&snapshot(&[], &[])).expect("node map build should succeed");
    for expected in accessibility::chrome_button_semantics(false) {
        let Some(node) = accessibility::chrome_node_from_map(&nodes, expected.node_id) else {
            panic!(
                "expected node map to contain chrome node id {:#x}",
                expected.node_id
            );
        };
        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.label(), Some(expected.label));
        assert_eq!(node.description(), Some(expected.shortcut_hint));
        assert_eq!(node.keyboard_shortcut(), Some(expected.shortcut_hint));
        assert!(node.supports_action(Action::Click));
    }
}

#[rstest]
#[case(false, accessibility::accessible_names::MAXIMIZE)]
#[case(true, accessibility::accessible_names::RESTORE)]
fn maximize_button_label_matches_window_state(
    #[case] is_maximized: bool,
    #[case] expected_label: &'static str,
) {
    let (nodes, _) = build_node_map(&snapshot_with_state(&[], &[], is_maximized))
        .expect("node map build should succeed");
    let Some(maximize_button) =
        accessibility::chrome_node_from_map(&nodes, accessibility::node_ids::MAXIMIZE_BUTTON)
    else {
        panic!(
            "expected node map to contain chrome node id {:#x}",
            accessibility::node_ids::MAXIMIZE_BUTTON
        );
    };
    assert_eq!(maximize_button.label(), Some(expected_label));
    assert_eq!(
        maximize_button.keyboard_shortcut(),
        Some(accessibility::shortcut_hints::MAXIMIZE)
    );
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
            .contains(&SHAPE_LIST_NODE_ID.0)
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
    assert!(record.updated_node_ids.contains(&SHAPE_LIST_NODE_ID.0));
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
#[case(0x1001)]
#[case(0x2_0000_0001)]
fn shape_snapshot_ids_stay_outside_reserved_chrome_id_space(#[case] raw_shape_id: u64) {
    let projected_shape_node_id = shape_id(raw_shape_id).to_accesskit_node_id();
    let reserved_chrome_node_ids = accessibility::chrome_button_semantics(false)
        .into_iter()
        .map(|button| button.node_id)
        .chain(std::iter::once(accessibility::node_ids::TITLEBAR))
        .collect::<Vec<_>>();
    assert!(
        !reserved_chrome_node_ids.contains(&projected_shape_node_id),
        "shape node id {projected_shape_node_id:#x} must not overlap reserved chrome ids"
    );
}

#[rstest]
fn node_map_focuses_canvas_when_selected_shape_is_missing() {
    let snapshot = snapshot(&[0x2_0000_0001], &[0x2_0000_0002]);
    let (_, focus) = build_node_map(&snapshot).expect("node map build should succeed");
    assert_eq!(focus, CANVAS_NODE_ID);
}

#[rstest]
fn emitted_updates_are_bounded_to_prevent_unbounded_growth() {
    let mut service = A11yService::new();
    let mut baseline = snapshot(&[0x2_0000_0001], &[]);
    service
        .sync_from_shell_like(baseline.clone())
        .expect("initial sync should succeed");

    for iteration in 0_u32..700 {
        baseline.is_maximized = iteration.is_multiple_of(2);
        service
            .sync_from_shell_like(baseline.clone())
            .expect("toggle sync should succeed");
    }

    assert!(service.pending_update_count() <= super::MAX_PENDING_UPDATES);
    assert_eq!(service.update_records().len(), super::MAX_UPDATE_RECORDS);
    assert!(
        service
            .pending_updates
            .iter()
            .any(|update| update.tree.is_some()),
        "expected overflow handling to retain a full-tree rebase update"
    );
}
