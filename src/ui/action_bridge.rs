//! Bridge between model Actions and GPUI Actions.
//!
//! This module provides GPUI Action structs that correspond to model-layer
//! [`Action`] variants, enabling keyboard shortcuts to dispatch model Actions
//! via GPUI's action dispatch system.
//!
//! # Design
//!
//! GPUI's action system uses struct types decorated with `#[gpui::Action]`.
//! The model layer's [`Action`] enum is GPUI-independent for testability. This
//! bridge module provides:
//!
//! - GPUI Action structs for each model [`Action`] variant that needs keyboard
//!   dispatch
//! - Registration functions to bind these actions to keyboard shortcuts
//!
//! The view layer wires `.on_action()` handlers that dispatch to the
//! corresponding model Action logic.
//!
//! # Examples
//!
//! ```rust,ignore
//! // In bind_keymap():
//! app.bind_keys([
//!     KeyBinding::new("cmd-z", GpuiUndo, Some(KeyContext::Global.as_ref())),
//! ]);
//!
//! // In view render:
//! div().on_action(cx.listener(|shell, _: &GpuiUndo, _, cx| {
//!     shell.undo_document();
//!     cx.notify();
//! }))
//! ```

use gpui::KeyBinding;

use crate::model::{Action, KeyContext, default_bindings};

// === GPUI Action structs ===

/// GPUI action for [`Action::DeleteSelection`].
///
/// Deletes the currently selected shapes from the document.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiDeleteSelection;

/// GPUI action for [`Action::SelectAll`].
///
/// Selects all shapes in the document.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiSelectAll;

/// GPUI action for [`Action::DeselectAll`].
///
/// Clears the current selection.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiDeselectAll;

/// GPUI action for [`Action::ActivatePenTool`].
///
/// Switches to the Pen (draw) tool.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiActivatePenTool;

/// GPUI action for [`Action::ActivateSelectTool`].
///
/// Switches to the Selection (manipulate) tool.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiActivateSelectTool;

/// GPUI action for [`Action::Undo`].
///
/// Undoes the last document change.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiUndo;

/// GPUI action for [`Action::Redo`].
///
/// Redoes the last undone change.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiRedo;

/// GPUI action for [`Action::SelectionUndo`].
///
/// Undoes the last selection change.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiSelectionUndo;

/// GPUI action for [`Action::SelectionRedo`].
///
/// Redoes the last undone selection change.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiSelectionRedo;

// === Registration ===

/// Register action keybindings from the model-layer binding registry.
///
/// This function reads the default bindings from [`default_bindings`] and
/// registers corresponding GPUI [`KeyBinding`]s for each action.
///
/// # Arguments
///
/// * `app` - The GPUI application to register bindings on.
///
/// # Note
///
/// This should be called during application initialisation, typically from
/// [`crate::ui::init`].
pub fn register_action_bindings(app: &mut gpui::App) {
    for binding in default_bindings() {
        let keystroke = binding.keystroke.to_gpui_string();

        // Register in each context the binding applies to
        for context in &binding.contexts {
            let context_str = Some(context.as_ref());

            match binding.action {
                Action::DeleteSelection => {
                    app.bind_keys([KeyBinding::new(
                        &keystroke,
                        GpuiDeleteSelection,
                        context_str,
                    )]);
                }
                Action::SelectAll => {
                    app.bind_keys([KeyBinding::new(&keystroke, GpuiSelectAll, context_str)]);
                }
                Action::DeselectAll => {
                    app.bind_keys([KeyBinding::new(&keystroke, GpuiDeselectAll, context_str)]);
                }
                Action::ActivatePenTool => {
                    app.bind_keys([KeyBinding::new(
                        &keystroke,
                        GpuiActivatePenTool,
                        context_str,
                    )]);
                }
                Action::ActivateSelectTool => {
                    app.bind_keys([KeyBinding::new(
                        &keystroke,
                        GpuiActivateSelectTool,
                        context_str,
                    )]);
                }
                Action::Undo => {
                    app.bind_keys([KeyBinding::new(&keystroke, GpuiUndo, context_str)]);
                }
                Action::Redo => {
                    app.bind_keys([KeyBinding::new(&keystroke, GpuiRedo, context_str)]);
                }
                Action::SelectionUndo => {
                    app.bind_keys([KeyBinding::new(&keystroke, GpuiSelectionUndo, context_str)]);
                }
                Action::SelectionRedo => {
                    app.bind_keys([KeyBinding::new(&keystroke, GpuiSelectionRedo, context_str)]);
                }
            }
        }
    }
}

/// Map a [`ToolMode`] to its corresponding [`KeyContext`].
///
/// This helper is used by views to determine which key context to apply based
/// on the current editor mode.
///
/// # Arguments
///
/// * `mode` - The tool mode to map.
///
/// # Returns
///
/// The [`KeyContext`] corresponding to the tool mode.
///
/// # Note
///
/// This function is currently unused because GPUI's `key_context()` replaces
/// rather than stacks contexts. It is retained for future use when mode-specific
/// shortcuts can be properly supported.
#[expect(
    dead_code,
    reason = "retained for future mode-specific context support"
)]
pub(crate) const fn context_for_tool_mode(mode: super::phase0_shell::draw::ToolMode) -> KeyContext {
    use super::phase0_shell::draw::ToolMode;
    match mode {
        ToolMode::Draw => KeyContext::DrawMode,
        ToolMode::Manipulate => KeyContext::ManipulateMode,
    }
}
