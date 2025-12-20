//! Layout and rendering for the Phase 0 shell.

use gpui::{Window, div, prelude::*, white};

use super::{
    KEY_CONTEXT, OpenSvg, Phase0Shell, SaveSvg, ToggleEdgeMode, chrome_palette::chrome_border,
    draw, file_dialogs,
};

impl Phase0Shell {
    pub(super) fn mode_status_line(&self) -> String {
        match self.tool_mode {
            draw::ToolMode::Draw => format!(
                "Mode: {} ({})",
                self.tool_mode.label(),
                self.edge_mode.label()
            ),
            draw::ToolMode::Manipulate => format!("Mode: {}", self.tool_mode.label()),
        }
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
            &mut shell.viewport,
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

        div()
            .size_full()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .on_key_down(
                cx.listener(|shell: &mut Self, event: &gpui::KeyDownEvent, _, view_cx| {
                    shell.handle_key_down(event, view_cx);
                }),
            )
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
            .on_action(
                cx.listener(|shell: &mut Self, _: &ToggleEdgeMode, _, action_cx| {
                    shell.handle_tab_action(action_cx);
                }),
            )
            .child(self.chrome_view(window, cx))
    }
}
