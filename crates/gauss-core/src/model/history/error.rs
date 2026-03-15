//! Semantic error types and messages for document undo/redo history.

use thiserror::Error;

/// Error returned when `begin_group()` is called while a group is active.
pub const GROUPING_ERROR_GROUP_ALREADY_ACTIVE: &str =
    "Cannot begin command group: group already active";

/// Error returned when `end_group()` is called without an active group.
pub const GROUPING_ERROR_NO_ACTIVE_GROUP: &str = "Cannot end command group: no active group";

/// Error returned when `undo()` is called while a group is still open.
pub const GROUPING_ERROR_UNDO_WHILE_GROUP_ACTIVE: &str =
    "Cannot undo while command group is active";

/// Error returned when `redo()` is called while a group is still open.
pub const GROUPING_ERROR_REDO_WHILE_GROUP_ACTIVE: &str =
    "Cannot redo while command group is active";

/// Semantic errors produced by document undo/redo history operations.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum HistoryError {
    /// `begin_group()` was called while another group is already active.
    #[error("{GROUPING_ERROR_GROUP_ALREADY_ACTIVE}")]
    GroupAlreadyActive,
    /// `end_group()` was called when no group was active.
    #[error("{GROUPING_ERROR_NO_ACTIVE_GROUP}")]
    NoActiveGroup,
    /// `undo()` was called while a group is active.
    #[error("{GROUPING_ERROR_UNDO_WHILE_GROUP_ACTIVE}")]
    UndoWhileGroupActive,
    /// `redo()` was called while a group is active.
    #[error("{GROUPING_ERROR_REDO_WHILE_GROUP_ACTIVE}")]
    RedoWhileGroupActive,
    /// Replaying a command during undo failed.
    #[error("Undo failed for '{command_name}': {reason}")]
    UndoReplayFailed {
        /// Human-readable command name that failed.
        command_name: String,
        /// Underlying command failure reason.
        reason: String,
    },
    /// Replaying a command during redo failed.
    #[error("Redo failed for '{command_name}': {reason}")]
    RedoReplayFailed {
        /// Human-readable command name that failed.
        command_name: String,
        /// Underlying command failure reason.
        reason: String,
    },
}

#[derive(Clone, Copy)]
pub(super) enum ReplayDirection {
    Undo,
    Redo,
}

impl ReplayDirection {
    pub(super) fn replay_failed(self, command_name: &str, reason: &str) -> HistoryError {
        let owned_command_name = command_name.to_owned();
        let owned_reason = reason.to_owned();
        match self {
            Self::Undo => HistoryError::UndoReplayFailed {
                command_name: owned_command_name,
                reason: owned_reason,
            },
            Self::Redo => HistoryError::RedoReplayFailed {
                command_name: owned_command_name,
                reason: owned_reason,
            },
        }
    }
}
