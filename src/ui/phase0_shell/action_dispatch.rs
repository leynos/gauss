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

use super::{A11yActionRequestError, A11yRequestedAction};
use super::{
    A11yWindowAction, CloseWindow, MinimizeWindow, Phase0Shell, ShowWindowMenu, StartWindowMove,
    StartWindowResize, ToggleFullscreen, ToggleMaximize, window_controls,
};

macro_rules! bind_gpui_model_actions {
    ($el:expr, $cx:expr, [ $( ($gpui_action:ty, $gauss_action:expr) ),* $(,)? ]) => {{
        let el = $el;
        $(
        let el = el.on_action(
            $cx.listener(|shell: &mut Self, _: &$gpui_action, _, action_cx| {
                shell.execute_model_action($gauss_action, action_cx);
            }),
        );
        )*
        el
    }};
}

impl Phase0Shell {
    pub(super) fn execute_model_action(
        &mut self,
        action: GaussAction,
        cx: &mut gpui::Context<Self>,
    ) {
        #[expect(
            clippy::match_same_arms,
            reason = "Wildcard arm needed for non_exhaustive enum; specific arms document future work"
        )]
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
            // Style and Transform actions - not yet implemented, treat as no-ops
            GaussAction::SetStrokeColor(_)
            | GaussAction::SetStrokeWidth(_)
            | GaussAction::SetStrokeOpacity(_)
            | GaussAction::SetFillColor(_)
            | GaussAction::SetFillOpacity(_)
            | GaussAction::ToggleNoFill
            | GaussAction::SetObjectPosition(_)
            | GaussAction::SetObjectSize(_)
            | GaussAction::SetObjectRotation(_) => {
                // TODO: Implement style/transform actions when command system supports them
                // For now, these are audit-only no-ops until execution exists
            }
            // Wildcard arm for future non_exhaustive variants
            _ => {
                // New action variants are handled here until explicitly implemented
                debug_assert!(false, "unhandled action variant: {action:?}");
                log::warn!("unhandled action variant: {:?}", action);
                // Error is propagated via shell state for observable handling
                self.report_error(format!("unhandled action variant: {action:?}"));
            }
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
            #[expect(
                clippy::unreachable,
                reason = "wildcard arm must panic in all builds when a non-tool action is dispatched"
            )]
            _ => {
                unreachable!("execute_tool_action called with non-tool action: {action:?}")
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
            #[expect(
                clippy::unreachable,
                reason = "wildcard arm must panic in all builds when a non-history action is dispatched"
            )]
            _ => {
                unreachable!("execute_history_action called with non-history action: {action:?}")
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

    // FIXME: remove once the production GPUI event hook is wired — see roadmap §1.9.5
    // Prepared for production AccessKit integration; currently exercised only
    // via test_helpers::handle_accesskit_action_request_for_tests.
    #[cfg_attr(
        not(any(test, feature = "test-support", coverage, coverage_nightly)),
        expect(
            dead_code,
            reason = "compiled unconditionally for production readiness; no GPUI event hook yet"
        )
    )]
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
        bind_gpui_model_actions!(
            el,
            cx,
            [
                (GpuiSelectAll, GaussAction::SelectAll),
                (GpuiDeselectAll, GaussAction::DeselectAll),
                (GpuiDeleteSelection, GaussAction::DeleteSelection),
                (
                    GpuiInsertAnchorOnSegment,
                    GaussAction::InsertAnchorOnSegment
                ),
                (
                    GpuiDeleteSelectedAnchors,
                    GaussAction::DeleteSelectedAnchors
                ),
                (GpuiRaiseSelection, GaussAction::RaiseSelection),
                (GpuiLowerSelection, GaussAction::LowerSelection),
                (GpuiToggleSegmentKind, GaussAction::ToggleSegmentKind),
                (GpuiActivatePenTool, GaussAction::ActivatePenTool),
                (GpuiActivateSelectTool, GaussAction::ActivateSelectTool),
                (GpuiUndo, GaussAction::Undo),
                (GpuiRedo, GaussAction::Redo),
                (GpuiSelectionUndo, GaussAction::SelectionUndo),
                (GpuiSelectionRedo, GaussAction::SelectionRedo),
            ]
        )
    }
}
