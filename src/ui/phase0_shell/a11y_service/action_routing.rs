//! Accessibility action routing for `A11yService`.
//!
//! This module translates supported AccessKit action requests into the
//! existing shell/model action paths used elsewhere in Phase 0.

use accesskit::{ActionRequest, TreeId};
use thiserror::Error;

use crate::model::Action;

use super::A11yService;

/// Routed action target for a supported accessibility request.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// The request targeted a node that is not present in the current tree.
    #[error("unknown accessibility node id {target_node:#x}")]
    UnknownNode {
        /// The target node ID from the request.
        target_node: u64,
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
    /// tree, when the node is missing from the current accessibility surface,
    /// or when the node/action pair is not supported by Gauss.
    pub fn route_action_request(
        &self,
        request: &ActionRequest,
    ) -> Result<A11yRequestedAction, A11yActionRequestError> {
        use super::super::accessibility::node_ids;

        if request.target_tree != TreeId::ROOT {
            return Err(A11yActionRequestError::UnsupportedTree {
                target_tree: request.target_tree,
            });
        }

        if !self.previous_nodes.contains_key(&request.target_node) {
            return Err(A11yActionRequestError::UnknownNode {
                target_node: request.target_node.0,
            });
        }

        let routed = match (request.target_node.0, request.action) {
            (node_ids::WINDOW_MENU, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::ShowWindowMenu)
            }
            (node_ids::MINIMIZE_BUTTON, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::Minimize)
            }
            (node_ids::MAXIMIZE_BUTTON, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::ToggleMaximize)
            }
            (node_ids::FULLSCREEN_BUTTON, accesskit::Action::Click) => {
                A11yRequestedAction::Window(A11yWindowAction::ToggleFullscreen)
            }
            (node_ids::CLOSE_BUTTON, accesskit::Action::Click) => {
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
