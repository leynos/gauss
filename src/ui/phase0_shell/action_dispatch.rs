//! Shared action dispatch and binding helpers for the Phase 0 shell.
//!
//! This module keeps command routing separate from rendering so the view module
//! stays focused on layout and paint concerns.

use accesskit::ActionRequest;
use gpui::{InteractiveElement, Window};

use crate::model::Action as GaussAction;
use crate::ui::action_bridge::{
    GpuiActivatePenTool, GpuiActivateSelectTool, GpuiDeleteSelectedAnchors, GpuiDeleteSelection,
    GpuiDeselectAll, GpuiInsertAnchorOnSegment, GpuiLowerSelection, GpuiRaiseSelection, GpuiRedo,
    GpuiSelectAll, GpuiSelectionRedo, GpuiSelectionUndo, GpuiToggleSegmentKind, GpuiUndo,
};

use super::{
    A11yActionRequestError, A11yRequestedAction, A11yWindowAction, CloseWindow, MinimizeWindow,
    Phase0Shell, ShowWindowMenu, StartWindowMove, StartWindowResize, ToggleFullscreen,
    ToggleMaximize, window_controls,
};

#[derive(Clone, Copy)]
enum SelectionAction {
    SelectAll,
    DeselectAll,
    DeleteSelection,
    InsertAnchorOnSegment,
    DeleteSelectedAnchors,
    RaiseSelection,
    LowerSelection,
    ToggleSegmentKind,
}

impl SelectionAction {
    const fn from_gauss(action: GaussAction) -> Option<Self> {
        match action {
            GaussAction::SelectAll => Some(Self::SelectAll),
            GaussAction::DeselectAll => Some(Self::DeselectAll),
            GaussAction::DeleteSelection => Some(Self::DeleteSelection),
            GaussAction::InsertAnchorOnSegment => Some(Self::InsertAnchorOnSegment),
            GaussAction::DeleteSelectedAnchors => Some(Self::DeleteSelectedAnchors),
            GaussAction::RaiseSelection => Some(Self::RaiseSelection),
            GaussAction::LowerSelection => Some(Self::LowerSelection),
            GaussAction::ToggleSegmentKind => Some(Self::ToggleSegmentKind),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum ToolAction {
    ActivatePenTool,
    ActivateSelectTool,
}

impl ToolAction {
    const fn from_gauss(action: GaussAction) -> Option<Self> {
        match action {
            GaussAction::ActivatePenTool => Some(Self::ActivatePenTool),
            GaussAction::ActivateSelectTool => Some(Self::ActivateSelectTool),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum HistoryAction {
    Undo,
    Redo,
    SelectionUndo,
    SelectionRedo,
}

impl HistoryAction {
    const fn from_gauss(action: GaussAction) -> Option<Self> {
        match action {
            GaussAction::Undo => Some(Self::Undo),
            GaussAction::Redo => Some(Self::Redo),
            GaussAction::SelectionUndo => Some(Self::SelectionUndo),
            GaussAction::SelectionRedo => Some(Self::SelectionRedo),
            _ => None,
        }
    }
}

impl Phase0Shell {
    pub(super) fn execute_model_action(
        &mut self,
        action: GaussAction,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(selection_action) = SelectionAction::from_gauss(action) {
            self.execute_selection_action(selection_action, cx);
        } else if let Some(tool_action) = ToolAction::from_gauss(action) {
            self.execute_tool_action(tool_action, cx);
        } else if let Some(history_action) = HistoryAction::from_gauss(action) {
            self.execute_history_action(history_action, cx);
        }
    }

    fn execute_selection_action(&mut self, action: SelectionAction, cx: &mut gpui::Context<Self>) {
        match action {
            SelectionAction::SelectAll => self.select_all(cx),
            SelectionAction::DeselectAll => self.deselect_all(cx),
            SelectionAction::DeleteSelection => self.execute_delete_selection(cx),
            SelectionAction::InsertAnchorOnSegment => {
                self.apply_change(Self::insert_anchor_on_selected_segment, cx);
            }
            SelectionAction::DeleteSelectedAnchors => {
                self.apply_change(Self::delete_selected_anchors, cx);
            }
            SelectionAction::RaiseSelection => {
                self.apply_change(Self::raise_selected_shapes, cx);
            }
            SelectionAction::LowerSelection => {
                self.apply_change(Self::lower_selected_shapes, cx);
            }
            SelectionAction::ToggleSegmentKind => {
                self.apply_change(Self::toggle_selected_segments_kind, cx);
            }
        }
    }

    /// Calls `f(self)` and invokes [`gpui::Context::notify`] when `f` returns
    /// `true`.
    fn apply_change(&mut self, f: fn(&mut Self) -> bool, cx: &mut gpui::Context<Self>) {
        if f(self) {
            cx.notify();
        }
    }

    fn execute_delete_selection(&mut self, cx: &mut gpui::Context<Self>) {
        let has_shape_selection = self.state.selection.selected_shapes().next().is_some();
        let did_change = if has_shape_selection {
            self.delete_selected_shapes()
        } else {
            self.delete_selected_anchors()
        };
        if did_change {
            cx.notify();
        }
    }

    fn execute_tool_action(&mut self, action: ToolAction, cx: &mut gpui::Context<Self>) {
        let error_before = self.last_history_error.clone();
        let did_change = match action {
            ToolAction::ActivatePenTool => self.activate_draw_tool(None),
            ToolAction::ActivateSelectTool => self.activate_select_tool(),
        };
        if did_change || self.last_history_error != error_before {
            cx.notify();
        }
    }

    fn execute_history_action(&mut self, action: HistoryAction, cx: &mut gpui::Context<Self>) {
        match action {
            HistoryAction::Undo => self.undo_document(),
            HistoryAction::Redo => self.redo_document(),
            HistoryAction::SelectionUndo => self.undo_selection(),
            HistoryAction::SelectionRedo => self.redo_selection(),
        }
        cx.notify();
    }

    pub(super) fn execute_window_action(
        &mut self,
        action: A11yWindowAction,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) {
        match action {
            A11yWindowAction::ShowWindowMenu => window_controls::show_window_menu(window),
            A11yWindowAction::Minimize => {
                window_controls::minimize(window);
                cx.notify();
            }
            A11yWindowAction::ToggleMaximize => {
                window_controls::toggle_maximize(window);
                cx.notify();
            }
            A11yWindowAction::ToggleFullscreen => {
                window_controls::toggle_fullscreen(window);
                cx.notify();
            }
            A11yWindowAction::CloseWindow => {
                self.did_request_quit = true;
                cx.quit();
            }
        }
    }

    pub(super) fn handle_accesskit_action_request(
        &mut self,
        request: &ActionRequest,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> Result<A11yRequestedAction, A11yActionRequestError> {
        let routed = super::A11yService::route_action_request(request)?;
        match routed {
            A11yRequestedAction::Model(action) => self.execute_model_action(action, cx),
            A11yRequestedAction::Window(action) => self.execute_window_action(action, window, cx),
        }
        Ok(routed)
    }

    /// Bind window control action handlers to the root element.
    pub(super) fn bind_window_actions(el: gpui::Div, cx: &mut gpui::Context<Self>) -> gpui::Div {
        el.on_action(
            cx.listener(|shell: &mut Self, _: &MinimizeWindow, w, view_cx| {
                shell.execute_window_action(A11yWindowAction::Minimize, w, view_cx);
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &ToggleMaximize, w, view_cx| {
                shell.execute_window_action(A11yWindowAction::ToggleMaximize, w, view_cx);
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &ToggleFullscreen, w, view_cx| {
                shell.execute_window_action(A11yWindowAction::ToggleFullscreen, w, view_cx);
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &CloseWindow, w, action_cx| {
                shell.execute_window_action(A11yWindowAction::CloseWindow, w, action_cx);
            }),
        )
        .on_action(cx.listener(|_: &mut Self, _: &StartWindowMove, w, _cx| {
            window_controls::start_move(w);
        }))
        .on_action(cx.listener(|_: &mut Self, _: &StartWindowResize, w, _cx| {
            window_controls::start_resize(w, gpui::ResizeEdge::BottomRight);
        }))
        .on_action(
            cx.listener(|shell: &mut Self, _: &ShowWindowMenu, w, action_cx| {
                shell.execute_window_action(A11yWindowAction::ShowWindowMenu, w, action_cx);
            }),
        )
    }

    pub(super) fn bind_model_actions(el: gpui::Div, cx: &mut gpui::Context<Self>) -> gpui::Div {
        let with_selection_actions = Self::bind_selection_actions(el, cx);
        let with_edit_actions = Self::bind_edit_actions(with_selection_actions, cx);
        let with_tool_actions = Self::bind_tool_actions(with_edit_actions, cx);
        Self::bind_history_actions(with_tool_actions, cx)
    }

    fn bind_selection_actions(el: gpui::Div, cx: &mut gpui::Context<Self>) -> gpui::Div {
        el.on_action(
            cx.listener(|shell: &mut Self, _: &GpuiSelectAll, _, action_cx| {
                shell.execute_model_action(GaussAction::SelectAll, action_cx);
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiDeselectAll, _, action_cx| {
                shell.execute_model_action(GaussAction::DeselectAll, action_cx);
            }),
        )
    }

    fn bind_edit_actions(el: gpui::Div, cx: &mut gpui::Context<Self>) -> gpui::Div {
        el.on_action(
            cx.listener(|shell: &mut Self, _: &GpuiDeleteSelection, _, action_cx| {
                shell.execute_model_action(GaussAction::DeleteSelection, action_cx);
            }),
        )
        .on_action(cx.listener(
            |shell: &mut Self, _: &GpuiInsertAnchorOnSegment, _, action_cx| {
                shell.execute_model_action(GaussAction::InsertAnchorOnSegment, action_cx);
            },
        ))
        .on_action(cx.listener(
            |shell: &mut Self, _: &GpuiDeleteSelectedAnchors, _, action_cx| {
                shell.execute_model_action(GaussAction::DeleteSelectedAnchors, action_cx);
            },
        ))
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiRaiseSelection, _, action_cx| {
                shell.execute_model_action(GaussAction::RaiseSelection, action_cx);
            }),
        )
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiLowerSelection, _, action_cx| {
                shell.execute_model_action(GaussAction::LowerSelection, action_cx);
            }),
        )
        .on_action(cx.listener(
            |shell: &mut Self, _: &GpuiToggleSegmentKind, _, action_cx| {
                shell.execute_model_action(GaussAction::ToggleSegmentKind, action_cx);
            },
        ))
    }

    fn bind_tool_actions(el: gpui::Div, cx: &mut gpui::Context<Self>) -> gpui::Div {
        el.on_action(
            cx.listener(|shell: &mut Self, _: &GpuiActivatePenTool, _, action_cx| {
                shell.execute_model_action(GaussAction::ActivatePenTool, action_cx);
            }),
        )
        .on_action(cx.listener(
            |shell: &mut Self, _: &GpuiActivateSelectTool, _, action_cx| {
                shell.execute_model_action(GaussAction::ActivateSelectTool, action_cx);
            },
        ))
    }

    fn bind_history_actions(el: gpui::Div, cx: &mut gpui::Context<Self>) -> gpui::Div {
        el.on_action(cx.listener(|shell: &mut Self, _: &GpuiUndo, _, action_cx| {
            shell.execute_model_action(GaussAction::Undo, action_cx);
        }))
        .on_action(cx.listener(|shell: &mut Self, _: &GpuiRedo, _, action_cx| {
            shell.execute_model_action(GaussAction::Redo, action_cx);
        }))
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiSelectionUndo, _, action_cx| {
                shell.execute_model_action(GaussAction::SelectionUndo, action_cx);
            }),
        )
        .on_action(cx.listener(
            |shell: &mut Self, _: &GpuiSelectionRedo, _, action_cx| {
                shell.execute_model_action(GaussAction::SelectionRedo, action_cx);
            },
        ))
    }
}
