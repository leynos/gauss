//! Unified engine state for the Gauss editor.
//!
//! `EngineState` is the single source of truth for all editor state, per
//! guiding principle section 2 from the architecture document:
//!
//! > "The document (and editor state such as selection, tool mode, viewport)
//! > must live in **engine state**, not in the view layer."
//!
//! This module is GPUI-independent for testability and scripting. History
//! stacks remain in the UI layer since they depend on `gpui_component::History`.
//!
//! # Examples
//!
//! ```rust
//! use gauss::model::{Document, EngineState, ToolMode, EdgeMode};
//!
//! // Create empty state
//! let state = EngineState::new();
//! assert!(state.document.is_empty());
//! assert!(state.selection.is_empty());
//! assert_eq!(state.tool_mode, ToolMode::Draw);
//!
//! // Create state with a document
//! let doc = Document::default();
//! let state = EngineState::with_document(doc);
//! ```

use crate::model::{
    Document, EdgeMode, PaintStyle, ResizeAnchor, Rgba, Selection, ShapeId, ToolMode, Viewport,
};

use super::{ResourceStore, StyleStore};

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
/// Undo/redo history is managed separately in the UI layer since it uses
/// `gpui_component::History`. The engine state represents the current
/// snapshot, not the full edit history.
#[derive(Clone, Debug, PartialEq)]
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
        doc.append_shape(crate::model::Shape {
            id: ShapeId::default(),
            z: 0,
            style: default_style(),
            path: crate::model::PathGeom {
                anchors: vec![],
                segments: vec![],
                closed: false,
                closing_segment: crate::model::SegmentKind::Line,
            },
        });

        let state = EngineState::with_document(doc.clone());
        assert_eq!(state.document.len(), 1);
    }

    #[rstest]
    fn state_is_clone() {
        let state = EngineState::new();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[rstest]
    fn state_default_equals_new() {
        assert_eq!(EngineState::default(), EngineState::new());
    }
}
