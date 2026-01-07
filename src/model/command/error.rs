//! User-facing errors from command preparation and execution.

use crate::model::ShapeId;

/// User-facing errors that can occur during command preparation or execution.
///
/// These errors represent semantic issues that should be presented to users
/// (e.g., via UI messages, disabled menu items, or accessibility feedback).
///
/// # UI Integration
///
/// UI code should handle [`UserError`] gracefully:
///
/// ```rust,ignore
/// use gauss::model::{Action, UserError, prepare_command};
///
/// // In UI action handler:
/// match prepare_command(action, &state) {
///     Ok(cmd) => {
///         // Execute command, add to undo stack
///     }
///     Err(UserError::EmptySelection) => {
///         // Show "Nothing selected" message or disable menu item
///     }
///     Err(UserError::ShapeNotFound(id)) => {
///         // Log error, show "Shape not found" message
///         // This shouldn't happen in normal use
///     }
///     // Handle other errors...
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum UserError {
    /// The command requires a non-empty selection, but nothing is selected.
    #[error("No selection")]
    EmptySelection,

    /// A referenced shape does not exist in the document.
    #[error("Shape not found")]
    ShapeNotFound(ShapeId),

    /// A referenced anchor does not exist in the shape.
    #[error("Anchor not found in shape")]
    AnchorNotFound(ShapeId, usize),

    /// A referenced segment does not exist in the shape.
    #[error("Segment not found in shape")]
    SegmentNotFound(ShapeId, usize),

    /// The operation cannot be performed in the current state.
    #[error("{0}")]
    InvalidOperation(String),
}
