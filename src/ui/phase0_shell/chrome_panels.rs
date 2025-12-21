//! Header and status bar panels for the Phase 1 chrome layout.

use gpui::{div, prelude::*};

use crate::ui::UiIcon;

use super::{
    chrome_palette::{chrome_border, chrome_muted_text, chrome_panel, chrome_text},
    icon_button::{IconButtonState, icon_button},
};

pub(super) fn document_header() -> impl gpui::IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .py_2()
        .bg(chrome_panel())
        .border_b_1()
        .border_color(chrome_border())
        .text_color(chrome_text())
        .child(div().text_sm().child("untitled"))
        .child(alignment_buttons())
}

pub(super) fn status_bar(
    mode_status_line: String,
    file_status_line: Option<String>,
) -> impl gpui::IntoElement {
    let left = div()
        .flex()
        .items_center()
        .gap_2()
        .child(icon_button(
            "zoom-out",
            UiIcon::ZoomOut,
            IconButtonState::Placeholder,
            Some("Zoom Out"),
        ))
        .child(icon_button(
            "zoom-in",
            UiIcon::ZoomIn,
            IconButtonState::Placeholder,
            Some("Zoom In"),
        ))
        .child(icon_button(
            "zoom-area",
            UiIcon::ZoomArea,
            IconButtonState::Placeholder,
            Some("Zoom to Area"),
        ))
        .child(icon_button(
            "snap-to-grid",
            UiIcon::SnapToGrid,
            IconButtonState::Placeholder,
            Some("Snap to Grid"),
        ));

    let mut right = div()
        .flex()
        .items_center()
        .gap_3()
        .text_sm()
        .text_color(chrome_muted_text())
        .child(mode_status_line);

    if let Some(status) = file_status_line {
        right = right.child(status);
    }

    right = right.child("1:1").child("Plain Text");

    div()
        .id("status-bar")
        .debug_selector(|| "#status-bar".to_owned())
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .py_2()
        .bg(chrome_panel())
        .border_t_1()
        .border_color(chrome_border())
        .child(left)
        .child(right)
}

fn alignment_buttons() -> impl gpui::IntoElement {
    const ALIGNMENT_BUTTONS: [(&str, UiIcon, &str); 6] = [
        ("align-left", UiIcon::AlignHorizontalLeft, "Align Left"),
        (
            "align-center",
            UiIcon::AlignHorizontalCenter,
            "Align Centre",
        ),
        ("align-right", UiIcon::AlignHorizontalRight, "Align Right"),
        ("align-top", UiIcon::AlignVerticalTop, "Align Top"),
        ("align-middle", UiIcon::AlignVerticalCenter, "Align Middle"),
        ("align-bottom", UiIcon::AlignVerticalBottom, "Align Bottom"),
    ];

    let mut row = div().flex().items_center().gap_2();
    for (id, icon, tooltip) in ALIGNMENT_BUTTONS {
        row = row.child(icon_button(
            id,
            icon,
            IconButtonState::Placeholder,
            Some(tooltip),
        ));
    }
    row
}
