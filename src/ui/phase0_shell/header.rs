//! Header row UI for the Phase 0 shell.
//!
//! We keep header construction in a separate module to avoid bloating
//! `phase0_shell/mod.rs`, which has a strict per-file line limit in this repo.

use gpui::{Window, div, prelude::*};

use super::{Phase0Shell, file_dialogs};

impl Phase0Shell {
    pub(super) fn header_row(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        self.ensure_style_pickers(window, cx);

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
                    .child(self.style_picker_row())
                    .child(Self::open_button(cx))
                    .child(Self::save_button(cx))
                    .child(Self::quit_button(cx)),
            )
    }

    fn open_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        Self::header_button_base("open-button")
            .on_click(cx.listener(
                |shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                    file_dialogs::request_open(shell.open_prompt_mode, click_window, click_cx);
                },
            ))
            .child("Open…")
    }

    fn save_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        Self::header_button_base("save-button")
            .on_click(cx.listener(
                |_shell: &mut Self, _event: &gpui::ClickEvent, click_window, click_cx| {
                    file_dialogs::request_save(click_window, click_cx);
                },
            ))
            .child("Save…")
    }

    fn quit_button(cx: &mut Context<Self>) -> impl gpui::IntoElement {
        Self::header_button_base("quit-button")
            .on_click(cx.listener(
                |shell: &mut Self, _event: &gpui::ClickEvent, _window, click_cx| {
                    shell.did_request_quit = true;
                    click_cx.quit();
                },
            ))
            .child("Quit")
    }

    fn header_button_base(id: &'static str) -> gpui::Stateful<gpui::Div> {
        div()
            .id(id)
            .debug_selector(move || format!("#{id}"))
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
    }
}
