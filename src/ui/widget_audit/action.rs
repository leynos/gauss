//! Audit-only action identifiers for widget capability mapping.
//!
//! This enum is a lightweight, payload-free mirror of
//! [`gauss_core::model::Action`] used by the widget audit to express action
//! linkage without needing to construct runtime action values.

/// Payload-free action identifiers for audit metadata.
///
/// `AuditAction` preserves the same variant names and identifiers as the
/// runtime `gauss_core::model::Action` enum, making it suitable for
/// [`ActionCommandLinkage`](super::ActionCommandLinkage) without requiring
/// dummy payload values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuditAction {
    /// Delete currently selected objects.
    DeleteSelection,

    /// Insert an anchor on the selected segment.
    InsertAnchorOnSegment,

    /// Delete the selected anchors.
    DeleteSelectedAnchors,

    /// Raise selected shapes in the z-order.
    RaiseSelection,

    /// Lower selected shapes in the z-order.
    LowerSelection,

    /// Toggle segment kind between Line and Cubic.
    ToggleSegmentKind,

    /// Select all selectable objects in the document.
    SelectAll,

    /// Clear the current selection.
    DeselectAll,

    /// Activate the Pen (draw) tool.
    ActivatePenTool,

    /// Activate the Selection (manipulate) tool.
    ActivateSelectTool,

    /// Undo the last document change.
    Undo,

    /// Redo the last undone change.
    Redo,

    /// Undo the last selection change.
    SelectionUndo,

    /// Redo the last undone selection change.
    SelectionRedo,

    /// Set the stroke colour of the selected shapes.
    SetStrokeColor,

    /// Set the stroke width of the selected shapes.
    SetStrokeWidth,

    /// Set the stroke opacity of the selected shapes.
    SetStrokeOpacity,

    /// Set the fill colour of the selected shapes.
    SetFillColor,

    /// Set the fill opacity of the selected shapes.
    SetFillOpacity,

    /// Toggle whether the selected shapes have no fill.
    ToggleNoFill,

    /// Set the position of the selected shapes.
    SetObjectPosition,

    /// Set the size of the selected shapes.
    SetObjectSize,

    /// Set the rotation of the selected shapes.
    SetObjectRotation,
}

impl AuditAction {
    /// Return the internal identifier for this action.
    ///
    /// The identifier matches the corresponding `gauss_core::model::Action` variant name.
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
}
