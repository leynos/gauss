//! Header and status bar panels for the Phase 1 chrome layout.

use gpui::{div, prelude::*};

use crate::i18n::MessageId;
use crate::ui::UiIcon;

use super::{
    Phase0Shell,
    chrome_palette::{chrome_border, chrome_muted_text, chrome_panel, chrome_text},
    icon_button::{IconButtonState, icon_button},
};

impl Phase0Shell {
    pub(super) fn document_header(&self) -> impl gpui::IntoElement {
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
            .child(
                div()
                    .text_sm()
                    .child(self.localize(&MessageId::doc_untitled())),
            )
            .child(self.alignment_buttons())
    }

    pub(super) fn status_bar(
        &self,
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
                Some(self.localize(&MessageId::status_zoom_out())),
            ))
            .child(icon_button(
                "zoom-in",
                UiIcon::ZoomIn,
                IconButtonState::Placeholder,
                Some(self.localize(&MessageId::status_zoom_in())),
            ))
            .child(icon_button(
                "zoom-area",
                UiIcon::ZoomArea,
                IconButtonState::Placeholder,
                Some(self.localize(&MessageId::status_zoom_area())),
            ))
            .child(icon_button(
                "snap-to-grid",
                UiIcon::SnapToGrid,
                IconButtonState::Placeholder,
                Some(self.localize(&MessageId::status_snap_grid())),
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

        right = right
            .child(self.localize(&MessageId::status_zoom_ratio_1_1()))
            .child(self.localize(&MessageId::status_plain_text()));

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

    fn alignment_buttons(&self) -> impl gpui::IntoElement {
        let buttons = [
            (
                "align-left",
                UiIcon::AlignHorizontalLeft,
                MessageId::align_left(),
            ),
            (
                "align-center",
                UiIcon::AlignHorizontalCenter,
                MessageId::align_centre(),
            ),
            (
                "align-right",
                UiIcon::AlignHorizontalRight,
                MessageId::align_right(),
            ),
            (
                "align-top",
                UiIcon::AlignVerticalTop,
                MessageId::align_top(),
            ),
            (
                "align-middle",
                UiIcon::AlignVerticalCenter,
                MessageId::align_middle(),
            ),
            (
                "align-bottom",
                UiIcon::AlignVerticalBottom,
                MessageId::align_bottom(),
            ),
        ];

        let mut row = div().flex().items_center().gap_2();
        for (id, icon, msg_id) in buttons {
            let tooltip = self.localize(&msg_id);
            row = row.child(icon_button(
                id,
                icon,
                IconButtonState::Placeholder,
                Some(tooltip),
            ));
        }
        row
    }
}
