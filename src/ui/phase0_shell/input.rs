//! Keyboard input mapping for the Phase 0 shell.
//!
//! We keep input mapping separate from drawing to make it straightforward to
//! locate and change keyboard shortcuts without wading through rendering code.

use gpui::{Context, KeyDownEvent, Keystroke};

use super::{Phase0Shell, draw::ToolMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction {
    Escape,
    Tab,
    Raise,
    Lower,
    DocumentUndo,
    SelectionUndo,
    DocumentRedo,
    SelectionRedo,
}

fn key_action_for(keystroke: &Keystroke) -> Option<KeyAction> {
    match keystroke.key.as_str() {
        "escape" => Some(KeyAction::Escape),
        "tab" if !keystroke.modifiers.modified() => Some(KeyAction::Tab),
        "]" if keystroke.modifiers.secondary() && !keystroke.modifiers.shift => {
            Some(KeyAction::Raise)
        }
        "[" if keystroke.modifiers.secondary() && !keystroke.modifiers.shift => {
            Some(KeyAction::Lower)
        }
        "z" if keystroke.modifiers.secondary() && keystroke.modifiers.shift => {
            Some(KeyAction::SelectionUndo)
        }
        "z" if keystroke.modifiers.secondary() => Some(KeyAction::DocumentUndo),
        "y" if keystroke.modifiers.secondary() && keystroke.modifiers.shift => {
            Some(KeyAction::SelectionRedo)
        }
        "y" if keystroke.modifiers.secondary() => Some(KeyAction::DocumentRedo),
        _ => None,
    }
}

impl Phase0Shell {
    pub(super) fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if event.is_held {
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

    fn apply_key_action(&mut self, action: KeyAction, cx: &mut Context<Self>) -> bool {
        match action {
            KeyAction::Escape => {
                self.handle_escape(cx);
                false
            }
            KeyAction::Tab => match self.tool_mode {
                ToolMode::Draw => {
                    self.edge_mode = self.edge_mode.toggle();
                    true
                }
                ToolMode::Manipulate => self.toggle_selected_segments_kind(),
            },
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
