//! Icon button helpers for the Phase 1 chrome layout.

use gpui::{SharedString, Stateful, div, prelude::*, px};
use gpui_component::tooltip::Tooltip;

use crate::ui::{UiIcon, icon_element};

use super::chrome_palette::{chrome_active, chrome_border, chrome_muted_text, chrome_text};

const ICON_SIZE: f32 = 16.0;
const BUTTON_SIZE: f32 = 28.0;

/// Visual and interactive states for an icon button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IconButtonState {
    /// Button can be clicked and uses the normal chrome styling.
    Enabled,
    /// Button can be clicked and represents the active mode or state.
    Active,
    /// Button is visible but cannot be clicked.
    Disabled,
    /// Button reserves layout space for a control that is not active yet.
    Placeholder,
}

/// Construct a sized, styled icon-button base without an icon child.
///
/// `id` is used for GPUI element identity and debug selectors. `state`
/// controls cursor, opacity, text colour, and active background styling.
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

/// Construct a complete icon button with an icon and optional tooltip.
///
/// Wraps [`icon_button_base`], adds the supplied `icon`, and attaches a tooltip
/// builder when `tooltip` is present. The returned div is ready for callers to
/// attach click handlers or other GPUI behaviours.
pub(super) fn icon_button<T>(
    id: &'static str,
    icon: UiIcon,
    state: IconButtonState,
    tooltip: Option<T>,
) -> Stateful<gpui::Div>
where
    T: Into<SharedString> + 'static,
{
    let mut button = icon_button_base(id, state).child(icon_element(icon, ICON_SIZE));
    if let Some(text) = tooltip {
        let tooltip_text = text.into();
        button =
            button.tooltip(move |window, cx| Tooltip::new(tooltip_text.clone()).build(window, cx));
    }
    button
}
