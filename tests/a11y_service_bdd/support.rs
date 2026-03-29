//! Shared fixtures and helpers for `A11yService` behaviour tests.

use std::collections::{BTreeMap, BTreeSet};

use accesskit::{Node, NodeId};
use gauss::model::{EdgeMode, ToolMode};
use gauss::ui::phase0_shell::{A11yRequestedAction, A11yService, A11yServiceError, A11ySnapshot};
use rstest::fixture;
use rstest_bdd_macros::given;

pub(crate) use test_support::{TestSupportError, TestSupportResult};

pub(crate) struct A11yWorld {
    pub(crate) service: A11yService,
    pub(crate) snapshot: A11ySnapshot,
    pub(crate) last_emitted_nodes: BTreeMap<NodeId, Node>,
    pub(crate) last_publish_result: Option<Result<bool, A11yServiceError>>,
    pub(crate) last_routed_action: Option<A11yRequestedAction>,
    pub(crate) last_route_error: Option<gauss::ui::phase0_shell::A11yActionRequestError>,
}

#[fixture]
pub(crate) fn world() -> A11yWorld {
    A11yWorld {
        service: A11yService::new(),
        snapshot: empty_snapshot(),
        last_emitted_nodes: BTreeMap::new(),
        last_publish_result: None,
        last_routed_action: None,
        last_route_error: None,
    }
}

pub(crate) fn empty_snapshot() -> A11ySnapshot {
    A11ySnapshot {
        tool_mode: ToolMode::Draw,
        edge_mode: EdgeMode::Line,
        can_undo: false,
        can_redo: false,
        is_maximized: false,
        selected_shape_ids: BTreeSet::new(),
        shapes: Vec::new(),
        localizer: gauss::i18n::Localizer::default(),
        locale: gauss::i18n::Locale::default(),
    }
}

pub(crate) fn capture_last_emitted_nodes(world: &mut A11yWorld) {
    let pending_updates = world.service.drain_pending_updates();
    world.last_emitted_nodes.clear();
    if let Some(update) = pending_updates.last() {
        world.last_emitted_nodes = update
            .nodes
            .iter()
            .map(|(node_id, node)| (*node_id, node.clone()))
            .collect();
    }
}

#[given("a fresh accessibility service snapshot")]
pub(crate) fn fresh_accessibility_service_snapshot(world: &mut A11yWorld) {
    world.service = A11yService::new();
    world.snapshot = empty_snapshot();
    world.last_emitted_nodes.clear();
    world.last_publish_result = None;
    world.last_routed_action = None;
    world.last_route_error = None;
}
