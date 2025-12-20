//! Phase 0 UI shell.
//!
//! This module intentionally contains "just enough UI" to validate that GPUI is
//! wired up correctly and to allow incremental integration tests using GPUI's
//! `TestAppContext`.

mod anchor_edit;
mod draw;
mod file_dialogs;
mod header;
mod input;
mod manipulate;
mod reorder;
mod segment_toggle;
mod selection_history;
mod style_controls;
mod view;

use std::path::{Path, PathBuf};

use gpui::prelude::*;
use gpui_component::history::History;

use crate::model::{Document, PaintStyle, Selection, ShapeId, Vec2, Viewport};

use super::phase0_support::demo_document;

use self::{draw::DrawEdgeMode, file_dialogs::OpenPromptMode};

/// Keymap context used for Phase 0 shell bindings.
///
/// GPUI key bindings are dispatched relative to an element's key context. Phase
/// 0 sets this context on the root `div()` for the shell so global editor
/// shortcuts (for example, `Tab` to toggle the draw edge mode) work even when a
/// child element holds focus.
///
/// Note: this string must be valid `gpui::KeyContext` syntax, which accepts
/// identifiers (letters/digits), plus `_` and `-`. Avoid `.` here: it is not a
/// valid identifier character in GPUI key contexts.
pub const KEY_CONTEXT: &str = "gauss-phase0";

/// Trigger an “Open…” workflow for loading a document from disk.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct OpenSvg;

/// Trigger a “Save…” workflow for the current document.
///
/// Phase 0 uses this action purely to validate that:
///
/// - action dispatch is wired end-to-end, and
/// - platform file prompts can be exercised in headless GPUI tests.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct SaveSvg;

/// Toggle the draw edge mode (Line vs Bézier auto) or, in manipulate mode,
/// toggle the kind of any selected segments.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct ToggleEdgeMode;

/// Register Phase 0 shell key bindings on the application keymap.
///
/// This is intentionally kept small and explicit so we can evolve from Phase 0
/// "direct key handling" into more idiomatic GPUI keymaps as the editor grows.
pub fn bind_keymap(app: &mut gpui::App) {
    use gpui::KeyBinding;

    app.bind_keys([KeyBinding::new("tab", ToggleEdgeMode, Some(KEY_CONTEXT))]);
}

/// Minimal root view for Phase 0.
///
/// This view exists to keep a stable "entrypoint view" for the `PoC` while the
/// real UI is built out.
pub struct Phase0Shell {
    focus_handle: gpui::FocusHandle,
    did_focus: bool,
    did_request_quit: bool,
    open_prompt_mode: OpenPromptMode,
    document: Document,
    viewport: Viewport,
    current_style: PaintStyle,
    tool_mode: draw::ToolMode,
    edge_mode: DrawEdgeMode,
    draw_active_shape: Option<ShapeId>,
    document_history: History<draw::DocHistoryItem>,
    selection_history: History<selection_history::SelectionHistoryItem>,
    selection: Selection,
    drag_state: Option<manipulate::DragState>,
    last_canvas_click_screen: Option<Vec2>,
    last_saved_path: Option<PathBuf>,
    last_save_error: Option<String>,
    last_opened_path: Option<PathBuf>,
    last_open_error: Option<String>,
    stroke_picker: Option<gpui::Entity<gpui_component::color_picker::ColorPickerState>>,
    fill_picker: Option<gpui::Entity<gpui_component::color_picker::ColorPickerState>>,
    style_picker_subscriptions: Vec<gpui::Subscription>,
}

impl Phase0Shell {
    /// Construct a new shell.
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            did_focus: false,
            did_request_quit: false,
            open_prompt_mode: OpenPromptMode::Native,
            document: demo_document(),
            viewport: Viewport::new(),
            current_style: PaintStyle::new(Some(crate::model::Rgba::new(0, 0, 0, 255)), 2.0, None),
            tool_mode: draw::ToolMode::Draw,
            edge_mode: DrawEdgeMode::Line,
            draw_active_shape: None,
            document_history: History::new(),
            selection_history: History::new(),
            selection: Selection::empty(),
            drag_state: None,
            last_canvas_click_screen: None,
            last_saved_path: None,
            last_save_error: None,
            last_opened_path: None,
            last_open_error: None,
            stroke_picker: None,
            fill_picker: None,
            style_picker_subscriptions: Vec::new(),
        }
    }

    /// Construct a new shell configured for headless `#[gpui::test]` tests.
    ///
    /// This differs from [`Self::new`] only in how it triggers the file dialog
    /// for “Open…”.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn new_for_tests(cx: &mut Context<Self>) -> Self {
        Self {
            open_prompt_mode: OpenPromptMode::TestNewPath,
            ..Self::new(cx)
        }
    }

    /// Return the last path selected by the platform save prompt, if any.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn last_saved_path(&self) -> Option<&Path> {
        self.last_saved_path.as_deref()
    }

    /// Return the last path selected by the platform open prompt, if any.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn last_opened_path(&self) -> Option<&Path> {
        self.last_opened_path.as_deref()
    }

    /// Return the current document.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real UI.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub const fn document(&self) -> &Document {
        &self.document
    }

    /// Return the current viewport.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub const fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// Return the current selection.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub const fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Replace the entire document.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI. It deliberately does not attempt to
    /// preserve history; callers that need undo/redo should drive changes via
    /// editor operations instead.
    #[cfg(any(test, feature = "test-support"))]
    pub fn replace_document_for_tests(&mut self, document: Document) {
        self.document = document;
        self.drag_state = None;
        self.draw_active_shape = None;
    }

    /// Replace the current selection.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI. Selection history is not updated by this
    /// helper.
    #[cfg(any(test, feature = "test-support"))]
    pub fn replace_selection_for_tests(&mut self, selection: Selection) {
        self.selection = selection;
        self.drag_state = None;
    }

    /// Return whether a drag gesture is currently active.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    /// Return whether a quit request has been triggered from the UI.
    ///
    /// This exists to keep `#[gpui::test]` assertions stable on the test
    /// platform, which does not necessarily exit when `App::quit()` is invoked.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub const fn did_request_quit(&self) -> bool {
        self.did_request_quit
    }

    /// Return the current mode indicator line as shown in the UI.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[cfg(any(test, feature = "test-support"))]
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
    #[cfg(any(test, feature = "test-support"))]
    pub const fn enter_manipulate_mode_for_tests(&mut self) {
        self.tool_mode = draw::ToolMode::Manipulate;
        self.draw_active_shape = None;
    }

    /// Return the last canvas click position in screen coordinates.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub const fn last_canvas_click_screen(&self) -> Option<Vec2> {
        self.last_canvas_click_screen
    }
}
