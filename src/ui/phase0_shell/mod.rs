//! Phase 0 UI shell.
//!
//! This module intentionally contains "just enough UI" to validate that GPUI is
//! wired up correctly and to allow incremental integration tests using GPUI's
//! `TestAppContext`.

mod accessibility;
mod anchor_edit;
mod chrome;
mod chrome_palette;
mod chrome_panels;
pub(crate) mod draw;
mod file_dialogs;
mod icon_button;
mod input;
mod manipulate;
mod reorder;
mod resize_border;
mod segment_toggle;
mod selection_edit;
mod selection_history;
mod style_controls;
#[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
mod test_helpers;
mod tool_rail;
mod view;
mod window_controls;

use std::path::PathBuf;

use gpui::prelude::*;
use gpui_component::history::History;

use crate::model::{EngineState, KeyContext, Vec2};

use super::phase0_support::demo_document;

use self::file_dialogs::OpenPromptMode;

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

/// Minimize the current window.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct MinimizeWindow;

/// Toggle maximize/restore state.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct ToggleMaximize;

/// Toggle fullscreen mode.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct ToggleFullscreen;

/// Close the current window.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct CloseWindow;

/// Enter window move mode for keyboard-driven repositioning.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct StartWindowMove;

/// Enter window resize mode for keyboard-driven resizing.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct StartWindowResize;

/// Show the system window menu (for keyboard accessibility).
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct ShowWindowMenu;

/// Register Phase 0 shell key bindings on the application keymap.
///
/// Key bindings follow platform conventions where possible:
///
/// - Window controls use Alt-based shortcuts (cross-platform fallback)
/// - macOS: Cmd+M for minimize, Cmd+Q for quit
/// - Linux: Alt+F7/F8 patterns for move/resize (GNOME/KDE convention)
///
/// Note: Model-layer action bindings (Undo, Redo, `SelectAll`, etc.) are
/// registered separately via [`crate::ui::action_bridge::register_action_bindings`].
pub fn bind_keymap(app: &mut gpui::App) {
    use gpui::KeyBinding;

    let mut bindings = Vec::new();
    for context in KeyContext::all() {
        let ctx = Some(context.as_ref());
        // Cross-platform window controls
        bindings.push(KeyBinding::new("alt-f4", CloseWindow, ctx));
        bindings.push(KeyBinding::new("alt-f9", MinimizeWindow, ctx));
        bindings.push(KeyBinding::new("alt-f10", ToggleMaximize, ctx));
        bindings.push(KeyBinding::new("alt-f11", ToggleFullscreen, ctx));
        bindings.push(KeyBinding::new("alt-space", ShowWindowMenu, ctx));
        bindings.push(KeyBinding::new("alt-f7", StartWindowMove, ctx));
        bindings.push(KeyBinding::new("alt-f8", StartWindowResize, ctx));

        #[cfg(target_os = "macos")]
        {
            bindings.push(KeyBinding::new("cmd-m", MinimizeWindow, ctx));
            bindings.push(KeyBinding::new("cmd-q", CloseWindow, ctx));
            bindings.push(KeyBinding::new("ctrl-cmd-f", ToggleFullscreen, ctx));
        }
    }

    // Editor-specific bindings (not in the model Action enum)
    bindings.push(KeyBinding::new(
        "tab",
        ToggleEdgeMode,
        Some(KeyContext::DrawMode.as_ref()),
    ));

    app.bind_keys(bindings);
}

/// Minimal root view for Phase 0.
///
/// This view exists to keep a stable "entrypoint view" for the `PoC` while the
/// real UI is built out.
///
/// Engine state (document, selection, viewport, tool mode, etc.) is consolidated
/// in the `state` field. This provides a single source of truth for editor state
/// per architecture document section 2.
pub struct Phase0Shell {
    /// Unified engine state (document, selection, viewport, tools).
    state: EngineState,

    // GPUI-specific state (cannot move to EngineState due to dependencies)
    focus_handle: gpui::FocusHandle,
    did_focus: bool,
    did_request_quit: bool,
    open_prompt_mode: OpenPromptMode,

    /// Document edit history (model-layer, GPUI-independent).
    document_history: crate::model::DocumentUndoHistory,
    /// Selection change history (separate from document history).
    selection_history: History<selection_history::SelectionHistoryItem>,

    // Interaction state
    drag_state: Option<manipulate::DragState>,
    last_canvas_click_screen: Option<Vec2>,

    // File I/O state
    last_saved_path: Option<PathBuf>,
    last_save_error: Option<String>,
    last_opened_path: Option<PathBuf>,
    last_open_error: Option<String>,
    last_history_error: Option<String>,

    // Style picker entities (GPUI-dependent)
    stroke_picker: Option<gpui::Entity<gpui_component::color_picker::ColorPickerState>>,
    fill_picker: Option<gpui::Entity<gpui_component::color_picker::ColorPickerState>>,
    style_picker_subscriptions: Vec<gpui::Subscription>,
    did_init_style_pickers: bool,

    /// Previous canvas size for resize anchoring.
    last_viewport_size: Option<gpui::Size<gpui::Pixels>>,
    /// Cached maximized state to trigger re-render on window state change.
    last_maximized_state: Option<bool>,
    /// Test override for maximized state (used to test resize border visibility).
    #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
    test_maximized_override: Option<bool>,
}

impl Phase0Shell {
    /// Construct a new shell.
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: EngineState::with_document(demo_document()),
            focus_handle: cx.focus_handle(),
            did_focus: false,
            did_request_quit: false,
            open_prompt_mode: OpenPromptMode::Native,
            document_history: crate::model::DocumentUndoHistory::new(),
            selection_history: History::new(),
            drag_state: None,
            last_canvas_click_screen: None,
            last_saved_path: None,
            last_save_error: None,
            last_opened_path: None,
            last_open_error: None,
            last_history_error: None,
            stroke_picker: None,
            fill_picker: None,
            style_picker_subscriptions: Vec::new(),
            did_init_style_pickers: false,
            last_viewport_size: None,
            last_maximized_state: None,
            #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
            test_maximized_override: None,
        }
    }

    /// Handle window resize by adjusting viewport pan to maintain anchor.
    pub(super) fn handle_window_resize(&mut self, window: &gpui::Window) {
        let new_size = window.viewport_size();

        if let Some(old_size) = self.last_viewport_size {
            // Only adjust if size actually changed
            if old_size != new_size {
                let old_vec = Vec2::new(f32::from(old_size.width), f32::from(old_size.height));
                let new_vec = Vec2::new(f32::from(new_size.width), f32::from(new_size.height));
                let anchor_factor = self.state.resize_anchor.as_factor();
                self.state
                    .viewport
                    .adjust_pan_for_resize(old_vec, new_vec, anchor_factor);
            }
        }

        self.last_viewport_size = Some(new_size);
    }

    /// Check if the window should be treated as maximized for resize border visibility.
    ///
    /// In test mode, this can be overridden via [`Self::set_maximized_for_tests`].
    /// In production, this queries the actual window state.
    pub(super) fn is_maximized_for_resize_borders(&self, window: &gpui::Window) -> bool {
        #[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
        if let Some(override_value) = self.test_maximized_override {
            return override_value;
        }

        window.is_maximized()
    }

    /// Check if maximized state changed and schedule a re-render if so.
    ///
    /// This ensures resize borders are properly added/removed when the window
    /// is maximized or restored via the window manager (not just our UI).
    /// Returns the current maximized state.
    pub(super) fn check_maximized_state_change(
        &mut self,
        window: &gpui::Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let is_maximized = self.is_maximized_for_resize_borders(window);

        if self.last_maximized_state != Some(is_maximized) {
            self.last_maximized_state = Some(is_maximized);
            // Schedule a re-render to update resize border visibility
            cx.notify();
        }

        is_maximized
    }
}
