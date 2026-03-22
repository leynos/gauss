//! AccessKit tree service for the Phase 0 shell.
//!
//! This module owns accessibility tree projection from current shell state and
//! emits incremental `TreeUpdate` values suitable for adapter delivery.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{EdgeMode, ShapeId, ToolMode};
use accesskit::{Node, NodeId, Tree, TreeId, TreeUpdate};
use thiserror::Error;

use super::Phase0Shell;
use diff::{
    changed_nodes, clone_nodes_in_order, inserted_node_ids, removed_node_ids, updated_node_ids,
};
use tree_builder::{ROOT_NODE_ID, build_node_map};

mod action_routing;
mod diff;
#[cfg(test)]
mod tests;
mod tree_builder;

pub use self::action_routing::{A11yActionRequestError, A11yRequestedAction, A11yWindowAction};

const MAX_PENDING_UPDATES: usize = 128;
const MAX_UPDATE_RECORDS: usize = 512;

/// A compact description of an emitted tree update for tests and diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A11yUpdateRecord {
    /// Whether this update initialized the tree or incrementally changed it.
    pub kind: A11yUpdateKind,
    /// Node IDs inserted by this update.
    pub inserted_node_ids: Vec<u64>,
    /// Node IDs updated by this update.
    pub updated_node_ids: Vec<u64>,
    /// Node IDs removed from the previous tree state.
    pub removed_node_ids: Vec<u64>,
    /// Focus node ID included in this update.
    pub focus_node_id: u64,
    /// Number of nodes serialized in `TreeUpdate::nodes`.
    pub nodes_serialized: usize,
}

/// Update classification for emitted AccessKit updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yUpdateKind {
    /// First update that initializes tree metadata.
    InitialTree,
    /// Incremental update after a prior tree has already been published.
    Incremental,
}

/// Errors produced when projecting shell state into an AccessKit tree.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum A11yServiceError {
    /// Shape-derived node IDs collided with reserved chrome IDs.
    #[error("shape node id {node_id:#x} collides with reserved accessibility id space")]
    ReservedNodeIdCollision {
        /// Colliding shape-derived node identifier.
        node_id: u64,
    },
    /// Duplicate node IDs were generated for multiple shapes.
    #[error("shape node id {node_id:#x} appears multiple times in one tree snapshot")]
    DuplicateShapeNodeId {
        /// Duplicate shape-derived node identifier.
        node_id: u64,
    },
}

/// Snapshot input consumed by [`A11yService`] to generate tree updates.
#[derive(Clone, Debug, PartialEq)]
pub struct A11ySnapshot {
    /// Active tool mode reflected in status node text.
    pub tool_mode: ToolMode,
    /// Active edge mode reflected in status node text.
    pub edge_mode: EdgeMode,
    /// Whether undo is currently available.
    pub can_undo: bool,
    /// Whether redo is currently available.
    pub can_redo: bool,
    /// Whether the window is currently maximized.
    pub is_maximized: bool,
    /// Selected shape IDs currently active in the document.
    pub selected_shape_ids: BTreeSet<ShapeId>,
    /// Draw-order shape snapshot entries.
    pub shapes: Vec<A11yShapeSnapshot>,
    /// Localizer for internationalized accessibility strings.
    pub localizer: crate::i18n::Localizer,
    /// Current locale for accessibility strings.
    pub locale: crate::i18n::Locale,
}

/// Shape projection used to generate list nodes in the accessibility tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A11yShapeSnapshot {
    /// Stable shape identifier.
    pub id: ShapeId,
    /// Optional shape label exposed to assistive technologies.
    pub name: Option<String>,
    /// Whether the shape is locked.
    pub locked: bool,
    /// Whether the shape is hidden.
    pub hidden: bool,
}

/// Builds and tracks AccessKit tree updates from shell/document state.
#[derive(Default)]
pub struct A11yService {
    previous_snapshot: Option<A11ySnapshot>,
    previous_nodes: BTreeMap<NodeId, Node>,
    previous_focus: Option<NodeId>,
    pending_updates: Vec<TreeUpdate>,
    update_records: Vec<A11yUpdateRecord>,
}

struct RebaseContext<'a> {
    nodes: &'a BTreeMap<NodeId, Node>,
    focus: NodeId,
}

impl A11yService {
    /// Create a new service with no baseline tree.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return pending serialized updates that should be pushed to adapters.
    pub fn drain_pending_updates(&mut self) -> Vec<TreeUpdate> {
        std::mem::take(&mut self.pending_updates)
    }

    /// Return the number of queued updates awaiting adapter delivery.
    #[must_use]
    pub const fn pending_update_count(&self) -> usize {
        self.pending_updates.len()
    }

    /// Clear queued updates that have not yet been delivered to adapters.
    pub fn clear_pending_updates(&mut self) {
        self.pending_updates.clear();
    }

    /// Return diagnostic records for already-queued updates.
    #[must_use]
    pub fn update_records(&self) -> &[A11yUpdateRecord] {
        &self.update_records
    }

    /// Clear diagnostic update records.
    pub fn clear_update_records(&mut self) {
        self.update_records.clear();
    }

    /// Build and queue an update from the latest shell snapshot.
    ///
    /// Returns `Ok(true)` when a new update is queued, `Ok(false)` when there
    /// is no semantic accessibility change.
    ///
    /// # Errors
    ///
    /// Returns [`A11yServiceError`] when shape IDs collide with reserved chrome
    /// IDs or when duplicate shape IDs are observed in one snapshot.
    pub fn sync_from_shell(&mut self, shell: &Phase0Shell) -> Result<bool, A11yServiceError> {
        let snapshot = snapshot_from_shell(shell);
        self.sync_snapshot(snapshot)
    }

    /// Build and queue an update from a prepared snapshot.
    ///
    /// Returns `Ok(true)` when a new update is queued, `Ok(false)` when there
    /// is no semantic accessibility change.
    ///
    /// # Errors
    ///
    /// Returns [`A11yServiceError`] when shape IDs collide with reserved chrome
    /// IDs or when duplicate shape IDs are observed in one snapshot.
    pub fn sync_snapshot(&mut self, snapshot: A11ySnapshot) -> Result<bool, A11yServiceError> {
        let (nodes, focus) = build_node_map(&snapshot)?;

        if self.previous_snapshot.is_none() {
            self.emit_initial_snapshot(snapshot, nodes, focus);
            return Ok(true);
        }

        if self.previous_snapshot.as_ref() == Some(&snapshot) && self.previous_focus == Some(focus)
        {
            return Ok(false);
        }

        Ok(self.emit_incremental_snapshot(snapshot, nodes, focus))
    }

    fn emit_initial_snapshot(
        &mut self,
        snapshot: A11ySnapshot,
        nodes: BTreeMap<NodeId, Node>,
        focus: NodeId,
    ) {
        let initial_update = TreeUpdate {
            nodes: clone_nodes_in_order(&nodes),
            tree: Some(Tree::new(ROOT_NODE_ID)),
            tree_id: TreeId::ROOT,
            focus,
        };
        self.store_emitted_update(
            initial_update,
            A11yUpdateRecord {
                kind: A11yUpdateKind::InitialTree,
                inserted_node_ids: nodes.keys().map(|id| id.0).collect(),
                updated_node_ids: Vec::new(),
                removed_node_ids: Vec::new(),
                focus_node_id: focus.0,
                nodes_serialized: nodes.len(),
            },
            &RebaseContext {
                nodes: &nodes,
                focus,
            },
        );
        self.set_previous_state(snapshot, nodes, focus);
    }

    fn emit_incremental_snapshot(
        &mut self,
        snapshot: A11ySnapshot,
        nodes: BTreeMap<NodeId, Node>,
        focus: NodeId,
    ) -> bool {
        let removed_node_ids = removed_node_ids(&self.previous_nodes, &nodes);
        let changed_nodes = changed_nodes(&self.previous_nodes, &nodes);
        let update = TreeUpdate {
            nodes: changed_nodes,
            tree: None,
            tree_id: TreeId::ROOT,
            focus,
        };
        let nodes_serialized = update.nodes.len();
        let inserted_ids = inserted_node_ids(&self.previous_nodes, &nodes);
        let updated_ids = updated_node_ids(&self.previous_nodes, &nodes);

        let has_no_node_deltas = removed_node_ids.is_empty() && nodes_serialized == 0;
        if has_no_node_deltas && self.previous_focus == Some(focus) {
            self.set_previous_state(snapshot, nodes, focus);
            return false;
        }

        self.store_emitted_update(
            update,
            A11yUpdateRecord {
                kind: A11yUpdateKind::Incremental,
                inserted_node_ids: inserted_ids,
                updated_node_ids: updated_ids,
                removed_node_ids,
                focus_node_id: focus.0,
                nodes_serialized,
            },
            &RebaseContext {
                nodes: &nodes,
                focus,
            },
        );
        self.set_previous_state(snapshot, nodes, focus);
        true
    }

    fn store_emitted_update(
        &mut self,
        update: TreeUpdate,
        record: A11yUpdateRecord,
        rebase_context: &RebaseContext<'_>,
    ) {
        self.pending_updates.push(update);
        if self.pending_updates.len() > MAX_PENDING_UPDATES {
            // Rebase queued deltas to a single full snapshot instead of silently
            // dropping delivery-critical updates.
            let rebase_update = TreeUpdate {
                nodes: clone_nodes_in_order(rebase_context.nodes),
                tree: Some(Tree::new(ROOT_NODE_ID)),
                tree_id: TreeId::ROOT,
                focus: rebase_context.focus,
            };
            self.pending_updates.clear();
            self.pending_updates.push(rebase_update);
        }
        self.update_records.push(record);
        truncate_oldest(&mut self.update_records, MAX_UPDATE_RECORDS);
    }

    fn set_previous_state(
        &mut self,
        snapshot: A11ySnapshot,
        nodes: BTreeMap<NodeId, Node>,
        focus: NodeId,
    ) {
        self.previous_snapshot = Some(snapshot);
        self.previous_nodes = nodes;
        self.previous_focus = Some(focus);
    }

    #[cfg(test)]
    fn sync_from_shell_like(&mut self, snapshot: A11ySnapshot) -> Result<bool, A11yServiceError> {
        self.sync_snapshot(snapshot)
    }
}

fn snapshot_from_shell(shell: &Phase0Shell) -> A11ySnapshot {
    let shapes = shell
        .state
        .document
        .iter_in_draw_order()
        .map(|shape| A11yShapeSnapshot {
            id: shape.id,
            name: shape.name.clone(),
            locked: shape.locked,
            hidden: shape.hidden,
        })
        .collect::<Vec<_>>();
    let valid_shape_ids = shapes.iter().map(|shape| shape.id).collect::<BTreeSet<_>>();
    let selected_shape_ids = shell
        .state
        .selection
        .selected_shapes()
        .filter(|id| valid_shape_ids.contains(id))
        .collect::<BTreeSet<_>>();

    A11ySnapshot {
        tool_mode: shell.state.tool_mode,
        edge_mode: shell.state.edge_mode,
        can_undo: shell.state.can_undo_document(),
        can_redo: shell.state.can_redo_document(),
        is_maximized: shell.last_maximized_state == Some(true),
        selected_shape_ids,
        shapes,
        localizer: shell.localizer.clone(),
        locale: shell.locale.clone(),
    }
}

fn truncate_oldest<T>(items: &mut Vec<T>, max_len: usize) {
    let overflow = items.len().saturating_sub(max_len);
    if overflow > 0 {
        items.drain(..overflow);
    }
}

impl Phase0Shell {
    /// Recompute and queue accessibility updates from current shell state.
    pub(super) fn sync_a11y_tree(&mut self) {
        let snapshot = snapshot_from_shell(self);
        if let Err(error) = self.a11y_service.sync_snapshot(snapshot) {
            log::error!("{error}");
        }
    }
}
