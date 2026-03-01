//! AccessKit tree construction for the Phase 0 shell accessibility snapshot.

use std::collections::BTreeMap;

use accesskit::{Action, Node, NodeId, Role};

use crate::ui::phase0_shell::accessibility;

use super::{A11yServiceError, A11ySnapshot};

pub(super) const ROOT_NODE_ID: NodeId = NodeId(0x1000);
pub(super) const CANVAS_NODE_ID: NodeId = NodeId(0x1007);
pub(super) const SHAPE_LIST_NODE_ID: NodeId = NodeId(0x1008);
const STATUS_NODE_ID: NodeId = NodeId(0x1009);

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

pub(super) fn build_node_map(
    snapshot: &A11ySnapshot,
) -> Result<(BTreeMap<NodeId, Node>, NodeId), A11yServiceError> {
    let mut nodes = BTreeMap::new();
    insert_chrome_nodes(&mut nodes, snapshot.is_maximized);
    insert_canvas_and_status_nodes(&mut nodes, snapshot);
    insert_shape_list_nodes(&mut nodes, snapshot)?;
    insert_root_node(&mut nodes);
    let focus = resolve_focus_node(snapshot, &nodes);
    Ok((nodes, focus))
}

fn insert_chrome_nodes(nodes: &mut BTreeMap<NodeId, Node>, is_maximized: bool) {
    let mut titlebar = Node::new(Role::TitleBar);
    titlebar.set_label("Window title bar");
    titlebar.set_children([
        NodeId(accessibility::node_ids::WINDOW_MENU),
        NodeId(accessibility::node_ids::MINIMIZE_BUTTON),
        NodeId(accessibility::node_ids::MAXIMIZE_BUTTON),
        NodeId(accessibility::node_ids::FULLSCREEN_BUTTON),
        NodeId(accessibility::node_ids::CLOSE_BUTTON),
    ]);
    nodes.insert(NodeId(accessibility::node_ids::TITLEBAR), titlebar);

    nodes.insert(
        NodeId(accessibility::node_ids::WINDOW_MENU),
        chrome_button_node(
            accessibility::accessible_names::WINDOW_MENU,
            accessibility::shortcut_hints::WINDOW_MENU,
        ),
    );
    nodes.insert(
        NodeId(accessibility::node_ids::MINIMIZE_BUTTON),
        chrome_button_node(
            accessibility::accessible_names::MINIMIZE,
            accessibility::shortcut_hints::MINIMIZE,
        ),
    );
    let maximize_name = if is_maximized {
        accessibility::accessible_names::RESTORE
    } else {
        accessibility::accessible_names::MAXIMIZE
    };
    nodes.insert(
        NodeId(accessibility::node_ids::MAXIMIZE_BUTTON),
        chrome_button_node(maximize_name, accessibility::shortcut_hints::MAXIMIZE),
    );
    nodes.insert(
        NodeId(accessibility::node_ids::FULLSCREEN_BUTTON),
        chrome_button_node(
            accessibility::accessible_names::FULLSCREEN,
            accessibility::shortcut_hints::FULLSCREEN,
        ),
    );
    nodes.insert(
        NodeId(accessibility::node_ids::CLOSE_BUTTON),
        chrome_button_node(
            accessibility::accessible_names::CLOSE,
            accessibility::shortcut_hints::CLOSE,
        ),
    );
}

fn insert_canvas_and_status_nodes(nodes: &mut BTreeMap<NodeId, Node>, snapshot: &A11ySnapshot) {
    let mut canvas = Node::new(Role::Canvas);
    canvas.set_label("Drawing canvas");
    nodes.insert(CANVAS_NODE_ID, canvas);

    let mut status = Node::new(Role::Status);
    status.set_label(format!(
        "Mode: {} ({})",
        snapshot.tool_mode.label(),
        snapshot.edge_mode.label()
    ));
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
        shape_node.set_label(shape_label(shape.name.as_deref(), index));
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
    shape_list.set_label("Shapes");
    shape_list.set_multiselectable();
    nodes.insert(SHAPE_LIST_NODE_ID, shape_list);
    Ok(())
}

fn shape_label(name: Option<&str>, index: usize) -> String {
    name.map_or_else(|| format!("Shape {}", index + 1), ToOwned::to_owned)
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

fn insert_root_node(nodes: &mut BTreeMap<NodeId, Node>) {
    let mut root = Node::new(Role::Window);
    root.set_label("Gauss");
    root.set_children([
        NodeId(accessibility::node_ids::TITLEBAR),
        STATUS_NODE_ID,
        CANVAS_NODE_ID,
        SHAPE_LIST_NODE_ID,
    ]);
    nodes.insert(ROOT_NODE_ID, root);
}

fn resolve_focus_node(snapshot: &A11ySnapshot, nodes: &BTreeMap<NodeId, Node>) -> NodeId {
    snapshot
        .selected_shape_ids
        .iter()
        .map(|id| NodeId(id.to_accesskit_node_id()))
        .find(|candidate| nodes.contains_key(candidate))
        .unwrap_or(CANVAS_NODE_ID)
}

fn chrome_button_node(label: &'static str, shortcut_hint: &'static str) -> Node {
    let mut node = Node::new(Role::Button);
    node.set_label(label);
    node.set_description(shortcut_hint);
    node.add_action(Action::Click);
    node
}
