//! Diff helpers for comparing serialized AccessKit node maps.

use std::collections::BTreeMap;

use accesskit::{Node, NodeId};

pub(super) fn clone_nodes_in_order(nodes: &BTreeMap<NodeId, Node>) -> Vec<(NodeId, Node)> {
    nodes.iter().map(|(id, node)| (*id, node.clone())).collect()
}

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
