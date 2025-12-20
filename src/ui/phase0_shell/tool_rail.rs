//! Tool rail UI for the Phase 1 chrome layout.

use gpui::{div, prelude::*};

use crate::ui::UiIcon;

use super::{
    Phase0Shell,
    chrome_palette::{chrome_border, chrome_panel},
    draw::{DrawEdgeMode, ToolMode},
    icon_button::{IconButtonState, icon_button},
};

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
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(tool_select_button(shell_state, cx))
        .child(tool_draw_line_button(shell_state, cx))
        .child(tool_draw_curve_button(shell_state, cx))
        .child(tool_placeholder_button(
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

fn tool_select_button(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
) -> impl gpui::IntoElement {
    let state = if shell_state.tool_mode == ToolMode::Manipulate {
        IconButtonState::Active
    } else {
        IconButtonState::Enabled
    };

    icon_button("tool-select", UiIcon::Select, state, Some("Select")).on_click(cx.listener(
        |shell: &mut Phase0Shell, _event, _window, view_cx| {
            shell.set_tool_mode(ToolMode::Manipulate);
            view_cx.notify();
        },
    ))
}

fn tool_draw_line_button(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
) -> impl gpui::IntoElement {
    let state =
        if shell_state.tool_mode == ToolMode::Draw && shell_state.edge_mode == DrawEdgeMode::Line {
            IconButtonState::Active
        } else {
            IconButtonState::Enabled
        };

    icon_button("tool-draw-line", UiIcon::DrawPath, state, Some("Draw Path")).on_click(cx.listener(
        |shell: &mut Phase0Shell, _event, _window, view_cx| {
            shell.set_tool_mode(ToolMode::Draw);
            shell.edge_mode = DrawEdgeMode::Line;
            view_cx.notify();
        },
    ))
}

fn tool_draw_curve_button(
    shell_state: &Phase0Shell,
    cx: &mut Context<Phase0Shell>,
) -> impl gpui::IntoElement {
    let state = if shell_state.tool_mode == ToolMode::Draw
        && shell_state.edge_mode == DrawEdgeMode::BezierAuto
    {
        IconButtonState::Active
    } else {
        IconButtonState::Enabled
    };

    icon_button(
        "tool-draw-curve",
        UiIcon::DrawCurve,
        state,
        Some("Draw Curve"),
    )
    .on_click(
        cx.listener(|shell: &mut Phase0Shell, _event, _window, view_cx| {
            shell.set_tool_mode(ToolMode::Draw);
            shell.edge_mode = DrawEdgeMode::BezierAuto;
            view_cx.notify();
        }),
    )
}

fn tool_placeholder_button(
    id: &'static str,
    icon: UiIcon,
    tooltip: &'static str,
) -> impl gpui::IntoElement {
    icon_button(id, icon, IconButtonState::Placeholder, Some(tooltip))
}
