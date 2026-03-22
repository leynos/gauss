//! Layout and rendering for the Phase 0 shell.

use gpui::{Window, div, prelude::*, white};

use crate::ui::action_bridge::context_for_tool_mode;

use super::{
    ExportSvgWebReady, OpenSvg, Phase0Shell, SaveSvg, ToggleEdgeMode,
    chrome_palette::chrome_border, draw, file_dialogs,
};

impl Phase0Shell {
    pub(super) fn mode_status_line(&self) -> String {
        let tool_label = self.localized_tool_mode_label();
        let edge_label = self.localized_edge_mode_label();

        let maximized_indicator = if self.last_maximized_state == Some(true) {
            " [MAX]"
        } else {
            ""
        };
        match self.state.tool_mode {
            draw::ToolMode::Draw => {
                format!("Mode: {tool_label} ({edge_label}){maximized_indicator}")
            }
            draw::ToolMode::Manipulate => {
                format!("Mode: {tool_label}{maximized_indicator}")
            }
        }
    }

    fn localized_tool_mode_label(&self) -> String {
        let message_id = match self.state.tool_mode {
            draw::ToolMode::Draw => crate::i18n::MessageId::tool_mode_draw(),
            draw::ToolMode::Manipulate => crate::i18n::MessageId::tool_mode_manipulate(),
        };
        self.localizer
            .lookup(&self.locale, &message_id)
            .unwrap_or_else(|_| self.state.tool_mode.label().to_owned())
    }

    fn localized_edge_mode_label(&self) -> String {
        let message_id = match self.state.edge_mode {
            draw::DrawEdgeMode::Line => crate::i18n::MessageId::edge_mode_line(),
            draw::DrawEdgeMode::BezierAuto => crate::i18n::MessageId::edge_mode_bezier_auto(),
        };
        self.localizer
            .lookup(&self.locale, &message_id)
            .unwrap_or_else(|_| self.state.edge_mode.label().to_owned())
    }

    pub(super) fn file_status_line(&self) -> Option<String> {
        if let Some(error) = self.last_history_error.as_deref() {
            return Some(format!("History error: {error}"));
        }

        if let Some(error) = self.last_save_error.as_deref() {
            return Some(format!("Save failed: {error}"));
        }

        if let Some(error) = self.last_open_error.as_deref() {
            return Some(format!("Open failed: {error}"));
        }

        if let Some(path) = self.last_saved_path.as_deref() {
            return Some(format!("Saved: {}", path.display()));
        }

        if let Some(path) = self.last_opened_path.as_deref() {
            return Some(format!("Opened: {}", path.display()));
        }

        None
    }

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
