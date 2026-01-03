//! User-intent Actions for the Gauss editor.
//!
//! Actions represent what the user wants to do (e.g., "delete selection")
//! without specifying how. Actions are dispatchable from UI, scripts, and
//! tests. They are the public API surface for all editor behaviour.
//!
//! Actions are GPUI-independent for testability and scripting.
//!
//! # Design
//!
//! Actions are implemented as an enum rather than a trait for several reasons:
//!
//! - **Exhaustive matching**: All action variants can be matched exhaustively,
//!   making dispatch tables complete and verifiable at compile time.
//! - **Serialization**: Enums are trivially serializable, enabling future macro
//!   recording and playback (see roadmap §0.1.2).
//! - **Simplicity**: No type erasure or dynamic dispatch complexity.
//! - **Hashable**: Both [`Action`] and [`ActionKind`] derive `Hash`, enabling
//!   use as map keys for dispatch table caching and keybinding lookups.
//!
//! # Relationship to Commands
//!
//! Actions represent user intent; Commands (task 0.1.2) represent concrete,
//! undoable state mutations. The relationship is:
//!
//! ```text
//! Action (user intent)
//!    |
//!    v  dispatch()
//! Command (undoable mutation)
//!    |
//!    v  apply()
//! DocChange / DocOp (atomic operations)
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use gauss::model::{Action, ActionKind};
//!
//! let action = Action::DeleteSelection;
//! assert_eq!(action.kind(), ActionKind::Document);
//! assert_eq!(action.name(), "Delete Selection");
//! assert!(action.requires_selection());
//! ```

/// Categorization of actions for dispatch routing.
///
/// Actions are grouped by the type of state they affect, which determines
/// how they are dispatched and whether they produce undoable commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActionKind {
    /// Mutates document state; produces undoable Command.
    ///
    /// Document actions require the command system (task 0.1.2) to execute
    /// and are recorded in the undo history.
    Document,

    /// Mutates editor state (selection, viewport, tool, history navigation).
    ///
    /// Editor actions may or may not be undoable. Selection changes have
    /// their own history stack; viewport and tool changes are typically
    /// not recorded. History navigation actions (Undo/Redo) traverse the
    /// document history stack but do not themselves produce new undo entries.
    Editor,
}

/// User intent representation.
///
/// Actions are the unit of user-visible behaviour. Every feature must be
/// expressible as an Action to satisfy the guiding principle "Everything is
/// an Action (and therefore scriptable)".
///
/// # Variants
///
/// This enum uses `#[non_exhaustive]` to allow adding new action variants
/// in future versions without breaking downstream code.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::Action;
///
/// // Actions can be matched exhaustively within this crate
/// let action = Action::Undo;
/// let name = action.name();
/// assert_eq!(name, "Undo");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Action {
    // === Document mutations ===
    /// Delete currently selected objects.
    ///
    /// Requires a non-empty selection. Produces a reversible command that
    /// removes the selected shapes from the document.
    DeleteSelection,

    /// Insert an anchor on the selected segment.
    ///
    /// Requires a segment to be selected. Inserts a new anchor at the
    /// midpoint of the selected segment, splitting it into two segments.
    InsertAnchorOnSegment,

    /// Delete the selected anchors.
    ///
    /// Requires anchors to be selected. Removes the selected anchors from
    /// their shapes. If a shape would have fewer than 2 anchors, the entire
    /// shape is removed.
    DeleteSelectedAnchors,

    /// Raise selected shapes in the z-order.
    ///
    /// Moves selected shapes one position higher in the document's shape
    /// list, causing them to render on top of shapes that were previously
    /// above them.
    RaiseSelection,

    /// Lower selected shapes in the z-order.
    ///
    /// Moves selected shapes one position lower in the document's shape
    /// list, causing them to render behind shapes that were previously
    /// below them.
    LowerSelection,

    /// Toggle segment kind between Line and Cubic.
    ///
    /// Requires segments to be selected. Toggles each selected segment
    /// between Line and Cubic kinds. When converting Line to Cubic,
    /// Catmull-Rom handles are synthesised. When converting Cubic to Line,
    /// handles are cleared.
    ToggleSegmentKind,

    // === Selection changes ===
    /// Select all selectable objects in the document.
    ///
    /// Clears any existing selection and selects all shapes in the document.
    SelectAll,

    /// Clear the current selection.
    ///
    /// Removes all items from the current selection, leaving nothing selected.
    DeselectAll,

    // === Tool activation ===
    /// Activate the Pen (draw) tool.
    ///
    /// Switches to draw mode where clicking places anchors to create paths.
    ActivatePenTool,

    /// Activate the Selection (manipulate) tool.
    ///
    /// Switches to manipulate mode where shapes, anchors, and handles can
    /// be selected and moved.
    ActivateSelectTool,

    // === History ===
    /// Undo the last document change.
    ///
    /// Reverts the most recent command from the document history stack.
    /// Has no effect if the history is empty.
    Undo,

    /// Redo the last undone change.
    ///
    /// Re-applies the most recently undone command from the redo stack.
    /// Has no effect if the redo stack is empty.
    Redo,

    /// Undo the last selection change.
    ///
    /// Reverts the most recent selection change from the selection history
    /// stack. Selection history is separate from document history, enabling
    /// independent traversal of selection and edit states.
    SelectionUndo,

    /// Redo the last undone selection change.
    ///
    /// Re-applies the most recently undone selection change from the selection
    /// redo stack.
    SelectionRedo,
}

impl Action {
    /// Return the kind of this action for dispatch routing.
    ///
    /// The kind determines how the action is processed:
    ///
    /// - [`ActionKind::Document`]: Requires command dispatch, produces undo entry
    /// - [`ActionKind::Editor`]: May update editor state directly
    ///
    /// # Returns
    ///
    /// The [`ActionKind`] categorizing this action for dispatch routing.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::{Action, ActionKind};
    ///
    /// assert_eq!(Action::DeleteSelection.kind(), ActionKind::Document);
    /// assert_eq!(Action::SelectAll.kind(), ActionKind::Editor);
    /// ```
    #[must_use]
    pub const fn kind(&self) -> ActionKind {
        match self {
            // Document mutations require command dispatch
            Self::DeleteSelection
            | Self::InsertAnchorOnSegment
            | Self::DeleteSelectedAnchors
            | Self::RaiseSelection
            | Self::LowerSelection
            | Self::ToggleSegmentKind => ActionKind::Document,

            // Editor state changes (selection, tools, history navigation)
            Self::SelectAll
            | Self::DeselectAll
            | Self::ActivatePenTool
            | Self::ActivateSelectTool
            | Self::Undo
            | Self::Redo
            | Self::SelectionUndo
            | Self::SelectionRedo => ActionKind::Editor,
        }
    }

    /// Return a human-readable name for this action.
    ///
    /// This name is suitable for:
    ///
    /// - Undo/redo menu descriptions ("Undo Delete Selection")
    /// - Accessibility labels
    /// - Scripting API documentation
    /// - Command palette display
    ///
    /// Note: These names will be replaced with localized strings when the
    /// i18n scaffolding (task 0.7) is implemented.
    ///
    /// # Returns
    ///
    /// A static string containing the human-readable action name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::Action;
    ///
    /// assert_eq!(Action::DeleteSelection.name(), "Delete Selection");
    /// assert_eq!(Action::Undo.name(), "Undo");
    /// ```
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::DeleteSelection => "Delete Selection",
            Self::InsertAnchorOnSegment => "Insert Anchor",
            Self::DeleteSelectedAnchors => "Delete Anchors",
            Self::RaiseSelection => "Raise",
            Self::LowerSelection => "Lower",
            Self::ToggleSegmentKind => "Toggle Segment",
            Self::SelectAll => "Select All",
            Self::DeselectAll => "Deselect All",
            Self::ActivatePenTool => "Pen Tool",
            Self::ActivateSelectTool => "Select Tool",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::SelectionUndo => "Selection Undo",
            Self::SelectionRedo => "Selection Redo",
        }
    }

    /// Return whether this action requires a non-empty selection to be valid.
    ///
    /// Actions that require selection should be disabled in the UI when
    /// nothing is selected, and should be rejected by the command dispatcher
    /// with an appropriate error.
    ///
    /// # Returns
    ///
    /// `true` if this action requires a non-empty selection, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::Action;
    ///
    /// assert!(Action::DeleteSelection.requires_selection());
    /// assert!(!Action::SelectAll.requires_selection());
    /// ```
    #[must_use]
    pub const fn requires_selection(&self) -> bool {
        matches!(
            self,
            Self::DeleteSelection
                | Self::InsertAnchorOnSegment
                | Self::DeleteSelectedAnchors
                | Self::RaiseSelection
                | Self::LowerSelection
                | Self::ToggleSegmentKind
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Action::DeleteSelection, ActionKind::Document)]
    #[case(Action::InsertAnchorOnSegment, ActionKind::Document)]
    #[case(Action::DeleteSelectedAnchors, ActionKind::Document)]
    #[case(Action::RaiseSelection, ActionKind::Document)]
    #[case(Action::LowerSelection, ActionKind::Document)]
    #[case(Action::ToggleSegmentKind, ActionKind::Document)]
    #[case(Action::SelectAll, ActionKind::Editor)]
    #[case(Action::DeselectAll, ActionKind::Editor)]
    #[case(Action::ActivatePenTool, ActionKind::Editor)]
    #[case(Action::ActivateSelectTool, ActionKind::Editor)]
    #[case(Action::Undo, ActionKind::Editor)]
    #[case(Action::Redo, ActionKind::Editor)]
    #[case(Action::SelectionUndo, ActionKind::Editor)]
    #[case(Action::SelectionRedo, ActionKind::Editor)]
    fn action_kind_is_correct(#[case] action: Action, #[case] expected: ActionKind) {
        assert_eq!(action.kind(), expected);
    }

    #[rstest]
    #[case(Action::DeleteSelection, "Delete Selection")]
    #[case(Action::InsertAnchorOnSegment, "Insert Anchor")]
    #[case(Action::DeleteSelectedAnchors, "Delete Anchors")]
    #[case(Action::RaiseSelection, "Raise")]
    #[case(Action::LowerSelection, "Lower")]
    #[case(Action::ToggleSegmentKind, "Toggle Segment")]
    #[case(Action::SelectAll, "Select All")]
    #[case(Action::DeselectAll, "Deselect All")]
    #[case(Action::ActivatePenTool, "Pen Tool")]
    #[case(Action::ActivateSelectTool, "Select Tool")]
    #[case(Action::Undo, "Undo")]
    #[case(Action::Redo, "Redo")]
    #[case(Action::SelectionUndo, "Selection Undo")]
    #[case(Action::SelectionRedo, "Selection Redo")]
    fn action_name_is_correct(#[case] action: Action, #[case] expected: &str) {
        assert_eq!(action.name(), expected);
    }

    #[rstest]
    #[case(Action::DeleteSelection)]
    #[case(Action::InsertAnchorOnSegment)]
    #[case(Action::DeleteSelectedAnchors)]
    #[case(Action::RaiseSelection)]
    #[case(Action::LowerSelection)]
    #[case(Action::ToggleSegmentKind)]
    #[case(Action::SelectAll)]
    #[case(Action::DeselectAll)]
    #[case(Action::ActivatePenTool)]
    #[case(Action::ActivateSelectTool)]
    #[case(Action::Undo)]
    #[case(Action::Redo)]
    #[case(Action::SelectionUndo)]
    #[case(Action::SelectionRedo)]
    fn actions_have_nonempty_names(#[case] action: Action) {
        assert!(!action.name().is_empty());
    }

    #[rstest]
    #[case(Action::DeleteSelection)]
    #[case(Action::InsertAnchorOnSegment)]
    #[case(Action::DeleteSelectedAnchors)]
    #[case(Action::RaiseSelection)]
    #[case(Action::LowerSelection)]
    #[case(Action::ToggleSegmentKind)]
    fn document_actions_require_selection(#[case] action: Action) {
        assert!(action.requires_selection());
    }

    #[rstest]
    #[case(Action::SelectAll)]
    #[case(Action::DeselectAll)]
    #[case(Action::ActivatePenTool)]
    #[case(Action::ActivateSelectTool)]
    #[case(Action::Undo)]
    #[case(Action::Redo)]
    #[case(Action::SelectionUndo)]
    #[case(Action::SelectionRedo)]
    fn non_document_actions_do_not_require_selection(#[case] action: Action) {
        assert!(!action.requires_selection());
    }

    #[test]
    fn document_actions_are_all_accounted_for() {
        // Ensure that any action requiring selection is a Document action.
        //
        // NOTE: This list is intentionally hardcoded rather than generated.
        // When adding a new Action variant, the developer must explicitly add
        // it here, forcing consideration of the selection-requires-Document
        // invariant. A compile error from an unmatched variant in the match
        // arms above will remind you to update this test.
        let all_actions = [
            Action::DeleteSelection,
            Action::InsertAnchorOnSegment,
            Action::DeleteSelectedAnchors,
            Action::RaiseSelection,
            Action::LowerSelection,
            Action::ToggleSegmentKind,
            Action::SelectAll,
            Action::DeselectAll,
            Action::ActivatePenTool,
            Action::ActivateSelectTool,
            Action::Undo,
            Action::Redo,
            Action::SelectionUndo,
            Action::SelectionRedo,
        ];

        for action in all_actions {
            if action.requires_selection() {
                assert_eq!(
                    action.kind(),
                    ActionKind::Document,
                    "{action:?} requires selection but is not Document kind"
                );
            }
        }
    }

    #[test]
    fn action_is_copy() {
        // Verify Action implements Copy (important for ergonomics)
        fn assert_copy<T: Copy>(_: T) {}

        assert_copy(Action::Undo);
    }

    #[test]
    fn action_kind_is_copy() {
        // Verify ActionKind implements Copy
        fn assert_copy<T: Copy>(_: T) {}

        assert_copy(ActionKind::Document);
    }
}
