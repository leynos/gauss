//! Keyboard input mapping for the Phase 0 shell.
//!
//! We keep input mapping separate from drawing to make it straightforward to
//! locate and change keyboard shortcuts without wading through rendering code.

use gpui::{
    Context, KeyDownEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, NavigationDirection,
};

use crate::model::{SelItem, Selection};

use super::{Phase0Shell, draw::ToolMode};

/// Key actions handled directly by the Phase 0 shell.
///
/// Note: Editing actions are handled by the GPUI action bridge system (see
/// `src/ui/action_bridge.rs`). Do not add them here, as this handler calls
/// `stop_propagation()` which would prevent the action bridge from receiving
/// the keystrokes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction {
    Escape,
}

fn key_action_for(keystroke: &Keystroke) -> Option<KeyAction> {
    if keystroke.key.as_str() == "escape" {
        return Some(KeyAction::Escape);
    }

    None
}

impl Phase0Shell {
    pub(super) fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if event.is_held {
            return;
        }

        if event.keystroke.key.as_str() == "tab" && !event.keystroke.modifiers.modified() {
            // Phase 0 uses `Tab` as an editor command. Without explicitly
            // stopping propagation, focused children (for example, a colour
            // picker control) can consume `Tab` for focus traversal before this
            // view sees it.
            //
            // The actual edge-mode toggle is dispatched via the keymap to the
            // `ToggleEdgeMode` action; we avoid also handling `Tab` here to
            // prevent double-toggling.
            cx.stop_propagation();
            return;
        }

        let Some(action) = key_action_for(&event.keystroke) else {
            return;
        };

        let did_change = self.apply_key_action(action, cx);
        if did_change {
            cx.notify();
        }
        cx.stop_propagation();
    }

    pub(super) fn handle_tab_action(&mut self, cx: &mut Context<Self>) {
        if self.state.tool_mode != ToolMode::Draw {
            return;
        }

        self.set_edge_mode(self.state.edge_mode.toggle());
        cx.notify();
    }

    pub(super) fn handle_navigation_mouse_down(&mut self, event: &MouseDownEvent) -> bool {
        let MouseDownEvent {
            button, modifiers, ..
        } = event;

        let MouseButton::Navigate(direction) = button else {
            return false;
        };

        self.handle_navigation_button(*direction, *modifiers)
    }

    fn handle_navigation_button(
        &mut self,
        direction: NavigationDirection,
        modifiers: Modifiers,
    ) -> bool {
        let use_selection_history = modifiers.shift;

        match (direction, use_selection_history) {
            (NavigationDirection::Back, true) => {
                self.undo_selection();
                true
            }
            (NavigationDirection::Back, false) => {
                self.undo_document();
                true
            }
            (NavigationDirection::Forward, true) => {
                self.redo_selection();
                true
            }
            (NavigationDirection::Forward, false) => {
                self.redo_document();
                true
            }
        }
    }

    fn apply_key_action(&mut self, action: KeyAction, cx: &mut Context<Self>) -> bool {
        match action {
            KeyAction::Escape => {
                self.handle_escape(cx);
                // `handle_escape` notifies directly; return false to avoid a
                // second `cx.notify()` from `handle_key_down`.
                false
            }
        }
    }

    fn handle_escape(&mut self, cx: &mut Context<Self>) {
        match self.state.tool_mode {
            ToolMode::Draw => {
                self.state.tool_mode = ToolMode::Manipulate;
                self.state.active_path = None;
            }
            ToolMode::Manipulate => {
                self.state.tool_mode = ToolMode::Draw;
            }
        }

        cx.notify();
    }

    /// Select all shapes in the document.
    pub(super) fn select_all(&mut self, cx: &mut Context<Self>) {
        let all_shapes: Vec<SelItem> = self
            .state
            .document
            .shapes
            .iter()
            .map(|shape| SelItem::Shape(shape.id))
            .collect();

        self.apply_selection_change(Selection { items: all_shapes }, cx);
    }

    /// Clear the current selection.
    pub(super) fn deselect_all(&mut self, cx: &mut Context<Self>) {
        self.apply_selection_change(Selection::empty(), cx);
    }

    /// Apply a selection change, recording it in history if different from current.
    fn apply_selection_change(&mut self, new_selection: Selection, cx: &mut Context<Self>) {
        if new_selection != self.state.selection {
            let previous = self.state.selection.clone();
            self.record_selection_change(previous, new_selection.clone());
            self.state.selection = new_selection;
            cx.notify();
        }
    }
}
