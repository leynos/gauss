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

        if is_toggle_edge_mode(&event.keystroke) {
            self.edge_mode = self.edge_mode.toggle();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if is_undo(&event.keystroke) {
            self.undo_document();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if is_redo(&event.keystroke) {
            self.redo_document();
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

fn is_toggle_edge_mode(keystroke: &Keystroke) -> bool {
    keystroke.key == "tab" && !keystroke.modifiers.modified()
}

fn is_undo(keystroke: &Keystroke) -> bool {
    keystroke.key == "z" && keystroke.modifiers.secondary() && !keystroke.modifiers.shift
}

fn is_redo(keystroke: &Keystroke) -> bool {
    let is_ctrl_y = keystroke.key == "y" && keystroke.modifiers.secondary();
    let is_cmd_shift_z =
        keystroke.key == "z" && keystroke.modifiers.secondary() && keystroke.modifiers.shift;
    is_ctrl_y || is_cmd_shift_z
}
