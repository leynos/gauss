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
//! use gauss_core::model::{Action, ActionKind};
//!
//! let action = Action::DeleteSelection;
//! assert_eq!(action.kind(), ActionKind::Document);
//! assert_eq!(action.name(), "Delete Selection");
//! assert!(action.requires_selection());
//! ```

#[path = "action_payloads.rs"]
mod action_payloads;

pub use action_payloads::{
    Color, Degrees, Dimensions, Opacity, Point, Points, Position, Rgb8, Rotation, Size,
    StrokeWidth, UnitF32,
};

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
/// use gauss_core::model::Action;
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

    // === Style mutations ===
    /// Set the stroke colour of the selected shapes.
    SetStrokeColor(Color),

    /// Set the stroke width of the selected shapes.
    SetStrokeWidth(StrokeWidth),

    /// Set the stroke opacity of the selected shapes.
    SetStrokeOpacity(Opacity),

    /// Set the fill colour of the selected shapes.
    SetFillColor(Color),

    /// Set the fill opacity of the selected shapes.
    SetFillOpacity(Opacity),

    /// Toggle whether the selected shapes have no fill.
    ToggleNoFill,

    // === Transform mutations ===
    /// Set the position of the selected shapes.
    SetObjectPosition(Position),

    /// Set the size of the selected shapes.
    SetObjectSize(Size),

    /// Set the rotation of the selected shapes.
    SetObjectRotation(Rotation),
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;
