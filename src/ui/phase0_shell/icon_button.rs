//! Icon button helpers for the Phase 1 chrome layout.

use gpui::{Stateful, div, prelude::*, px};
use gpui_component::tooltip::Tooltip;

use crate::ui::{UiIcon, icon_element};

use super::chrome_palette::{chrome_active, chrome_border, chrome_muted_text, chrome_text};

const ICON_SIZE: f32 = 16.0;
const BUTTON_SIZE: f32 = 28.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IconButtonState {
    Enabled,
    Active,
    Disabled,
    Placeholder,
}

pub(super) fn icon_button_base(id: &'static str, state: IconButtonState) -> Stateful<gpui::Div> {
    let mut base = div()
        .id(id)
        .debug_selector(move || format!("#{id}"))
        .size(px(BUTTON_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_color(chrome_border())
        .rounded_md()
        .text_color(chrome_text());

    match state {
        IconButtonState::Enabled => {
            base = base.cursor_pointer();
        }
        IconButtonState::Active => {
            base = base.bg(chrome_active()).cursor_pointer();
        }
        IconButtonState::Disabled => {
            base = base
                .text_color(chrome_muted_text())
                .opacity(0.6)
                .cursor_default();
        }
        IconButtonState::Placeholder => {
            base = base
                .text_color(chrome_muted_text())
                .opacity(0.45)
                .cursor_default();
        }
    }

    base
}

pub(super) fn icon_button(
    id: &'static str,
    icon: UiIcon,
    state: IconButtonState,
    tooltip: Option<&'static str>,
) -> Stateful<gpui::Div> {
    let mut button = icon_button_base(id, state).child(icon_element(icon, ICON_SIZE));
    if let Some(text) = tooltip {
        button = button.tooltip(move |window, cx| Tooltip::new(text).build(window, cx));
    }
    button
}
