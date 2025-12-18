//! Keyboard input mapping for the Phase 0 shell.
//!
//! We keep input mapping separate from drawing to make it straightforward to
//! locate and change keyboard shortcuts without wading through rendering code.

use gpui::{
    Context, KeyDownEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, NavigationDirection,
};

use super::{Phase0Shell, draw::ToolMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction {
    Escape,
    InsertAnchor,
    DeleteAnchors,
    Raise,
    Lower,
    DocumentUndo,
    SelectionUndo,
    DocumentRedo,
    SelectionRedo,
}

fn key_action_for(keystroke: &Keystroke) -> Option<KeyAction> {
    if keystroke.key.as_str() == "escape" {
        return Some(KeyAction::Escape);
    }

    if !keystroke.modifiers.modified() {
        return key_action_for_unmodified_key(keystroke.key.as_str());
    }

    key_action_for_modified_key(keystroke)
}

fn key_action_for_unmodified_key(key: &str) -> Option<KeyAction> {
    match key {
        "i" => Some(KeyAction::InsertAnchor),
        "backspace" | "delete" => Some(KeyAction::DeleteAnchors),
        _ => None,
    }
}

fn key_action_for_modified_key(keystroke: &Keystroke) -> Option<KeyAction> {
    let key = keystroke.key.as_str();
    let is_secondary = keystroke.modifiers.secondary();
    let is_shift = keystroke.modifiers.shift;

    match (key, is_secondary, is_shift) {
        ("]", true, false) => Some(KeyAction::Raise),
        ("[", true, false) => Some(KeyAction::Lower),
        ("z", true, true) => Some(KeyAction::SelectionUndo),
        ("z", true, false) => Some(KeyAction::DocumentUndo),
        ("y", true, true) => Some(KeyAction::SelectionRedo),
        ("y", true, false) => Some(KeyAction::DocumentRedo),
        _ => None,
    }
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
        let did_change = match self.tool_mode {
            ToolMode::Draw => {
                self.edge_mode = self.edge_mode.toggle();
                true
            }
            ToolMode::Manipulate => self.toggle_selected_segments_kind(),
        };

        if did_change {
            cx.notify();
        }
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
                false
            }
            KeyAction::InsertAnchor => self.insert_anchor_on_selected_segment(),
            KeyAction::DeleteAnchors => self.delete_selected_anchors(),
            KeyAction::Raise => self.raise_selected_shapes(),
            KeyAction::Lower => self.lower_selected_shapes(),
            KeyAction::DocumentUndo => {
                self.undo_document();
                true
            }
            KeyAction::SelectionUndo => {
                self.undo_selection();
                true
            }
            KeyAction::DocumentRedo => {
                self.redo_document();
                true
            }
            KeyAction::SelectionRedo => {
                self.redo_selection();
                true
            }
        }
    }

    fn handle_escape(&mut self, cx: &mut Context<Self>) {
        match self.tool_mode {
            ToolMode::Draw => {
                self.tool_mode = ToolMode::Manipulate;
                self.draw_active_shape = None;
            }
            ToolMode::Manipulate => {
                self.tool_mode = ToolMode::Draw;
            }
        }

        cx.notify();
    }
}
