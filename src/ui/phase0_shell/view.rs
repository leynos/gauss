//! Layout and rendering for the Phase 0 shell.

use std::path::Path;

use gpui::{Window, div, prelude::*, white};

use crate::i18n::MessageId;
use crate::ui::action_bridge::context_for_tool_mode;

use super::{
    ExportSvgWebReady, OpenSvg, Phase0Shell, SaveSvg, ToggleEdgeMode,
    chrome_palette::chrome_border, draw, file_dialogs,
};

/// File status variants for status line display.
///
/// These variants represent the possible file-related status messages
/// in order of precedence (highest to lowest).
#[derive(Debug, Clone, PartialEq)]
pub(super) enum FileStatus<'a> {
    /// History operation error (highest priority).
    HistoryError { error: &'a str },
    /// Generic shell operation error.
    ShellError { error: &'a str },
    /// Save operation failed.
    SaveFailed { error: &'a str },
    /// Open operation failed.
    OpenFailed { error: &'a str },
    /// File was saved successfully.
    Saved { path: &'a Path },
    /// File was opened successfully.
    Opened { path: &'a Path },
}

impl FileStatus<'_> {
    /// Convert the file status to a display string using localization.
    fn to_display_string(&self, shell: &Phase0Shell) -> String {
        match self {
            Self::HistoryError { error } => {
                let template = lookup_template(
                    shell,
                    &MessageId::status_history_error(),
                    "History error: {error}",
                );
                template.replace("{error}", error)
            }
            Self::ShellError { error } => {
                let template = lookup_template(
                    shell,
                    &MessageId::status_shell_error(),
                    "Shell error: {error}",
                );
                template.replace("{error}", error)
            }
            Self::SaveFailed { error } => {
                let template = lookup_template(
                    shell,
                    &MessageId::status_save_failed(),
                    "Save failed: {error}",
                );
                template.replace("{error}", error)
            }
            Self::OpenFailed { error } => {
                let template = lookup_template(
                    shell,
                    &MessageId::status_open_failed(),
                    "Open failed: {error}",
                );
                template.replace("{error}", error)
            }
            Self::Saved { path } => {
                let template = lookup_template(shell, &MessageId::status_saved(), "Saved: {path}");
                template.replace("{path}", &path.display().to_string())
            }
            Self::Opened { path } => {
                let template =
                    lookup_template(shell, &MessageId::status_opened(), "Opened: {path}");
                template.replace("{path}", &path.display().to_string())
            }
        }
    }
}

fn lookup_template(shell: &Phase0Shell, id: &MessageId, fallback: &str) -> String {
    shell
        .localizer
        .lookup(&shell.locale, id)
        .unwrap_or_else(|_| fallback.to_owned())
}

impl Phase0Shell {
    /// Return a localised mode status string for the current tool state.
    ///
    /// The string includes the active tool mode, the edge mode when drawing,
    /// and the maximised-window indicator when the shell last observed a
    /// maximised window state.
    pub(super) fn mode_status_line(&self) -> String {
        let tool_label = self.localized_tool_mode_label();
        let edge_label = self.localized_edge_mode_label();

        let maximized_indicator = if self.last_maximized_state == Some(true) {
            lookup_template(self, &MessageId::status_maximized(), " [MAX]")
        } else {
            String::new()
        };
        match self.state.tool_mode {
            draw::ToolMode::Draw => {
                let template = lookup_template(
                    self,
                    &MessageId::tool_status_mode_with_edge(),
                    "Mode: {tool} ({edge})",
                );
                template
                    .replace("{tool}", &tool_label)
                    .replace("{edge}", &edge_label)
                    + &maximized_indicator
            }
            draw::ToolMode::Manipulate => {
                let template =
                    lookup_template(self, &MessageId::tool_status_mode(), "Mode: {tool}");
                template.replace("{tool}", &tool_label) + &maximized_indicator
            }
        }
    }

    fn localized_tool_mode_label(&self) -> String {
        super::i18n_helpers::localized_tool_mode_label(
            self.state.tool_mode,
            &self.localizer,
            &self.locale,
        )
    }

    fn localized_edge_mode_label(&self) -> String {
        super::i18n_helpers::localized_edge_mode_label(
            self.state.edge_mode,
            &self.localizer,
            &self.locale,
        )
    }

    /// Determine the current file status based on shell state.
    ///
    /// Returns the highest-priority file status variant if any status
    /// condition is present, or `None` if there is no status to display.
    pub(super) fn current_file_status(&self) -> Option<FileStatus<'_>> {
        if let Some(error) = self.last_history_error.as_deref() {
            return Some(FileStatus::HistoryError { error });
        }

        if let Some(error) = self.shell_status_error.as_deref() {
            return Some(FileStatus::ShellError { error });
        }

        if let Some(error) = self.last_save_error.as_deref() {
            return Some(FileStatus::SaveFailed { error });
        }

        if let Some(error) = self.last_open_error.as_deref() {
            return Some(FileStatus::OpenFailed { error });
        }

        if let Some(path) = self.last_saved_path.as_deref() {
            return Some(FileStatus::Saved { path });
        }

        if let Some(path) = self.last_opened_path.as_deref() {
            return Some(FileStatus::Opened { path });
        }

        None
    }

    /// Return the highest-priority localised file status string, if active.
    ///
    /// Returns `None` when there is no history, shell, save, open, or recent
    /// file operation status to display.
    pub(super) fn file_status_line(&self) -> Option<String> {
        self.current_file_status()
            .map(|status| status.to_display_string(self))
    }

    /// Build the interactive canvas area for document rendering and input.
    ///
    /// The returned element owns the canvas event handlers for pointer,
    /// navigation, click, and scroll interactions.
    pub(super) fn canvas_area(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .id("phase0-canvas")
            .debug_selector(|| "#phase0-canvas".to_owned())
            .flex()
            .flex_1()
            .border_1()
            .border_color(chrome_border())
            .bg(white())
            .rounded_md()
            .overflow_hidden()
            .occlude()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::canvas_mouse_down),
            )
            .on_mouse_down(
                gpui::MouseButton::Navigate(gpui::NavigationDirection::Back),
                cx.listener(Self::canvas_navigate_mouse_down),
            )
            .on_mouse_down(
                gpui::MouseButton::Navigate(gpui::NavigationDirection::Forward),
                cx.listener(Self::canvas_navigate_mouse_down),
            )
            .on_mouse_move(cx.listener(Self::canvas_mouse_move))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::canvas_mouse_up))
            .on_mouse_up_out(gpui::MouseButton::Left, cx.listener(Self::canvas_mouse_up))
            .on_click(cx.listener(Self::canvas_click))
            .on_scroll_wheel(cx.listener(Self::canvas_scroll_wheel))
            .child(super::super::canvas_paint::canvas_for_document(
                &self.state.document,
                &self.state.selection,
                self.state.viewport,
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

    fn canvas_navigate_mouse_down(
        shell: &mut Self,
        event: &gpui::MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if shell.handle_navigation_mouse_down(event) {
            cx.notify();
            cx.stop_propagation();
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
        let did_change = super::super::viewport_input::apply_scroll_wheel_event(
            &mut shell.state.viewport,
            event,
            line_height,
        );

        if did_change {
            cx.notify();
            cx.stop_propagation();
        }
    }
}

impl Render for Phase0Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        if !self.did_focus {
            self.did_focus = true;
            window.focus(&self.focus_handle);
        }

        if !self.did_init_style_pickers {
            self.ensure_style_pickers(window, cx);
            self.did_init_style_pickers = true;
        }

        // Track viewport size changes and adjust pan to maintain anchor point
        self.handle_window_resize(window);
        self.sync_a11y_tree();

        // Mode-specific key contexts are applied based on the active tool mode.
        // Global shortcuts are registered for all contexts via the action bridge.

        let root = div()
            .size_full()
            // Apply the active tool context so mode-specific shortcuts resolve.
            .key_context(context_for_tool_mode(self.state.tool_mode).as_ref())
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .on_key_down(
                cx.listener(|shell: &mut Self, event: &gpui::KeyDownEvent, _, view_cx| {
                    shell.handle_key_down(event, view_cx);
                }),
            )
            .on_action(cx.listener(|shell: &mut Self, _: &OpenSvg, w, action_cx| {
                file_dialogs::request_open(shell.open_prompt_mode, w, action_cx);
            }))
            .on_action(cx.listener(|_: &mut Self, _: &SaveSvg, w, action_cx| {
                file_dialogs::request_save(w, action_cx);
            }))
            .on_action(
                cx.listener(|_: &mut Self, _: &ExportSvgWebReady, w, action_cx| {
                    file_dialogs::request_web_ready_export(w, action_cx);
                }),
            )
            .on_action(
                cx.listener(|shell: &mut Self, _: &ToggleEdgeMode, _, action_cx| {
                    shell.handle_tab_action(action_cx);
                }),
            );

        // Bind action handlers for model actions and window controls
        let root_with_actions = Self::bind_model_actions(root, cx);

        // Check for maximized state change and trigger re-render if needed
        let is_maximized = self.check_maximized_state_change(window, cx);
        Self::bind_window_actions(root_with_actions, cx).child(self.chrome_view(is_maximized, cx))
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
