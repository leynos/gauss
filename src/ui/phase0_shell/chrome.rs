//! Zed-inspired chrome layout for the Phase 1 UI shell.

use gpui::{div, prelude::*};

use crate::ui::UiIcon;

use super::{
    Phase0Shell,
    chrome_palette::{
        chrome_background, chrome_border, chrome_muted_text, chrome_panel, chrome_text,
    },
    chrome_panels,
    draw::ToolMode,
    file_dialogs,
    icon_button::{IconButtonState, icon_button},
    tool_rail,
};

impl Phase0Shell {
    pub(super) fn chrome_view(&mut self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .size_full()
            .bg(chrome_background())
            .flex()
            .flex_col()
            .child(self.top_bar(cx))
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

    fn top_bar(&mut self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .bg(chrome_panel())
            .border_b_1()
            .border_color(chrome_border())
            .text_color(chrome_text())
            .child(Self::top_bar_left())
            .child(self.top_bar_right(cx))
    }

    fn top_bar_left() -> impl gpui::IntoElement {
        div().flex().items_center().gap_2().child(
            div()
                .text_sm()
                .text_color(chrome_muted_text())
                .child("Open recent project"),
        )
    }

    fn top_bar_right(&mut self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(Self::top_bar_file_actions(cx))
            .child(Self::top_bar_edit_actions(cx))
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

    fn top_bar_edit_actions(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                icon_button(
                    "undo-button",
                    UiIcon::EditUndo,
                    IconButtonState::Enabled,
                    Some("Undo"),
                )
                .on_click(cx.listener(
                    |shell: &mut Self, _event, _window, view_cx| {
                        shell.undo_document();
                        view_cx.notify();
                    },
                )),
            )
            .child(
                icon_button(
                    "redo-button",
                    UiIcon::EditRedo,
                    IconButtonState::Enabled,
                    Some("Redo"),
                )
                .on_click(cx.listener(
                    |shell: &mut Self, _event, _window, view_cx| {
                        shell.redo_document();
                        view_cx.notify();
                    },
                )),
            )
    }

    fn window_controls(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(icon_button(
                "window-minimize",
                UiIcon::WindowMinimize,
                IconButtonState::Placeholder,
                Some("Minimise"),
            ))
            .child(icon_button(
                "window-maximize",
                UiIcon::WindowMaximize,
                IconButtonState::Placeholder,
                Some("Maximise"),
            ))
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
        self.tool_mode = mode;
        if mode == ToolMode::Manipulate {
            self.draw_active_shape = None;
        }
    }
}
