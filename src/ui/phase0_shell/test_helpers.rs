//! Test helper methods for Phase 0 shell.
//!
//! This module contains test-only methods that are gated behind the
//! `test-support` feature or test compilation. They provide controlled access
//! to internal state for headless GPUI tests.

use std::path::Path;

use crate::model::history::HistoryError;
use crate::model::{
    Command, Document, ResourceStore, Selection, ShapeId, UserError, Vec2, Viewport,
};

use super::{Phase0Shell, draw, file_dialogs::OpenPromptMode};

impl Phase0Shell {
    /// Construct a new shell configured for headless `#[gpui::test]` tests.
    ///
    /// This differs from [`Self::new`] only in how it triggers the file dialog
    /// for "Open…".
    #[must_use]
    pub fn new_for_tests(cx: &mut gpui::Context<Self>) -> Self {
        Self {
            open_prompt_mode: OpenPromptMode::TestNewPath,
            ..Self::new(cx)
        }
    }

    /// Return the last path selected by the platform save prompt, if any.
    #[must_use]
    pub fn last_saved_path(&self) -> Option<&Path> {
        self.last_saved_path.as_deref()
    }

    /// Return the last path selected by the platform open prompt, if any.
    #[must_use]
    pub fn last_opened_path(&self) -> Option<&Path> {
        self.last_opened_path.as_deref()
    }

    /// Return the latest open error, if any.
    #[must_use]
    pub fn last_open_error(&self) -> Option<&str> {
        self.last_open_error.as_deref()
    }

    /// Return the latest save error, if any.
    #[must_use]
    pub fn last_save_error(&self) -> Option<&str> {
        self.last_save_error.as_deref()
    }

    /// Return the current document.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real UI.
    #[must_use]
    pub const fn document(&self) -> &Document {
        &self.state.document
    }

    /// Return the current shared resource store.
    #[must_use]
    pub const fn resources(&self) -> &ResourceStore {
        &self.state.resources
    }

    /// Return the current viewport.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn viewport(&self) -> Viewport {
        self.state.viewport
    }

    /// Return the current selection.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn selection(&self) -> &Selection {
        &self.state.selection
    }

    /// Return the active shape identifier, if any.
    ///
    /// This is intended for headless tests that validate tool mode switches.
    #[must_use]
    pub const fn draw_active_shape_for_tests(&self) -> Option<ShapeId> {
        self.state.active_path
    }

    /// Override the active shape during tests.
    ///
    /// This is used to verify that switching to manipulate mode clears the
    /// active draw state.
    pub const fn set_draw_active_shape_for_tests(&mut self, shape: Option<ShapeId>) {
        self.state.active_path = shape;
    }

    /// Override the maximized state for tests.
    ///
    /// This controls whether resize borders are visible. Set to `Some(true)`
    /// to simulate a maximized window (no resize borders), `Some(false)` for
    /// a normal window (resize borders visible), or `None` to use the actual
    /// window state.
    pub const fn set_maximized_for_tests(&mut self, maximized: Option<bool>) {
        self.test_maximized_override = maximized;
    }

    /// Replace the entire document.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI. It deliberately does not attempt to
    /// preserve history; callers that need undo/redo should drive changes via
    /// editor operations instead.
    pub fn replace_document_for_tests(&mut self, document: Document) {
        self.state.document = document;
        self.drag_state = None;
        self.state.active_path = None;
    }

    /// Replace the current selection.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI. Selection history is not updated by this
    /// helper.
    pub fn replace_selection_for_tests(&mut self, selection: Selection) {
        self.state.selection = selection;
        self.drag_state = None;
    }

    /// Return whether a drag gesture is currently active.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    /// Return whether a quit request has been triggered from the UI.
    ///
    /// This exists to keep `#[gpui::test]` assertions stable on the test
    /// platform, which does not necessarily exit when `App::quit()` is invoked.
    #[must_use]
    pub const fn did_request_quit(&self) -> bool {
        self.did_request_quit
    }

    /// Return the current mode indicator line as shown in the UI.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub fn mode_status_line_for_tests(&self) -> String {
        self.mode_status_line()
    }

    /// Force manipulate mode for headless tests.
    ///
    /// GPUI's headless harness does not always guarantee that keyboard focus is
    /// established before the test sends synthetic key events. This helper
    /// allows tests to set the tool mode explicitly without relying on
    /// `Escape` dispatch.
    pub fn enter_manipulate_mode_for_tests(&mut self) {
        self.set_tool_mode(draw::ToolMode::Manipulate);
    }

    /// Return the last canvas click position in screen coordinates.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn last_canvas_click_screen(&self) -> Option<Vec2> {
        self.last_canvas_click_screen
    }

    /// Check if the window should be treated as maximized for resize border visibility.
    ///
    /// This exposes `is_maximized_for_resize_borders` for tests.
    #[must_use]
    pub fn is_maximized_for_resize_borders_for_tests(&self, window: &gpui::Window) -> bool {
        self.is_maximized_for_resize_borders(window)
    }

    /// Check if the current tool mode is Draw.
    #[must_use]
    pub const fn is_draw_mode(&self) -> bool {
        matches!(self.state.tool_mode, draw::ToolMode::Draw)
    }

    /// Check if the current tool mode is Manipulate.
    #[must_use]
    pub const fn is_manipulate_mode(&self) -> bool {
        matches!(self.state.tool_mode, draw::ToolMode::Manipulate)
    }

    /// Check if the current edge mode is Line.
    #[must_use]
    pub const fn is_line_edge_mode(&self) -> bool {
        matches!(self.state.edge_mode, draw::DrawEdgeMode::Line)
    }

    /// Check if the current edge mode is Bezier (auto).
    #[must_use]
    pub const fn is_bezier_edge_mode(&self) -> bool {
        matches!(self.state.edge_mode, draw::DrawEdgeMode::BezierAuto)
    }

    /// Apply a command through the shell's history system.
    ///
    /// This is the test entry point for building undo history. The command
    /// is applied to the document and recorded in the history stack.
    ///
    /// # Errors
    ///
    /// Returns a [`UserError`] if the command fails to apply to the document
    /// (e.g., the target shape does not exist).
    pub fn apply_command_for_tests(&mut self, command: Command) -> Result<(), UserError> {
        self.apply_command(command)
    }

    /// Begin a grouped document command transaction for headless tests.
    ///
    /// Commands applied after this call are accumulated into a single
    /// undoable history entry until
    /// [`Self::end_document_command_group_for_tests`] is called.
    ///
    /// # Errors
    ///
    /// Returns an error when a group is already active.
    pub fn begin_document_command_group_for_tests(&mut self) -> Result<(), HistoryError> {
        self.state.begin_document_history_group()
    }

    /// End the active grouped document command transaction for headless tests.
    ///
    /// Closing an empty group is a no-op. Closing a non-empty group commits one
    /// undoable history entry.
    ///
    /// # Errors
    ///
    /// Returns an error when no group is active.
    pub fn end_document_command_group_for_tests(&mut self) -> Result<(), HistoryError> {
        self.state.end_document_history_group()
    }

    /// Return a mutable reference to the document for direct mutation.
    ///
    /// Unlike [`replace_document_for_tests`], this preserves the history stack,
    /// allowing tests to create invalid states that trigger history errors.
    pub const fn document_mut_for_tests(&mut self) -> &mut Document {
        &mut self.state.document
    }

    /// Return a mutable reference to shared resources for direct mutation.
    ///
    /// This helper supports headless tests that need to exercise validation
    /// behaviour (for example, dangling paint references).
    pub const fn resources_mut_for_tests(&mut self) -> &mut ResourceStore {
        &mut self.state.resources
    }

    /// Return the current Gauss metadata block, if any.
    #[must_use]
    pub fn gauss_metadata_block(&self) -> Option<&str> {
        self.state.gauss_metadata_block.as_deref()
    }

    /// Set the Gauss metadata block for tests.
    pub fn set_gauss_metadata_block_for_tests(&mut self, block: Option<String>) {
        self.state.gauss_metadata_block = block;
    }

    /// Trigger an undo operation through the shell's history system.
    ///
    /// This exercises the full error propagation path through
    /// `apply_history_operation` → `apply_history_group`.
    pub fn undo_document_for_tests(&mut self) {
        self.undo_document();
    }

    /// Trigger a redo operation through the shell's history system.
    ///
    /// This exercises the full error propagation path through
    /// `apply_history_operation` → `apply_history_group`.
    pub fn redo_document_for_tests(&mut self) {
        self.redo_document();
    }

    /// Return the number of realised entries in the document undo history.
    ///
    /// This exposes `DocumentUndoHistory::len()` for headless GPUI tests
    /// that verify the single-entry-per-gesture invariant.
    #[must_use]
    pub fn document_history_len_for_tests(&self) -> usize {
        self.state.document_history_len()
    }

    /// Return the last history error, if any.
    ///
    /// This allows tests to verify that history operation failures are
    /// properly surfaced through the error propagation path.
    #[must_use]
    pub fn last_history_error_for_tests(&self) -> Option<&str> {
        self.last_history_error.as_deref()
    }
}
