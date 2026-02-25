//! Unified engine state for the Gauss editor.
//!
//! `EngineState` is the single source of truth for all editor state, per
//! guiding principle section 2 from the architecture document:
//!
//! > "The document (and editor state such as selection, tool mode, viewport)
//! > must live in **engine state**, not in the view layer."
//!
//! This module is GPUI-independent for testability and scripting.
//! Document history is owned here using `DocumentUndoHistory` (model layer,
//! backed by `undo_2`); selection history remains in the UI layer using
//! `gpui_component::History`.
//!
//! # Examples
//!
//! ```rust
//! use gauss::model::{Document, EdgeMode, EngineState, ToolMode};
//!
//! // Create empty state
//! let state = EngineState::new();
//! assert!(state.document.is_empty());
//! assert!(state.selection.is_empty());
//! assert_eq!(state.tool_mode, ToolMode::Draw);
//! assert_eq!(state.document_history_len(), 0);
//!
//! // Create state with a document
//! let doc = Document::default();
//! let state = EngineState::with_document(doc);
//! assert_eq!(state.document_history_len(), 0);
//! ```

use crate::model::{
    Command, Document, EdgeMode, PaintStyle, ResizeAnchor, Rgba, Selection, ShapeId, ToolMode,
    UserError, Viewport,
};

use super::{DocumentUndoHistory, HistoryError, ResourceStore, StyleStore};

/// Unified state for the Gauss editor.
///
/// `EngineState` consolidates all editor state into a single structure:
///
/// - **Document state**: shapes, geometry, styles
/// - **Editor state**: selection, viewport, tool mode
/// - **Resources**: shared gradients, patterns, symbols (future)
/// - **Styles**: named style presets (future)
///
/// This structure serves as the single source of truth for the editor,
/// enabling consistent state access from UI, scripting, and tests.
///
/// # GPUI Independence
///
/// `EngineState` has no GPUI dependencies. This allows:
///
/// - Pure unit tests without GPUI context
/// - Scripting access (`RustPython` integration)
/// - Potential for multiple frontends
///
/// # History
///
/// Document undo/redo history is owned here via `DocumentUndoHistory` in
/// the model layer (backed by `undo_2`). Selection history remains in
/// the view layer using `gpui_component::History`. See ADR-002 for
/// rationale.
pub struct EngineState {
    /// The document containing all shapes and their geometry.
    pub document: Document,

    /// The current selection (shapes, anchors, handles, segments).
    pub selection: Selection,

    /// The viewport transform (pan and zoom).
    pub viewport: Viewport,

    /// The active tool mode (Draw or Manipulate).
    pub tool_mode: ToolMode,

    /// The edge mode for new path segments (Line or Bezier auto).
    pub edge_mode: EdgeMode,

    /// The shape currently being drawn (None if not in active draw).
    pub active_path: Option<ShapeId>,

    /// The current style applied to new shapes.
    pub current_style: PaintStyle,

    /// The anchor point for viewport adjustment during window resize.
    pub resize_anchor: ResizeAnchor,

    /// Shared resources (gradients, patterns, symbols).
    pub resources: ResourceStore,

    /// Named style presets.
    pub styles: StyleStore,

    /// Document edit history (undo/redo + grouping).
    document_history: DocumentUndoHistory,

    /// Raw Gauss metadata block content preserved for round-trip fidelity.
    pub gauss_metadata_block: Option<String>,
}

impl EngineState {
    /// Construct a new engine state with default values.
    ///
    /// Creates an empty document, empty selection, default viewport (no pan,
    /// 1x zoom), Draw tool mode, Line edge mode, and black stroke style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::model::EngineState;
    ///
    /// let state = EngineState::new();
    /// assert!(state.document.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            selection: Selection::empty(),
            viewport: Viewport::new(),
            tool_mode: ToolMode::default(),
            edge_mode: EdgeMode::default(),
            active_path: None,
            current_style: default_style(),
            resize_anchor: ResizeAnchor::default(),
            resources: ResourceStore::new(),
            styles: StyleStore::new(),
            document_history: DocumentUndoHistory::new(),
            gauss_metadata_block: None,
        }
    }

    /// Construct engine state with a specific document.
    ///
    /// All other fields are initialised to defaults. This is useful for
    /// loading saved documents or creating test fixtures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gauss::model::{Document, EngineState, Shape};
    ///
    /// let mut doc = Document::default();
    /// // Add shapes to doc...
    /// let state = EngineState::with_document(doc);
    /// ```
    #[must_use]
    pub fn with_document(document: Document) -> Self {
        Self {
            document,
            ..Self::new()
        }
    }

    /// Apply a command to the document and record the resulting inverse.
    ///
    /// # Errors
    ///
    /// Returns a [`UserError`] if the command cannot be applied to the
    /// current document state.
    pub fn apply_document_command(&mut self, command: Command) -> Result<(), UserError> {
        let inverse = command.apply(&mut self.document)?;
        self.document_history.record(command, inverse);
        Ok(())
    }

    /// Undo the most recent document history entry.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError`] when replay fails or an invalid boundary
    /// operation is attempted.
    pub fn undo_document(&mut self) -> Result<(), HistoryError> {
        self.document_history.undo(&mut self.document)
    }

    /// Redo the most recently undone document history entry.
    ///
    /// # Errors
    ///
    /// Returns [`HistoryError`] when replay fails or an invalid boundary
    /// operation is attempted.
    pub fn redo_document(&mut self) -> Result<(), HistoryError> {
        self.document_history.redo(&mut self.document)
    }

    /// Begin a grouped document-history transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when a history group is already active.
    pub fn begin_document_history_group(&mut self) -> Result<(), HistoryError> {
        self.document_history.begin_group()
    }

    /// End the active grouped document-history transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when no history group is active.
    pub fn end_document_history_group(&mut self) -> Result<(), HistoryError> {
        self.document_history.end_group()
    }

    /// Clear all document-history entries and any active group.
    pub fn clear_document_history(&mut self) {
        self.document_history.clear();
    }

    /// Return whether document undo is currently available.
    #[must_use]
    pub fn can_undo_document(&self) -> bool {
        self.document_history.can_undo()
    }

    /// Return whether document redo is currently available.
    #[must_use]
    pub fn can_redo_document(&self) -> bool {
        self.document_history.can_redo()
    }

    /// Return realized document-history depth.
    #[must_use]
    pub fn document_history_len(&self) -> usize {
        self.document_history.len()
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the default style for new shapes.
///
/// Black stroke (2px width), no fill.
const fn default_style() -> PaintStyle {
    PaintStyle::new(Some(Rgba::new(0, 0, 0, 255)), 2.0, None)
}

#[cfg(test)]
mod tests {
    //! Tests for engine state defaults and helpers.

    use super::*;
    use rstest::rstest;

    #[rstest]
    fn new_state_has_empty_document() {
        let state = EngineState::new();
        assert!(state.document.is_empty());
    }

    #[rstest]
    fn new_state_has_empty_selection() {
        let state = EngineState::new();
        assert!(state.selection.is_empty());
    }

    #[rstest]
    fn new_state_has_default_viewport() {
        let state = EngineState::new();
        assert_eq!(state.viewport, Viewport::new());
    }

    #[rstest]
    fn new_state_has_draw_tool_mode() {
        let state = EngineState::new();
        assert_eq!(state.tool_mode, ToolMode::Draw);
    }

    #[rstest]
    fn new_state_has_line_edge_mode() {
        let state = EngineState::new();
        assert_eq!(state.edge_mode, EdgeMode::Line);
    }

    #[rstest]
    fn new_state_has_no_active_path() {
        let state = EngineState::new();
        assert!(state.active_path.is_none());
    }

    #[rstest]
    fn with_document_preserves_document() {
        let mut doc = Document::new();
        // Insert a shape manually to verify preservation
        doc.append_shape(sample_shape());

        let state = EngineState::with_document(doc.clone());
        assert_eq!(state.document.len(), 1);
        assert_eq!(state.document_history_len(), 0);
    }

    #[rstest]
    fn new_state_document_history_starts_empty() {
        let state = EngineState::new();
        assert_eq!(state.document_history_len(), 0);
        assert!(!state.can_undo_document());
        assert!(!state.can_redo_document());
    }

    #[rstest]
    fn document_history_round_trip_via_engine_state() {
        let mut state = EngineState::new();
        state
            .apply_document_command(Command::InsertShape {
                insertion: crate::model::ShapeInsertion {
                    index: 0,
                    shape: sample_shape(),
                },
            })
            .expect("insert shape should succeed");
        assert_eq!(state.document.len(), 1);
        assert_eq!(state.document_history_len(), 1);
        assert!(state.can_undo_document());

        state.undo_document().expect("undo should succeed");
        assert!(state.document.is_empty());
        assert_eq!(state.document_history_len(), 0);
        assert!(state.can_redo_document());

        state.redo_document().expect("redo should succeed");
        assert_eq!(state.document.len(), 1);
        assert_eq!(state.document_history_len(), 1);
    }

    #[rstest]
    fn empty_document_history_undo_redo_is_noop() {
        let mut state = EngineState::new();

        state
            .undo_document()
            .expect("undo on empty history should succeed");
        state
            .redo_document()
            .expect("redo on empty history should succeed");

        assert!(state.document.is_empty());
        assert_eq!(state.document_history_len(), 0);
    }

    #[rstest]
    fn ending_group_without_begin_returns_error() {
        let mut state = EngineState::new();
        assert_eq!(
            state
                .end_document_history_group()
                .expect_err("ending group without begin should fail"),
            HistoryError::NoActiveGroup,
        );
    }

    #[rstest]
    fn clear_document_history_resets_realized_entries() {
        let mut state = EngineState::new();
        state
            .apply_document_command(Command::InsertShape {
                insertion: crate::model::ShapeInsertion {
                    index: 0,
                    shape: sample_shape(),
                },
            })
            .expect("insert shape should succeed");
        assert_eq!(state.document_history_len(), 1);

        state.clear_document_history();
        assert_eq!(state.document_history_len(), 0);
        assert!(!state.can_undo_document());
        assert!(!state.can_redo_document());
    }

    fn sample_shape() -> crate::model::Shape {
        crate::model::Shape {
            id: ShapeId::default(),
            z: 0,
            style: default_style(),
            path: crate::model::PathGeom {
                anchors: vec![],
                segments: vec![],
                closed: false,
                closing_segment: crate::model::SegmentKind::Line,
            },
            name: None,
            locked: false,
            hidden: false,
            gauss_metadata: Vec::new(),
        }
    }
}
