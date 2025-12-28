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

#[cfg(test)]
mod tests;

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

/// Collected keybindings grouped by action type.
///
/// This struct holds vectors of GPUI `KeyBinding`s for each action type,
/// enabling batch registration with `app.bind_keys()`.
#[derive(Default)]
struct CollectedBindings {
    delete_selection: Vec<KeyBinding>,
    select_all: Vec<KeyBinding>,
    deselect_all: Vec<KeyBinding>,
    activate_pen_tool: Vec<KeyBinding>,
    activate_select_tool: Vec<KeyBinding>,
    undo: Vec<KeyBinding>,
    redo: Vec<KeyBinding>,
    selection_undo: Vec<KeyBinding>,
    selection_redo: Vec<KeyBinding>,
}

impl CollectedBindings {
    /// Collect all default bindings from the model layer.
    ///
    /// # Context Registration
    ///
    /// GPUI's key context system requires bindings to be registered for each
    /// context they should be active in. When a binding specifies
    /// [`KeyContext::Global`], it is expanded to all known contexts so the
    /// binding works regardless of the current editor mode. Non-global
    /// contexts (e.g., `ManipulateMode`) are registered literally.
    ///
    /// This expansion is necessary because the view layer currently only
    /// sets `KeyContext::Global` as the GPUI context. Future GPUI context
    /// stacking support may enable more granular mode-specific registration.
    fn from_default_bindings() -> Self {
        let mut collected = Self::default();

        for binding in default_bindings() {
            let keystroke = binding.keystroke.to_gpui_string();
            collected.add_binding_for_contexts(binding.action, &keystroke, &binding.contexts);
        }

        collected
    }

    /// Add bindings for an action across all specified contexts.
    ///
    /// When a context is [`KeyContext::Global`], the binding is expanded to
    /// all known contexts so it works regardless of the current editor mode.
    fn add_binding_for_contexts(
        &mut self,
        action: Action,
        keystroke: &str,
        contexts: &[KeyContext],
    ) {
        // Expand contexts: Global becomes all contexts, others remain literal.
        let expanded = contexts
            .iter()
            .flat_map(|ctx| {
                if *ctx == KeyContext::Global {
                    // Global bindings are registered for all contexts.
                    KeyContext::all().to_vec()
                } else {
                    // Mode-specific bindings are registered literally.
                    // Note: Currently unreachable since the view only applies
                    // KeyContext::Global; retained for future GPUI context stacking.
                    vec![*ctx]
                }
            })
            .collect::<Vec<_>>();

        for context in expanded {
            self.add_binding(action, keystroke, Some(context.as_ref()));
        }
    }

    /// Add a single binding to the appropriate collection.
    fn add_binding(&mut self, action: Action, keystroke: &str, context: Option<&str>) {
        match action {
            Action::DeleteSelection => {
                self.delete_selection.push(KeyBinding::new(
                    keystroke,
                    GpuiDeleteSelection,
                    context,
                ));
            }
            Action::SelectAll => {
                self.select_all
                    .push(KeyBinding::new(keystroke, GpuiSelectAll, context));
            }
            Action::DeselectAll => {
                self.deselect_all
                    .push(KeyBinding::new(keystroke, GpuiDeselectAll, context));
            }
            Action::ActivatePenTool => {
                self.activate_pen_tool.push(KeyBinding::new(
                    keystroke,
                    GpuiActivatePenTool,
                    context,
                ));
            }
            Action::ActivateSelectTool => {
                self.activate_select_tool.push(KeyBinding::new(
                    keystroke,
                    GpuiActivateSelectTool,
                    context,
                ));
            }
            Action::Undo => {
                self.undo
                    .push(KeyBinding::new(keystroke, GpuiUndo, context));
            }
            Action::Redo => {
                self.redo
                    .push(KeyBinding::new(keystroke, GpuiRedo, context));
            }
            Action::SelectionUndo => {
                self.selection_undo
                    .push(KeyBinding::new(keystroke, GpuiSelectionUndo, context));
            }
            Action::SelectionRedo => {
                self.selection_redo
                    .push(KeyBinding::new(keystroke, GpuiSelectionRedo, context));
            }
        }
    }

    /// Register all collected bindings with the GPUI application.
    fn register_all(self, app: &mut gpui::App) {
        app.bind_keys(self.delete_selection);
        app.bind_keys(self.select_all);
        app.bind_keys(self.deselect_all);
        app.bind_keys(self.activate_pen_tool);
        app.bind_keys(self.activate_select_tool);
        app.bind_keys(self.undo);
        app.bind_keys(self.redo);
        app.bind_keys(self.selection_undo);
        app.bind_keys(self.selection_redo);
    }
}

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
    let bindings = CollectedBindings::from_default_bindings();
    bindings.register_all(app);
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
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "retained for future mode-specific context support"
    )
)]
pub(crate) const fn context_for_tool_mode(mode: super::phase0_shell::draw::ToolMode) -> KeyContext {
    use super::phase0_shell::draw::ToolMode;
    match mode {
        ToolMode::Draw => KeyContext::DrawMode,
        ToolMode::Manipulate => KeyContext::ManipulateMode,
    }
}
