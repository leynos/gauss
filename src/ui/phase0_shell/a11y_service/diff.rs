//! Diff helpers for comparing serialized AccessKit node maps.

use std::collections::BTreeMap;

use accesskit::{Node, NodeId};

/// Clone every node, preserving the key order of the source map.
pub(super) fn clone_nodes_in_order(nodes: &BTreeMap<NodeId, Node>) -> Vec<(NodeId, Node)> {
    nodes.iter().map(|(id, node)| (*id, node.clone())).collect()
}

/// Return the raw IDs present in the old map but absent from the new one.
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

/// Return the raw IDs present in the new map but absent from the old one.
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

/// Return the raw IDs of nodes present in both maps whose values differ.
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

/// Clone the nodes that are new or whose value differs from the old map.
///
/// A node counts as changed when it is absent from the old map or its
/// serialized value is not equal to the previous one.
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
