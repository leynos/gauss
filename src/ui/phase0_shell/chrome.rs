//! Zed-inspired chrome layout for the Phase 1 UI shell.

use gpui::{Pixels, div, prelude::*, px};

use crate::ui::UiIcon;

/// Top bar vertical padding in normal (non-maximized) state.
const TOP_BAR_PAD_Y: Pixels = px(8.0);

/// Top bar horizontal padding (right side) in normal state.
const TOP_BAR_PAD_X: Pixels = px(12.0);

/// Extra top bar vertical padding when maximized.
const TOP_BAR_PAD_Y_MAXIMIZED: Pixels = px(12.0);

/// Extra top bar horizontal padding when maximized.
///
/// Wayland compositors intercept pointer events near window edges for resize
/// gestures, even on maximized windows. A 24px inset clears this zone on
/// tested compositors (GNOME Mutter, KDE `KWin`).
const TOP_BAR_PAD_X_MAXIMIZED: Pixels = px(24.0);

use super::{
    Phase0Shell,
    chrome_palette::{
        chrome_background, chrome_border, chrome_muted_text, chrome_panel, chrome_text,
    },
    chrome_panels,
    draw::{DrawEdgeMode, ToolMode},
    file_dialogs,
    icon_button::{IconButtonState, icon_button},
    resize_border, tool_rail, window_controls,
};

impl Phase0Shell {
    pub(super) fn chrome_view(
        &mut self,
        is_maximized: bool,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        // Start with resize borders (if visible) so they're rendered BEHIND
        // other UI elements. This ensures buttons remain clickable even if
        // there's a brief delay before re-render removes the borders.
        let mut view = div().size_full().relative(); // For absolute positioning of resize borders

        // Resize borders are disabled when maximized - the window cannot be
        // resized in that state. Adding them first (before other content)
        // gives them lower z-order so they don't block clicks to buttons.
        if !is_maximized {
            view = view.children(resize_border::resize_borders());
        }

        view.bg(chrome_background())
            .flex()
            .flex_col()
            .child(self.top_bar(is_maximized, cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .child(tool_rail::tool_rail(self, cx))
                    .child(self.editor_panel(cx)),
            )
            .child(chrome_panels::status_bar(
                self.mode_status_line(),
                self.file_status_line(),
            ))
    }

    fn top_bar(&mut self, is_maximized: bool, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let (pt, pr) = if is_maximized {
            (TOP_BAR_PAD_Y_MAXIMIZED, TOP_BAR_PAD_X_MAXIMIZED)
        } else {
            (TOP_BAR_PAD_Y, TOP_BAR_PAD_X)
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .pl_3()
            .pr(pr)
            .pt(pt)
            .pb_2()
            .bg(chrome_panel())
            .border_b_1()
            .border_color(chrome_border())
            .text_color(chrome_text())
            .child(Self::top_bar_left(is_maximized, cx))
            .child(self.top_bar_right(cx))
    }

    /// Left side of the top bar (drag region for window move).
    ///
    /// This region captures mouse-down events to initiate window dragging
    /// and double-clicks to toggle maximize/restore. It is separate from
    /// the right side to avoid interfering with button clicks. The drag
    /// region is disabled when maximized since window movement is not
    /// possible in that state.
    fn top_bar_left(is_maximized: bool, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let mut el = div()
            .id("titlebar-drag-region")
            .debug_selector(|| "#titlebar-drag-region".to_owned())
            .flex()
            .flex_1() // Take remaining horizontal space
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(chrome_muted_text())
                    .child("Open recent project"),
            )
            // Double-click to toggle maximize/restore
            .on_click(cx.listener(
                |_shell: &mut Self, event: &gpui::ClickEvent, window, view_cx| {
                    if let gpui::ClickEvent::Mouse(mouse_event) = event
                        && mouse_event.down.click_count == 2
                    {
                        window_controls::toggle_maximize(window);
                        view_cx.notify();
                    }
                },
            ));

        // Only enable drag region when not maximized
        if !is_maximized {
            el = el.on_mouse_down(gpui::MouseButton::Left, |_event, window, _cx| {
                window.start_window_move();
            });
        }

        el
    }

    fn top_bar_right(&mut self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(Self::top_bar_file_actions(cx))
            .child(self.top_bar_edit_actions(cx))
            .child(icon_button(
                "settings-button",
                UiIcon::Settings,
                IconButtonState::Placeholder,
                Some("Settings"),
            ))
            .child(self.style_picker_row())
            .child(Self::window_controls(cx))
    }

    fn top_bar_file_actions(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(icon_button(
                "file-new-button",
                UiIcon::FileNew,
                IconButtonState::Placeholder,
                Some("New"),
            ))
            .child(
                icon_button(
                    "open-button",
                    UiIcon::FileOpen,
                    IconButtonState::Enabled,
                    Some("Open"),
                )
                .on_click(cx.listener(
                    |shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                        file_dialogs::request_open(shell.open_prompt_mode, click_window, click_cx);
                    },
                )),
            )
            .child(
                icon_button(
                    "save-button",
                    UiIcon::FileSave,
                    IconButtonState::Enabled,
                    Some("Save"),
                )
                .on_click(cx.listener(
                    |_shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                        file_dialogs::request_save(click_window, click_cx);
                    },
                )),
            )
    }

    fn top_bar_edit_actions(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let can_undo = self.state.can_undo_document();
        let undo_state = if can_undo {
            IconButtonState::Enabled
        } else {
            IconButtonState::Disabled
        };
        let can_redo = self.state.can_redo_document();
        let redo_state = if can_redo {
            IconButtonState::Enabled
        } else {
            IconButtonState::Disabled
        };

        let mut undo_button =
            icon_button("undo-button", UiIcon::EditUndo, undo_state, Some("Undo"));
        if can_undo {
            undo_button =
                undo_button.on_click(cx.listener(|shell: &mut Self, _event, _window, view_cx| {
                    shell.undo_document();
                    view_cx.notify();
                }));
        }

        let mut redo_button =
            icon_button("redo-button", UiIcon::EditRedo, redo_state, Some("Redo"));
        if can_redo {
            redo_button =
                redo_button.on_click(cx.listener(|shell: &mut Self, _event, _window, view_cx| {
                    shell.redo_document();
                    view_cx.notify();
                }));
        }

        div()
            .flex()
            .items_center()
            .gap_2()
            .child(undo_button)
            .child(redo_button)
    }

    fn window_controls(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                icon_button(
                    "window-minimize",
                    UiIcon::WindowMinimize,
                    IconButtonState::Enabled,
                    Some("Minimize (Alt+F9)"),
                )
                .on_click(cx.listener(|_: &mut Self, _event, window, view_cx| {
                    window_controls::minimize(window);
                    view_cx.notify();
                })),
            )
            .child(
                icon_button(
                    "window-maximize",
                    UiIcon::WindowMaximize,
                    IconButtonState::Enabled,
                    Some("Maximize (Alt+F10)"),
                )
                .on_click(cx.listener(|_: &mut Self, _event, window, view_cx| {
                    window_controls::toggle_maximize(window);
                    view_cx.notify();
                })),
            )
            .child(Self::quit_button(cx))
    }

    fn editor_panel(&mut self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .child(chrome_panels::document_header())
            .child(div().flex().flex_1().p_4().child(self.canvas_area(cx)))
    }

    fn quit_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        icon_button(
            "quit-button",
            UiIcon::WindowClose,
            IconButtonState::Enabled,
            Some("Close Window"),
        )
        .on_click(cx.listener(|shell: &mut Self, _event, _window, click_cx| {
            shell.did_request_quit = true;
            click_cx.quit();
        }))
    }

    pub(super) fn set_tool_mode(&mut self, mode: ToolMode) {
        self.state.tool_mode = mode;
        if mode == ToolMode::Manipulate {
            self.state.active_path = None;
        }
    }

    #[expect(
        clippy::missing_const_for_fn,
        reason = "Keep setter consistent with runtime tool-mode mutation."
    )]
    pub(super) fn set_edge_mode(&mut self, mode: DrawEdgeMode) {
        self.state.edge_mode = mode;
    }
}
