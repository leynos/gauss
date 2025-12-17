//! Phase 0 UI shell.
//!
//! This module intentionally contains "just enough UI" to validate that GPUI is
//! wired up correctly and to allow incremental integration tests using GPUI's
//! `TestAppContext`.

mod draw;
mod file_dialogs;
mod input;
mod manipulate;

use std::path::{Path, PathBuf};

use gpui::{Window, div, prelude::*};
use gpui_component::history::History;

use crate::model::{Document, Selection, ShapeId, Vec2, Viewport};

use super::phase0_support::demo_document;

use self::{draw::DrawEdgeMode, file_dialogs::OpenPromptMode};

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

/// Minimal root view for Phase 0.
///
/// This view exists to keep a stable "entrypoint view" for the `PoC` while the
/// real UI is built out.
pub struct Phase0Shell {
    focus_handle: gpui::FocusHandle,
    did_focus: bool,
    open_prompt_mode: OpenPromptMode,
    document: Document,
    viewport: Viewport,
    tool_mode: draw::ToolMode,
    edge_mode: DrawEdgeMode,
    draw_active_shape: Option<ShapeId>,
    document_history: History<draw::DocHistoryItem>,
    selection: Selection,
    drag_state: Option<manipulate::DragState>,
    last_canvas_click_screen: Option<Vec2>,
    last_saved_path: Option<PathBuf>,
    last_save_error: Option<String>,
    last_opened_path: Option<PathBuf>,
    last_open_error: Option<String>,
}

impl Phase0Shell {
    /// Construct a new shell.
    #[must_use]
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            did_focus: false,
            open_prompt_mode: OpenPromptMode::Native,
            document: demo_document(),
            viewport: Viewport::new(),
            tool_mode: draw::ToolMode::Draw,
            edge_mode: DrawEdgeMode::Line,
            draw_active_shape: None,
            document_history: History::new(),
            selection: Selection::empty(),
            drag_state: None,
            last_canvas_click_screen: None,
            last_saved_path: None,
            last_save_error: None,
            last_opened_path: None,
            last_open_error: None,
        }
    }

    /// Construct a new shell configured for headless `#[gpui::test]` tests.
    ///
    /// This differs from [`Self::new`] only in how it triggers the file dialog
    /// for “Open…”.
    #[must_use]
    pub fn new_for_tests(cx: &mut Context<Self>) -> Self {
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

    /// Return the current document.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real UI.
    #[must_use]
    pub const fn document(&self) -> &Document {
        &self.document
    }

    /// Return the current viewport.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// Return the current selection.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Return whether a drag gesture is currently active.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    /// Return the last canvas click position in screen coordinates.
    ///
    /// This is intended for tests and debugging while Phase 0 is still
    /// assembling the real editor UI.
    #[must_use]
    pub const fn last_canvas_click_screen(&self) -> Option<Vec2> {
        self.last_canvas_click_screen
    }

    fn mode_status_line(&self) -> String {
        match self.tool_mode {
            draw::ToolMode::Draw => format!(
                "Mode: {} ({})",
                self.tool_mode.label(),
                self.edge_mode.label()
            ),
            draw::ToolMode::Manipulate => format!("Mode: {}", self.tool_mode.label()),
        }
    }

    fn save_status_line(&self) -> String {
        match (&self.last_saved_path, &self.last_save_error) {
            (_, Some(err)) => format!("Save failed: {err}"),
            (Some(path), None) => format!("Last saved path: {}", path.display()),
            (None, None) => "Last saved path: (none)".to_owned(),
        }
    }

    fn open_status_line(&self) -> String {
        match (&self.last_opened_path, &self.last_open_error) {
            (_, Some(err)) => format!("Open failed: {err}"),
            (Some(path), None) => format!("Last opened path: {}", path.display()),
            (None, None) => "Last opened path: (none)".to_owned(),
        }
    }

    fn canvas_area(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("phase0-canvas")
            .debug_selector(|| "#phase0-canvas".to_owned())
            .flex()
            .flex_1()
            .border_1()
            .rounded_md()
            .occlude()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::canvas_mouse_down),
            )
            .on_mouse_move(cx.listener(Self::canvas_mouse_move))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::canvas_mouse_up))
            .on_mouse_up_out(gpui::MouseButton::Left, cx.listener(Self::canvas_mouse_up))
            .on_click(cx.listener(Self::canvas_click))
            .on_scroll_wheel(cx.listener(Self::canvas_scroll_wheel))
            .child(super::canvas_paint::canvas_for_document(
                &self.document,
                &self.selection,
                self.viewport,
            ))
    }

    fn canvas_mouse_down(
        shell: &mut Self,
        event: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if shell.handle_canvas_mouse_down(event) {
            cx.notify();
        }
    }

    fn canvas_mouse_move(
        shell: &mut Self,
        event: &gpui::MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if shell.handle_canvas_mouse_move(event) {
            cx.notify();
        }
    }

    fn canvas_mouse_up(
        shell: &mut Self,
        event: &gpui::MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if shell.handle_canvas_mouse_up(event) {
            cx.notify();
        }
    }

    fn canvas_click(
        shell: &mut Self,
        event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if shell.handle_canvas_click(event.position()) {
            cx.notify();
            cx.stop_propagation();
        }
    }

    fn canvas_scroll_wheel(
        shell: &mut Self,
        event: &gpui::ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line_height = window.line_height();
        let did_change = super::viewport_input::apply_scroll_wheel_event(
            &mut shell.viewport,
            event,
            line_height,
        );

        if did_change {
            cx.notify();
            cx.stop_propagation();
        }
    }

    fn header_row(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .child("Gauss PoC: Phase 0 shell")
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Self::open_button(cx))
                    .child(Self::save_button(cx)),
            )
    }

    fn open_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("open-button")
            .debug_selector(|| "#open-button".to_owned())
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .on_click(cx.listener(
                |shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                    file_dialogs::request_open(shell.open_prompt_mode, click_window, click_cx);
                },
            ))
            .child("Open…")
    }

    fn save_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("save-button")
            .debug_selector(|| "#save-button".to_owned())
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .on_click(cx.listener(
                |_shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                    file_dialogs::request_save(click_window, click_cx);
                },
            ))
            .child("Save…")
    }
}

impl Render for Phase0Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        if !self.did_focus {
            self.did_focus = true;
            window.focus(&self.focus_handle);
        }

        div()
            .p_4()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .gap_4()
            .on_key_down(
                cx.listener(|shell: &mut Self, event: &gpui::KeyDownEvent, _, view_cx| {
                    shell.handle_key_down(event, view_cx);
                }),
            )
            .child(Self::header_row(cx))
            .on_action(
                cx.listener(|shell: &mut Self, _: &OpenSvg, action_window, action_cx| {
                    file_dialogs::request_open(shell.open_prompt_mode, action_window, action_cx);
                }),
            )
            .on_action(
                cx.listener(|_shell: &mut Self, _: &SaveSvg, action_window, action_cx| {
                    file_dialogs::request_save(action_window, action_cx);
                }),
            )
            .child(
                "This view validates action wiring, native open/save prompts, and canvas \
                 painting, while Phase 0 assembles the real editor UI.",
            )
            .child(self.canvas_area(cx))
            .child(self.mode_status_line())
            .child(self.save_status_line())
            .child(self.open_status_line())
    }
}
