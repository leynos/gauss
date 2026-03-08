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

impl Phase0Shell {
    pub(super) fn execute_model_action(
        &mut self,
        action: GaussAction,
        cx: &mut gpui::Context<Self>,
    ) {
        match action {
            // Selection actions
            GaussAction::SelectAll => self.select_all(cx),
            GaussAction::DeselectAll => self.deselect_all(cx),
            GaussAction::DeleteSelection => self.execute_delete_selection(cx),
            GaussAction::InsertAnchorOnSegment => {
                if self.insert_anchor_on_selected_segment() {
                    cx.notify();
                }
            }
            GaussAction::DeleteSelectedAnchors => {
                if self.delete_selected_anchors() {
                    cx.notify();
                }
            }
            GaussAction::RaiseSelection => {
                if self.raise_selected_shapes() {
                    cx.notify();
                }
            }
            GaussAction::LowerSelection => {
                if self.lower_selected_shapes() {
                    cx.notify();
                }
            }
            GaussAction::ToggleSegmentKind => {
                if self.toggle_selected_segments_kind() {
                    cx.notify();
                }
            }
            // Tool actions
            GaussAction::ActivatePenTool | GaussAction::ActivateSelectTool => {
                self.execute_tool_action(action, cx);
            }
            // History actions
            GaussAction::Undo
            | GaussAction::Redo
            | GaussAction::SelectionUndo
            | GaussAction::SelectionRedo => self.execute_history_action(action, cx),
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

    fn execute_tool_action(&mut self, action: GaussAction, cx: &mut gpui::Context<Self>) {
        let error_before = self.last_history_error.clone();
        let did_change = match action {
            GaussAction::ActivatePenTool => self.activate_draw_tool(None),
            GaussAction::ActivateSelectTool => self.activate_select_tool(),
            _ => {
                debug_assert!(
                    false,
                    "execute_tool_action called with non-tool action: {action:?}"
                );
                false
            }
        };
        if did_change || self.last_history_error != error_before {
            cx.notify();
        }
    }

    fn execute_history_action(&mut self, action: GaussAction, cx: &mut gpui::Context<Self>) {
        match action {
            GaussAction::Undo => self.undo_document(),
            GaussAction::Redo => self.redo_document(),
            GaussAction::SelectionUndo => self.undo_selection(),
            GaussAction::SelectionRedo => self.redo_selection(),
            _ => {
                debug_assert!(
                    false,
                    "execute_history_action called with non-history action: {action:?}"
                );
            }
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
        let routed = self.a11y_service.route_action_request(request)?;
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
        .on_action(
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
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiActivatePenTool, _, action_cx| {
                shell.execute_model_action(GaussAction::ActivatePenTool, action_cx);
            }),
        )
        .on_action(cx.listener(
            |shell: &mut Self, _: &GpuiActivateSelectTool, _, action_cx| {
                shell.execute_model_action(GaussAction::ActivateSelectTool, action_cx);
            },
        ))
        .on_action(cx.listener(|shell: &mut Self, _: &GpuiUndo, _, action_cx| {
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
        .on_action(
            cx.listener(|shell: &mut Self, _: &GpuiSelectionRedo, _, action_cx| {
                shell.execute_model_action(GaussAction::SelectionRedo, action_cx);
            }),
        )
    }
}
