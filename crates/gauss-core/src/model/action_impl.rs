//! `Action` method implementations.

use super::{Action, ActionKind};

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
    /// use gauss_core::model::{Action, ActionKind};
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
            | Self::ToggleSegmentKind
            | Self::SetStrokeColor
            | Self::SetStrokeWidth
            | Self::SetStrokeOpacity
            | Self::SetFillColor
            | Self::SetFillOpacity
            | Self::ToggleNoFill
            | Self::SetObjectPosition
            | Self::SetObjectSize
            | Self::SetObjectRotation => ActionKind::Document,

            // Editor state changes (selection, tools, history navigation)
            Self::SelectAll
            | Self::DeselectAll
            | Self::ActivatePenTool
            | Self::ActivateSelectTool
            | Self::ActivateMyNewTool
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
    /// use gauss_core::model::Action;
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
            Self::ActivateMyNewTool => "My New Tool",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::SelectionUndo => "Selection Undo",
            Self::SelectionRedo => "Selection Redo",
            Self::SetStrokeColor => "Set Stroke Colour",
            Self::SetStrokeWidth => "Set Stroke Width",
            Self::SetStrokeOpacity => "Set Stroke Opacity",
            Self::SetFillColor => "Set Fill Colour",
            Self::SetFillOpacity => "Set Fill Opacity",
            Self::ToggleNoFill => "Toggle No Fill",
            Self::SetObjectPosition => "Set Position",
            Self::SetObjectSize => "Set Size",
            Self::SetObjectRotation => "Set Rotation",
        }
    }

    /// Return the internal identifier for this action.
    ///
    /// The identifier is the enum variant name and is suitable for:
    ///
    /// - Command linkage metadata
    /// - Serialization and dispatch tables
    /// - Log and telemetry labels
    ///
    /// # Returns
    ///
    /// A static string containing the internal action identifier.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss_core::model::Action;
    ///
    /// assert_eq!(Action::DeleteSelection.identifier(), "DeleteSelection");
    /// assert_eq!(Action::Undo.identifier(), "Undo");
    /// ```
    #[must_use]
    pub const fn identifier(&self) -> &'static str {
        match self {
            Self::DeleteSelection => "DeleteSelection",
            Self::InsertAnchorOnSegment => "InsertAnchorOnSegment",
            Self::DeleteSelectedAnchors => "DeleteSelectedAnchors",
            Self::RaiseSelection => "RaiseSelection",
            Self::LowerSelection => "LowerSelection",
            Self::ToggleSegmentKind => "ToggleSegmentKind",
            Self::SelectAll => "SelectAll",
            Self::DeselectAll => "DeselectAll",
            Self::ActivatePenTool => "ActivatePenTool",
            Self::ActivateSelectTool => "ActivateSelectTool",
            Self::ActivateMyNewTool => "ActivateMyNewTool",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::SelectionUndo => "SelectionUndo",
            Self::SelectionRedo => "SelectionRedo",
            Self::SetStrokeColor => "SetStrokeColor",
            Self::SetStrokeWidth => "SetStrokeWidth",
            Self::SetStrokeOpacity => "SetStrokeOpacity",
            Self::SetFillColor => "SetFillColor",
            Self::SetFillOpacity => "SetFillOpacity",
            Self::ToggleNoFill => "ToggleNoFill",
            Self::SetObjectPosition => "SetObjectPosition",
            Self::SetObjectSize => "SetObjectSize",
            Self::SetObjectRotation => "SetObjectRotation",
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
    /// use gauss_core::model::Action;
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
                | Self::SetStrokeColor
                | Self::SetStrokeWidth
                | Self::SetStrokeOpacity
                | Self::SetFillColor
                | Self::SetFillOpacity
                | Self::ToggleNoFill
                | Self::SetObjectPosition
                | Self::SetObjectSize
                | Self::SetObjectRotation
        )
    }
}
