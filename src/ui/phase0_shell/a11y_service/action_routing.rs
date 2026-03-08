//! Accessibility action routing for `A11yService`.
//!
//! This module translates supported AccessKit action requests into the
//! existing shell/model action paths used elsewhere in Phase 0.

use accesskit::{ActionRequest, NodeId, TreeId};
use thiserror::Error;

use crate::model::Action;

use super::A11yService;

/// Routed action target for a supported accessibility request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yRequestedAction {
    /// Route the request through an existing model-layer action path.
    Model(Action),
    /// Route the request through an existing shell window-control action path.
    Window(A11yWindowAction),
}

/// Window-control actions currently exposed through accessibility requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yWindowAction {
    /// Open the system window menu.
    ShowWindowMenu,
    /// Minimize the current window.
    Minimize,
    /// Toggle maximize or restore.
    ToggleMaximize,
    /// Toggle fullscreen.
    ToggleFullscreen,
    /// Close the current window.
    CloseWindow,
}

/// Typed failures returned when routing accessibility action requests.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum A11yActionRequestError {
    /// The request targeted an unsupported accessibility tree.
    #[error("unsupported accessibility tree id {target_tree:?}")]
    UnsupportedTree {
        /// Tree requested by the accessibility client.
        target_tree: TreeId,
    },
    /// The request targeted a node/action pair not supported by Gauss.
    #[error("node {target_node:#x} does not support accessibility action {action:?}")]
    UnsupportedAction {
        /// The target node ID from the request.
        target_node: u64,
        /// The action requested for that node.
        action: accesskit::Action,
    },
}

impl A11yService {
    /// Route an AccessKit action request to the existing Gauss action path.
    ///
    /// # Errors
    ///
    /// Returns [`A11yActionRequestError`] when the target tree is not the root
    /// tree or when the node/action pair is not supported by the current
    /// accessibility surface.
    pub fn route_action_request(
        &self,
        request: &ActionRequest,
    ) -> Result<A11yRequestedAction, A11yActionRequestError> {
        if request.target_tree != TreeId::ROOT {
            return Err(A11yActionRequestError::UnsupportedTree {
                target_tree: request.target_tree,
            });
        }

        if !self
            .previous_nodes
            .contains_key(&NodeId(request.target_node.0))
        {
            return Err(A11yActionRequestError::UnsupportedAction {
                target_node: request.target_node.0,
                action: request.action,
            });
        }

        let routed = match (request.target_node.0, request.action) {
            (super::super::accessibility::node_ids::WINDOW_MENU, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::ShowWindowMenu)
            }
            (super::super::accessibility::node_ids::MINIMIZE_BUTTON, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::Minimize)
            }
            (super::super::accessibility::node_ids::MAXIMIZE_BUTTON, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::ToggleMaximize)
            }
            (
                super::super::accessibility::node_ids::FULLSCREEN_BUTTON,
                accesskit::Action::Click,
            ) => A11yRequestedAction::Window(A11yWindowAction::ToggleFullscreen),
            (super::super::accessibility::node_ids::CLOSE_BUTTON, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::CloseWindow)
            }
            _ => {
                return Err(A11yActionRequestError::UnsupportedAction {
                    target_node: request.target_node.0,
                    action: request.action,
                });
            }
        };

        Ok(routed)
    }
}
