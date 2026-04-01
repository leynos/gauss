//! Action-to-command preparation.

use crate::model::{Action, EngineState};

use super::Command;
use super::anchor::{prepare_delete_selected_anchors, prepare_insert_anchor_on_segment};
use super::delete_shapes::prepare_delete_selection;
use super::error::UserError;
use super::reorder::{prepare_lower_selection, prepare_raise_selection};
use super::segment::prepare_toggle_segment_kind;

/// Panic message for dispatcher bugs where editor actions reach `prepare_command`.
///
/// Editor actions (`Undo`, `Redo`, `SelectAll`, etc.) do not produce commands and
/// should be routed directly by the dispatcher. If they reach `prepare_command`,
/// it indicates a bug in the dispatch logic.
const DISPATCHER_BUG_MSG: &str = concat!(
    "dispatcher bug: this action does not produce a command ",
    "and should be routed directly",
);

/// Prepare a command from an action and current engine state.
///
/// This function bridges user intent (Action) to concrete command (Command).
/// It captures required context from the unified engine state at the moment
/// the action is invoked.
///
/// # Parameters
///
/// - `action`: The user action to convert.
/// - `state`: The current engine state (document, selection, viewport, etc.).
///
/// # Returns
///
/// A [`Command`] ready for execution, or an error if the action cannot
/// produce a valid command.
///
/// # Errors
///
/// Returns [`UserError`] if the action cannot produce a valid command:
///
/// - [`UserError::EmptySelection`]: Action requires selection but none exists.
/// - [`UserError::ShapeNotFound`]: Selected shape not in document.
///
/// # Panics
///
/// Panics if an editor action (e.g., `Undo`, `Redo`, `SelectAll`) is passed.
/// These actions do not produce commands and should be routed directly by the
/// dispatcher. A panic here indicates a dispatcher bug.
///
/// # Examples
///
/// ```rust
/// use gauss_core::model::{Action, EngineState, prepare_command};
///
/// let state = EngineState::new();
///
/// // Empty selection produces an error
/// let result = prepare_command(Action::DeleteSelection, &state);
/// assert!(result.is_err());
/// ```
#[expect(
    clippy::panic_in_result_fn,
    reason = "Panic is intentional fail-fast for dispatcher bugs; editor actions \
              should never reach this function in correct code"
)]
pub fn prepare_command(action: Action, state: &EngineState) -> Result<Command, UserError> {
    match action {
        Action::DeleteSelection => prepare_delete_selection(&state.document, &state.selection),
        Action::InsertAnchorOnSegment => prepare_insert_anchor_on_segment(state),
        Action::DeleteSelectedAnchors => prepare_delete_selected_anchors(state),
        Action::RaiseSelection => prepare_raise_selection(state),
        Action::LowerSelection => prepare_lower_selection(state),
        Action::ToggleSegmentKind => prepare_toggle_segment_kind(state),
        // Style actions are not yet implemented in the command system.
        Action::SetStrokeColor
        | Action::SetStrokeWidth
        | Action::SetStrokeOpacity
        | Action::SetFillColor
        | Action::SetFillOpacity
        | Action::ToggleNoFill => Err(UserError::InvalidOperation(format!(
            "{action:?} command not yet implemented"
        ))),
        // Editor actions do not produce commands; this is a dispatcher bug.
        // We panic unconditionally; the match arm is never reached in correct code.
        Action::SelectAll
        | Action::DeselectAll
        | Action::ActivatePenTool
        | Action::ActivateSelectTool
        | Action::ActivateMyNewTool
        | Action::Undo
        | Action::Redo
        | Action::SelectionUndo
        | Action::SelectionRedo => panic!("{DISPATCHER_BUG_MSG} (got {action:?})"),
    }
}
