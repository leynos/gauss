//! Shared i18n helper functions for localizing UI elements.

use crate::i18n::{Locale, Localizer, MessageId};
use crate::model::{EdgeMode, ToolMode};

/// Returns the localized label for a tool mode, falling back to the default label.
pub fn localized_tool_mode_label(
    tool_mode: ToolMode,
    localizer: &Localizer,
    locale: &Locale,
) -> String {
    let message_id = match tool_mode {
        ToolMode::Draw => MessageId::tool_mode_draw(),
        ToolMode::Manipulate => MessageId::tool_mode_manipulate(),
    };
    localizer
        .lookup(locale, &message_id)
        .unwrap_or_else(|_| tool_mode.label().to_owned())
}

/// Returns the localized label for an edge mode, falling back to the default label.
pub fn localized_edge_mode_label(
    edge_mode: EdgeMode,
    localizer: &Localizer,
    locale: &Locale,
) -> String {
    let message_id = match edge_mode {
        EdgeMode::Line => MessageId::edge_mode_line(),
        EdgeMode::BezierAuto => MessageId::edge_mode_bezier_auto(),
    };
    localizer
        .lookup(locale, &message_id)
        .unwrap_or_else(|_| edge_mode.label().to_owned())
}
