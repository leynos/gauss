//! Diff helpers for comparing serialized AccessKit node maps.

use std::collections::BTreeMap;

use accesskit::{Node, NodeId};

/// Clone each node in ascending ID order.
/// For IDs 2 and 9, the returned pairs remain ordered as 2 then 9.
pub(super) fn clone_nodes_in_order(nodes: &BTreeMap<NodeId, Node>) -> Vec<(NodeId, Node)> {
    nodes.iter().map(|(id, node)| (*id, node.clone())).collect()
}

/// Return old IDs missing from the new map; old `{1, 2}` and new `{2, 3}` yield `[1]`.
pub(super) fn removed_node_ids(
    old_nodes: &BTreeMap<NodeId, Node>,
    new_nodes: &BTreeMap<NodeId, Node>,
) -> Vec<u64> {
    old_nodes
        .keys()
        .filter(|id| !new_nodes.contains_key(id))
        .map(|id| id.0)
        .collect()
}

/// Return IDs newly present in the new map; old `{1, 2}` and new `{2, 3}` yield `[3]`.
pub(super) fn inserted_node_ids(
    old_nodes: &BTreeMap<NodeId, Node>,
    new_nodes: &BTreeMap<NodeId, Node>,
) -> Vec<u64> {
    new_nodes
        .keys()
        .filter(|id| !old_nodes.contains_key(id))
        .map(|id| id.0)
        .collect()
}

/// Return IDs present in both maps whose AccessKit node data changed.
pub(super) fn updated_node_ids(
    old_nodes: &BTreeMap<NodeId, Node>,
    new_nodes: &BTreeMap<NodeId, Node>,
) -> Vec<u64> {
    new_nodes
        .iter()
        .filter_map(|(id, node)| {
            old_nodes
                .get(id)
                .filter(|old_node| *old_node != node)
                .map(|_| id.0)
        })
        .collect()
}

/// Return inserted or changed nodes from the new map; removed nodes have no returned payload.
pub(super) fn changed_nodes(
    old_nodes: &BTreeMap<NodeId, Node>,
    new_nodes: &BTreeMap<NodeId, Node>,
) -> Vec<(NodeId, Node)> {
    new_nodes
        .iter()
        .filter_map(|(id, node)| match old_nodes.get(id) {
            Some(old_node) if old_node == node => None,
            _ => Some((*id, node.clone())),
        })
        .collect()
}
