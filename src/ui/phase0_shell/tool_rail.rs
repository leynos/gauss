//! Tool rail UI for the Phase 1 chrome layout.

use gpui::{div, prelude::*};

use crate::i18n::MessageId;
use crate::ui::UiIcon;

use super::{
    Phase0Shell,
    chrome_palette::{chrome_border, chrome_panel},
    draw::{DrawEdgeMode, ToolMode},
    icon_button::{IconButtonState, icon_button},
};

/// Build the vertical tool rail for the chrome shell.
///
/// Uses the current shell state to set button states and wires click handlers
/// with the provided context.
pub(super) fn tool_rail(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
) -> impl gpui::IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .px_2()
        .py_3()
        .bg(chrome_panel())
        .border_r_1()
        .border_color(chrome_border())
        .child(tool_rail_buttons(shell_state, cx))
}

fn tool_rail_buttons(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
) -> impl gpui::IntoElement {
    let mut rail = div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(tool_select_button(shell_state, cx));

    for spec in DRAW_BUTTON_SPECS {
        rail = rail.child(tool_draw_button(shell_state, cx, spec));
    }

    rail.child(tool_placeholder_button(
        shell_state,
        "tool-draw-square",
        UiIcon::DrawSquare,
        "tool.tooltip.draw_rectangle",
    ))
    .child(tool_placeholder_button(
        shell_state,
        "tool-draw-circle",
        UiIcon::DrawCircle,
        "tool.tooltip.draw_circle",
    ))
}

struct ToolModeButtonSpec<F, S> {
    id: &'static str,
    icon: UiIcon,
    tooltip: String,
    state_fn: S,
    on_click: F,
}

struct ToolDrawButtonSpec {
    id: &'static str,
    icon: UiIcon,
    message_key: &'static str,
    edge_mode: DrawEdgeMode,
}

const DRAW_BUTTON_SPECS: &[ToolDrawButtonSpec] = &[
    ToolDrawButtonSpec {
        id: "tool-draw-line",
        icon: UiIcon::DrawPath,
        message_key: "tool.tooltip.draw_path",
        edge_mode: DrawEdgeMode::Line,
    },
    ToolDrawButtonSpec {
        id: "tool-draw-curve",
        icon: UiIcon::DrawCurve,
        message_key: "tool.tooltip.draw_curve",
        edge_mode: DrawEdgeMode::BezierAuto,
    },
];

/// Shared builder for tool-mode buttons to keep wiring consistent.
fn tool_mode_button<F, S>(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
    spec: ToolModeButtonSpec<F, S>,
) -> impl gpui::IntoElement
where
    F: Fn(&mut Phase0Shell) + 'static,
    S: Fn(&Phase0Shell) -> IconButtonState,
{
    let ToolModeButtonSpec {
        id,
        icon,
        tooltip,
        state_fn,
        on_click,
    } = spec;
    let state = state_fn(shell_state);
    icon_button(id, icon, state, Some(tooltip)).on_click(cx.listener(
        move |shell: &mut Phase0Shell, _event, _window, view_cx| {
            on_click(shell);
            view_cx.notify();
        },
    ))
}

fn tool_draw_button(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
    spec: &ToolDrawButtonSpec,
) -> impl gpui::IntoElement {
    let tooltip = shell_state.localize(&MessageId::from(spec.message_key));
    let edge_mode = spec.edge_mode;
    tool_mode_button(
        shell_state,
        cx,
        ToolModeButtonSpec {
            id: spec.id,
            icon: spec.icon,
            tooltip,
            state_fn: move |shell: &Phase0Shell| {
                draw_button_state(shell.state.tool_mode, shell.state.edge_mode, edge_mode)
            },
            on_click: move |shell: &mut Phase0Shell| {
                let _ = shell.activate_draw_tool(Some(edge_mode));
            },
        },
    )
}

fn tool_select_button(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
) -> impl gpui::IntoElement {
    let tooltip = shell_state.localize(&MessageId::tool_tooltip_select());
    tool_mode_button(
        shell_state,
        cx,
        ToolModeButtonSpec {
            id: "tool-select",
            icon: UiIcon::Select,
            tooltip,
            state_fn: |shell: &Phase0Shell| select_button_state(shell.state.tool_mode),
            on_click: |shell: &mut Phase0Shell| {
                let _ = shell.activate_select_tool();
            },
        },
    )
}

fn tool_placeholder_button(
    shell_state: &Phase0Shell,
    id: &'static str,
    icon: UiIcon,
    message_key: &'static str,
) -> impl gpui::IntoElement {
    let tooltip = shell_state.localize(&MessageId::from(message_key));
    icon_button(id, icon, IconButtonState::Placeholder, Some(tooltip))
}

fn select_button_state(tool_mode: ToolMode) -> IconButtonState {
    if tool_mode == ToolMode::Manipulate {
        IconButtonState::Active
    } else {
        IconButtonState::Enabled
    }
}

fn draw_button_state(
    tool_mode: ToolMode,
    current_edge_mode: DrawEdgeMode,
    target_edge_mode: DrawEdgeMode,
) -> IconButtonState {
    if tool_mode == ToolMode::Draw && current_edge_mode == target_edge_mode {
        IconButtonState::Active
    } else {
        IconButtonState::Enabled
    }
}

#[cfg(test)]
mod tests {
    //! Tests for tool rail button state helpers.

    use super::{DrawEdgeMode, IconButtonState, ToolMode};
    use super::{draw_button_state, select_button_state};

    #[test]
    fn select_button_state_tracks_tool_mode() {
        assert_eq!(
            select_button_state(ToolMode::Manipulate),
            IconButtonState::Active
        );
        assert_eq!(
            select_button_state(ToolMode::Draw),
            IconButtonState::Enabled
        );
    }

    #[test]
    fn draw_line_button_state_tracks_edge_mode() {
        assert_eq!(
            draw_button_state(ToolMode::Draw, DrawEdgeMode::Line, DrawEdgeMode::Line),
            IconButtonState::Active
        );
        assert_eq!(
            draw_button_state(ToolMode::Draw, DrawEdgeMode::BezierAuto, DrawEdgeMode::Line),
            IconButtonState::Enabled
        );
        assert_eq!(
            draw_button_state(ToolMode::Manipulate, DrawEdgeMode::Line, DrawEdgeMode::Line),
            IconButtonState::Enabled
        );
    }

    #[test]
    fn draw_curve_button_state_tracks_edge_mode() {
        assert_eq!(
            draw_button_state(
                ToolMode::Draw,
                DrawEdgeMode::BezierAuto,
                DrawEdgeMode::BezierAuto
            ),
            IconButtonState::Active
        );
        assert_eq!(
            draw_button_state(ToolMode::Draw, DrawEdgeMode::Line, DrawEdgeMode::BezierAuto),
            IconButtonState::Enabled
        );
        assert_eq!(
            draw_button_state(
                ToolMode::Manipulate,
                DrawEdgeMode::BezierAuto,
                DrawEdgeMode::BezierAuto
            ),
            IconButtonState::Enabled
        );
    }
}
