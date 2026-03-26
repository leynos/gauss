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

/// Returns the localized status label for the current tool mode.
///
/// For Draw mode, includes the edge mode in parentheses.
/// For Manipulate mode, shows only the tool mode.
pub fn localized_status_label(
    tool_mode: ToolMode,
    edge_mode: EdgeMode,
    localizer: &Localizer,
    locale: &Locale,
) -> String {
    let tool_label = localized_tool_mode_label(tool_mode, localizer, locale);

    match tool_mode {
        ToolMode::Draw => {
            let edge_label = localized_edge_mode_label(edge_mode, localizer, locale);
            let template = localizer
                .lookup(locale, &MessageId::tool_status_mode_with_edge())
                .unwrap_or_else(|_| "Mode: {tool} ({edge})".to_owned());
            template
                .replace("{tool}", &tool_label)
                .replace("{edge}", &edge_label)
        }
        ToolMode::Manipulate => {
            let template = localizer
                .lookup(locale, &MessageId::tool_status_mode())
                .unwrap_or_else(|_| "Mode: {tool}".to_owned());
            template.replace("{tool}", &tool_label)
        }
    }
}
