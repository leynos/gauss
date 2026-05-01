//! AccessKit tree construction for the Phase 0 shell accessibility snapshot.

use std::collections::{BTreeMap, BTreeSet};

use accesskit::{Action, Node, NodeId, Role};

use crate::i18n::{Localizer, MessageId};
use crate::ui::phase0_shell::accessibility;
use crate::ui::phase0_shell::i18n_helpers::localized_status_label;

use super::{A11yServiceError, A11ySnapshot};

pub(super) const ROOT_NODE_ID: NodeId = NodeId(0x1000);
pub(super) const CANVAS_NODE_ID: NodeId = NodeId(0x1007);
pub(super) const SHAPE_LIST_NODE_ID: NodeId = NodeId(0x1008);
pub(super) const STATUS_NODE_ID: NodeId = NodeId(0x1009);

const RESERVED_NODE_IDS: [u64; 10] = [
    ROOT_NODE_ID.0,
    accessibility::node_ids::MINIMIZE_BUTTON,
    accessibility::node_ids::MAXIMIZE_BUTTON,
    accessibility::node_ids::CLOSE_BUTTON,
    accessibility::node_ids::FULLSCREEN_BUTTON,
    accessibility::node_ids::WINDOW_MENU,
    accessibility::node_ids::TITLEBAR,
    CANVAS_NODE_ID.0,
    SHAPE_LIST_NODE_ID.0,
    STATUS_NODE_ID.0,
];

/// Construct the full AccessKit node map and focus node for a snapshot.
///
/// The returned map contains window chrome, canvas, status, and shape-list
/// nodes. The focus node is selected from the current shape selection when
/// possible, falling back to the canvas node.
pub(super) fn build_node_map(
    snapshot: &A11ySnapshot,
) -> Result<(BTreeMap<NodeId, Node>, NodeId), A11yServiceError> {
    let mut nodes = BTreeMap::new();
    insert_chrome_nodes(&mut nodes, snapshot.is_maximized);
    insert_canvas_and_status_nodes(&mut nodes, snapshot);
    insert_shape_list_nodes(&mut nodes, snapshot)?;
    insert_root_node(&mut nodes, snapshot);
    let focus = resolve_focus_node(snapshot, &nodes);
    Ok((nodes, focus))
}

fn insert_chrome_nodes(nodes: &mut BTreeMap<NodeId, Node>, is_maximized: bool) {
    let chrome_buttons = accessibility::chrome_button_semantics(is_maximized);
    let mut titlebar = Node::new(Role::TitleBar);
    titlebar.set_label(accessibility::accessible_names::TITLEBAR);
    titlebar.set_children(chrome_buttons.map(|button| NodeId(button.node_id)));
    nodes.insert(NodeId(accessibility::node_ids::TITLEBAR), titlebar);

    for chrome_button in chrome_buttons {
        nodes.insert(
            NodeId(chrome_button.node_id),
            chrome_button_node(chrome_button.label, chrome_button.shortcut_hint),
        );
    }
}

fn insert_canvas_and_status_nodes(nodes: &mut BTreeMap<NodeId, Node>, snapshot: &A11ySnapshot) {
    let mut canvas = Node::new(Role::Canvas);
    let canvas_label = snapshot
        .localizer
        .lookup(&snapshot.locale, &MessageId::a11y_canvas())
        .unwrap_or_else(|err| {
            log::warn!(
                "a11y i18n lookup failed for {:?}: {err}",
                MessageId::a11y_canvas().as_str()
            );
            "Drawing canvas".to_owned()
        });
    canvas.set_label(canvas_label);
    nodes.insert(CANVAS_NODE_ID, canvas);

    let mut status = Node::new(Role::Status);
    let status_text = localized_status_label(
        snapshot.tool_mode,
        snapshot.edge_mode,
        &snapshot.localizer,
        &snapshot.locale,
    );
    status.set_label(status_text);
    nodes.insert(STATUS_NODE_ID, status);
}

fn insert_shape_list_nodes(
    nodes: &mut BTreeMap<NodeId, Node>,
    snapshot: &A11ySnapshot,
) -> Result<(), A11yServiceError> {
    let mut shape_list = Node::new(Role::List);
    let mut shape_node_ids = Vec::with_capacity(snapshot.shapes.len());
    for (index, shape) in snapshot.shapes.iter().enumerate() {
        let shape_node_id = NodeId(shape.id.to_accesskit_node_id());
        validate_shape_node_id(shape_node_id, nodes)?;
        let mut shape_node = Node::new(Role::ListItem);
        shape_node.set_label(shape_label(
            shape.name.as_deref(),
            index,
            &snapshot.localizer,
            &snapshot.locale,
        ));
        if shape.locked {
            shape_node.set_disabled();
        }
        if shape.hidden {
            shape_node.set_hidden();
        }
        if snapshot.selected_shape_ids.contains(&shape.id) {
            shape_node.set_selected(true);
        }
        nodes.insert(shape_node_id, shape_node);
        shape_node_ids.push(shape_node_id);
    }
    shape_list.set_children(shape_node_ids);
    let shape_list_label = snapshot
        .localizer
        .lookup(&snapshot.locale, &MessageId::a11y_shape_list())
        .unwrap_or_else(|err| {
            log::warn!(
                "a11y i18n lookup failed for {:?}: {err}",
                MessageId::a11y_shape_list().as_str()
            );
            "Shapes".to_owned()
        });
    shape_list.set_label(shape_list_label);
    shape_list.set_multiselectable();
    nodes.insert(SHAPE_LIST_NODE_ID, shape_list);
    Ok(())
}

fn shape_label(
    name: Option<&str>,
    index: usize,
    localizer: &Localizer,
    locale: &crate::i18n::Locale,
) -> String {
    name.map_or_else(
        || {
            let template = localizer
                .lookup(locale, &MessageId::a11y_shape_item())
                .unwrap_or_else(|err| {
                    log::warn!(
                        "a11y i18n lookup failed for {:?}: {err}",
                        MessageId::a11y_shape_item().as_str()
                    );
                    "Shape {index}".to_owned()
                });
            template.replace("{index}", &(index + 1).to_string())
        },
        ToOwned::to_owned,
    )
}

fn validate_shape_node_id(
    shape_node_id: NodeId,
    nodes: &BTreeMap<NodeId, Node>,
) -> Result<(), A11yServiceError> {
    if RESERVED_NODE_IDS.contains(&shape_node_id.0) {
        return Err(A11yServiceError::ReservedNodeIdCollision {
            node_id: shape_node_id.0,
        });
    }
    if nodes.contains_key(&shape_node_id) {
        return Err(A11yServiceError::DuplicateShapeNodeId {
            node_id: shape_node_id.0,
        });
    }
    Ok(())
}

fn insert_root_node(nodes: &mut BTreeMap<NodeId, Node>, snapshot: &A11ySnapshot) {
    let mut root = Node::new(Role::Window);
    let window_title = snapshot
        .localizer
        .lookup(&snapshot.locale, &MessageId::a11y_window_title())
        .unwrap_or_else(|err| {
            log::warn!(
                "a11y i18n lookup failed for {:?}: {err}",
                MessageId::a11y_window_title().as_str()
            );
            "Gauss".to_owned()
        });
    root.set_label(window_title);
    root.set_children([
        NodeId(accessibility::node_ids::TITLEBAR),
        STATUS_NODE_ID,
        CANVAS_NODE_ID,
        SHAPE_LIST_NODE_ID,
    ]);
    nodes.insert(ROOT_NODE_ID, root);
}

fn resolve_focus_node(snapshot: &A11ySnapshot, nodes: &BTreeMap<NodeId, Node>) -> NodeId {
    let shape_node_ids = snapshot
        .shapes
        .iter()
        .map(|shape| NodeId(shape.id.to_accesskit_node_id()))
        .collect::<BTreeSet<_>>();
    snapshot
        .selected_shape_ids
        .iter()
        .map(|id| NodeId(id.to_accesskit_node_id()))
        .find(|candidate| shape_node_ids.contains(candidate) && nodes.contains_key(candidate))
        .unwrap_or(CANVAS_NODE_ID)
}

fn chrome_button_node(label: &'static str, shortcut_hint: &'static str) -> Node {
    let mut node = Node::new(Role::Button);
    node.set_label(label);
    node.set_description(shortcut_hint);
    node.set_keyboard_shortcut(shortcut_hint);
    node.add_action(Action::Click);
    node
}
