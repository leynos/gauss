//! Layout and rendering for the Phase 0 shell.

use gpui::{Window, div, prelude::*, white};

use crate::model::KeyContext;
use crate::ui::action_bridge::{
    GpuiActivatePenTool, GpuiActivateSelectTool, GpuiDeleteSelection, GpuiDeselectAll, GpuiRedo,
    GpuiSelectAll, GpuiSelectionRedo, GpuiSelectionUndo, GpuiUndo, context_for_tool_mode,
};

use super::{
    CloseWindow, MinimizeWindow, OpenSvg, Phase0Shell, SaveSvg, ShowWindowMenu, StartWindowMove,
    StartWindowResize, ToggleEdgeMode, ToggleFullscreen, ToggleMaximize,
    chrome_palette::chrome_border, draw, file_dialogs, window_controls,
};

impl Phase0Shell {
    pub(super) fn mode_status_line(&self) -> String {
        let maximized_indicator = if self.last_maximized_state == Some(true) {
            " [MAX]"
        } else {
            ""
        };
        match self.tool_mode {
            draw::ToolMode::Draw => format!(
                "Mode: {} ({}){}",
                self.tool_mode.label(),
                self.edge_mode.label(),
                maximized_indicator
            ),
            draw::ToolMode::Manipulate => {
                format!("Mode: {}{}", self.tool_mode.label(), maximized_indicator)
            }
        }
    }

    pub(super) fn file_status_line(&self) -> Option<String> {
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

impl Phase0Shell {
    /// Bind window control action handlers to the root element.
    fn bind_window_actions(el: gpui::Div, cx: &mut Context<Self>) -> gpui::Div {
        el.on_action(cx.listener(|_: &mut Self, _: &MinimizeWindow, w, view_cx| {
            window_controls::minimize(w);
            view_cx.notify();
        }))
        .on_action(cx.listener(|_: &mut Self, _: &ToggleMaximize, w, view_cx| {
            window_controls::toggle_maximize(w);
            view_cx.notify();
        }))
        .on_action(
            cx.listener(|_: &mut Self, _: &ToggleFullscreen, w, view_cx| {
                window_controls::toggle_fullscreen(w);
                view_cx.notify();
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &CloseWindow, _, action_cx| {
                shell.did_request_quit = true;
                action_cx.quit();
            }),
        )
        .on_action(cx.listener(|_: &mut Self, _: &StartWindowMove, w, _cx| {
            window_controls::start_move(w);
        }))
        .on_action(cx.listener(|_: &mut Self, _: &StartWindowResize, w, _cx| {
            window_controls::start_resize(w, gpui::ResizeEdge::BottomRight);
        }))
        .on_action(cx.listener(|_: &mut Self, _: &ShowWindowMenu, w, _cx| {
            window_controls::show_window_menu(w);
        }))
    }
}

impl Phase0Shell {
    /// Bind model action handlers (from the action bridge) to the root element.
    fn bind_model_actions(el: gpui::Div, cx: &mut Context<Self>) -> gpui::Div {
        el.on_action(
            cx.listener(|shell: &mut Self, _: &GpuiSelectAll, _, action_cx| {
                shell.select_all(action_cx);
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiDeselectAll, _, action_cx| {
                shell.deselect_all(action_cx);
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiDeleteSelection, _, action_cx| {
                shell.delete_selected_anchors();
                action_cx.notify();
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiActivatePenTool, _, action_cx| {
                shell.tool_mode = draw::ToolMode::Draw;
                action_cx.notify();
            }),
        )
        .on_action(cx.listener(
            |shell: &mut Self, _: &GpuiActivateSelectTool, _, action_cx| {
                shell.tool_mode = draw::ToolMode::Manipulate;
                action_cx.notify();
            },
        ))
        .on_action(cx.listener(|shell: &mut Self, _: &GpuiUndo, _, action_cx| {
            shell.undo_document();
            action_cx.notify();
        }))
        .on_action(cx.listener(|shell: &mut Self, _: &GpuiRedo, _, action_cx| {
            shell.redo_document();
            action_cx.notify();
        }))
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiSelectionUndo, _, action_cx| {
                shell.undo_selection();
                action_cx.notify();
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiSelectionRedo, _, action_cx| {
                shell.redo_selection();
                action_cx.notify();
            }),
        )
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

        // Determine which key context to apply. For now, we use only the global
        // context. Mode-specific contexts (for ManipulateMode-only shortcuts like
        // Delete) would require a different approach since GPUI's key_context()
        // replaces rather than adds to the context.
        let _mode_context = context_for_tool_mode(self.tool_mode);

        let root = div()
            .size_full()
            // Apply global context for global shortcuts (Undo, Redo, Tab, etc.)
            // Note: We only use the global context for now. Mode-specific context
            // (for ManipulateMode-only shortcuts) will need a different approach
            // since GPUI's key_context() replaces rather than adds.
            .key_context(KeyContext::Global.as_ref())
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
mod tests {
    use std::path::PathBuf;

    use gpui::TestAppContext;

    use super::Phase0Shell;

    #[gpui::test]
    fn file_status_line_prefers_save_error(cx: &mut TestAppContext) {
        cx.update(crate::ui::init);

        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                shell.last_save_error = Some("disk full".to_owned());
                shell.last_open_error = Some("missing file".to_owned());
                shell.last_saved_path = Some(PathBuf::from("/tmp/out.svg"));
                shell.last_opened_path = Some(PathBuf::from("/tmp/in.svg"));
            });
        });
        visual_cx.run_until_parked();

        let status = visual_cx.read(|app| view.read(app).file_status_line());
        assert_eq!(status, Some("Save failed: disk full".to_owned()));
    }

    #[gpui::test]
    fn file_status_line_falls_back_to_open_error(cx: &mut TestAppContext) {
        cx.update(crate::ui::init);

        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                shell.last_open_error = Some("missing file".to_owned());
                shell.last_saved_path = Some(PathBuf::from("/tmp/out.svg"));
            });
        });
        visual_cx.run_until_parked();

        let status = visual_cx.read(|app| view.read(app).file_status_line());
        assert_eq!(status, Some("Open failed: missing file".to_owned()));
    }

    #[gpui::test]
    fn file_status_line_reports_paths_when_no_errors(cx: &mut TestAppContext) {
        cx.update(crate::ui::init);

        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                shell.last_saved_path = Some(PathBuf::from("/tmp/out.svg"));
            });
        });
        visual_cx.run_until_parked();

        let status = visual_cx.read(|app| view.read(app).file_status_line());
        assert_eq!(status, Some("Saved: /tmp/out.svg".to_owned()));

        visual_cx.update(|_window, app| {
            view.update(app, |shell, _cx| {
                shell.last_saved_path = None;
                shell.last_opened_path = Some(PathBuf::from("/tmp/in.svg"));
            });
        });
        visual_cx.run_until_parked();

        let status_after = visual_cx.read(|app| view.read(app).file_status_line());
        assert_eq!(status_after, Some("Opened: /tmp/in.svg".to_owned()));
    }

    #[gpui::test]
    fn file_status_line_returns_none_when_empty(cx: &mut TestAppContext) {
        cx.update(crate::ui::init);

        let (view, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        visual_cx.run_until_parked();

        let status = visual_cx.read(|app| view.read(app).file_status_line());
        assert_eq!(status, None);
    }
}
