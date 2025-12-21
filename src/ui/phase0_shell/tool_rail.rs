//! Tool rail UI for the Phase 1 chrome layout.

use gpui::{div, prelude::*};

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
        rail = rail.child(tool_draw_button(shell_state, cx, &spec));
    }

    rail.child(tool_placeholder_button(
        "tool-draw-square",
        UiIcon::DrawSquare,
        "Draw Rectangle",
    ))
    .child(tool_placeholder_button(
        "tool-draw-circle",
        UiIcon::DrawCircle,
        "Draw Circle",
    ))
}

struct ToolModeButtonSpec<F, S> {
    id: &'static str,
    icon: UiIcon,
    tooltip: &'static str,
    state_fn: S,
    on_click: F,
}

#[derive(Clone, Copy)]
struct ToolDrawButtonSpec {
    id: &'static str,
    icon: UiIcon,
    tooltip: &'static str,
    edge_mode: DrawEdgeMode,
}

const DRAW_BUTTON_SPECS: [ToolDrawButtonSpec; 2] = [
    ToolDrawButtonSpec {
        id: "tool-draw-line",
        icon: UiIcon::DrawPath,
        tooltip: "Draw Path",
        edge_mode: DrawEdgeMode::Line,
    },
    ToolDrawButtonSpec {
        id: "tool-draw-curve",
        icon: UiIcon::DrawCurve,
        tooltip: "Draw Curve",
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
    let id = spec.id;
    let icon = spec.icon;
    let tooltip = spec.tooltip;
    let edge_mode = spec.edge_mode;
    tool_mode_button(
        shell_state,
        cx,
        ToolModeButtonSpec {
            id,
            icon,
            tooltip,
            state_fn: move |shell: &Phase0Shell| match edge_mode {
                DrawEdgeMode::Line => draw_line_button_state(shell.tool_mode, shell.edge_mode),
                DrawEdgeMode::BezierAuto => {
                    draw_curve_button_state(shell.tool_mode, shell.edge_mode)
                }
            },
            on_click: move |shell: &mut Phase0Shell| {
                shell.set_tool_mode(ToolMode::Draw);
                shell.set_edge_mode(edge_mode);
            },
        },
    )
}

fn tool_select_button(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
) -> impl gpui::IntoElement {
    tool_mode_button(
        shell_state,
        cx,
        ToolModeButtonSpec {
            id: "tool-select",
            icon: UiIcon::Select,
            tooltip: "Select",
            state_fn: |shell: &Phase0Shell| select_button_state(shell.tool_mode),
            on_click: |shell: &mut Phase0Shell| shell.set_tool_mode(ToolMode::Manipulate),
        },
    )
}

fn tool_placeholder_button(
    id: &'static str,
    icon: UiIcon,
    tooltip: &'static str,
) -> impl gpui::IntoElement {
    icon_button(id, icon, IconButtonState::Placeholder, Some(tooltip))
}

fn select_button_state(tool_mode: ToolMode) -> IconButtonState {
    if tool_mode == ToolMode::Manipulate {
        IconButtonState::Active
    } else {
        IconButtonState::Enabled
    }
}

fn draw_line_button_state(tool_mode: ToolMode, edge_mode: DrawEdgeMode) -> IconButtonState {
    if tool_mode == ToolMode::Draw && edge_mode == DrawEdgeMode::Line {
        IconButtonState::Active
    } else {
        IconButtonState::Enabled
    }
}

fn draw_curve_button_state(tool_mode: ToolMode, edge_mode: DrawEdgeMode) -> IconButtonState {
    if tool_mode == ToolMode::Draw && edge_mode == DrawEdgeMode::BezierAuto {
        IconButtonState::Active
    } else {
        IconButtonState::Enabled
    }
}

#[cfg(test)]
mod tests {
    use super::{DrawEdgeMode, IconButtonState, ToolMode};
    use super::{draw_curve_button_state, draw_line_button_state, select_button_state};

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
            draw_line_button_state(ToolMode::Draw, DrawEdgeMode::Line),
            IconButtonState::Active
        );
        assert_eq!(
            draw_line_button_state(ToolMode::Draw, DrawEdgeMode::BezierAuto),
            IconButtonState::Enabled
        );
        assert_eq!(
            draw_line_button_state(ToolMode::Manipulate, DrawEdgeMode::Line),
            IconButtonState::Enabled
        );
    }

    #[test]
    fn draw_curve_button_state_tracks_edge_mode() {
        assert_eq!(
            draw_curve_button_state(ToolMode::Draw, DrawEdgeMode::BezierAuto),
            IconButtonState::Active
        );
        assert_eq!(
            draw_curve_button_state(ToolMode::Draw, DrawEdgeMode::Line),
            IconButtonState::Enabled
        );
        assert_eq!(
            draw_curve_button_state(ToolMode::Manipulate, DrawEdgeMode::BezierAuto),
            IconButtonState::Enabled
        );
    }
}
