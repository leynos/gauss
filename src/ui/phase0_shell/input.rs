//! Keyboard input mapping for the Phase 0 shell.
//!
//! We keep input mapping separate from drawing to make it straightforward to
//! locate and change keyboard shortcuts without wading through rendering code.

use gpui::{Context, KeyDownEvent, Keystroke};

use super::{Phase0Shell, draw::ToolMode};

impl Phase0Shell {
    pub(super) fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if event.is_held {
            return;
        }

        if is_escape(&event.keystroke) {
            self.handle_escape(cx);
            cx.stop_propagation();
            return;
        }

        if is_tab(&event.keystroke) {
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

            cx.stop_propagation();
            return;
        }

        if is_document_undo(&event.keystroke) {
            self.undo_document();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if is_selection_undo(&event.keystroke) {
            self.undo_selection();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if is_document_redo(&event.keystroke) {
            self.redo_document();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if is_selection_redo(&event.keystroke) {
            self.redo_selection();
            cx.notify();
            cx.stop_propagation();
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

fn is_escape(keystroke: &Keystroke) -> bool {
    keystroke.key == "escape"
}

fn is_tab(keystroke: &Keystroke) -> bool {
    keystroke.key == "tab" && !keystroke.modifiers.modified()
}

fn is_document_undo(keystroke: &Keystroke) -> bool {
    keystroke.key == "z" && keystroke.modifiers.secondary() && !keystroke.modifiers.shift
}

fn is_selection_undo(keystroke: &Keystroke) -> bool {
    keystroke.key == "z" && keystroke.modifiers.secondary() && keystroke.modifiers.shift
}

fn is_document_redo(keystroke: &Keystroke) -> bool {
    keystroke.key == "y" && keystroke.modifiers.secondary() && !keystroke.modifiers.shift
}

fn is_selection_redo(keystroke: &Keystroke) -> bool {
    keystroke.key == "y" && keystroke.modifiers.secondary() && keystroke.modifiers.shift
}
