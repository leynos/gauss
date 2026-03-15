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

mod keystroke;

use gpui::KeyBinding;

use crate::model::{Action, KeyContext, default_bindings};

pub use keystroke::keystroke_to_gpui_string;

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

/// GPUI action for [`Action::InsertAnchorOnSegment`].
///
/// Inserts an anchor on the selected segment.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiInsertAnchorOnSegment;

/// GPUI action for [`Action::DeleteSelectedAnchors`].
///
/// Deletes the selected anchors.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiDeleteSelectedAnchors;

/// GPUI action for [`Action::RaiseSelection`].
///
/// Raises selected shapes in the z-order.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiRaiseSelection;

/// GPUI action for [`Action::LowerSelection`].
///
/// Lowers selected shapes in the z-order.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiLowerSelection;

/// GPUI action for [`Action::ToggleSegmentKind`].
///
/// Toggles segment kind between Line and Cubic.
#[derive(Clone, Debug, Default, PartialEq, gpui::Action)]
#[action(no_json)]
pub struct GpuiToggleSegmentKind;

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
    insert_anchor_on_segment: Vec<KeyBinding>,
    delete_selected_anchors: Vec<KeyBinding>,
    raise_selection: Vec<KeyBinding>,
    lower_selection: Vec<KeyBinding>,
    toggle_segment_kind: Vec<KeyBinding>,
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
            let keystroke = keystroke_to_gpui_string(&binding.keystroke);
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
        for ctx in contexts {
            if *ctx == KeyContext::Global {
                // Global bindings are registered for all contexts.
                self.add_binding_for_all_contexts(action, keystroke);
            } else {
                // Mode-specific bindings are registered literally.
                // Note: Currently unreachable since the view only applies
                // KeyContext::Global; retained for future GPUI context stacking.
                self.add_binding(action, keystroke, Some(ctx.as_ref()));
            }
        }
    }

    /// Add a binding for an action in all known contexts.
    fn add_binding_for_all_contexts(&mut self, action: Action, keystroke: &str) {
        for context in KeyContext::all() {
            self.add_binding(action, keystroke, Some(context.as_ref()));
        }
    }

    /// Add a single binding to the appropriate collection.
    fn add_binding(&mut self, action: Action, keystroke: &str, ctx: Option<&str>) {
        match action {
            Action::DeleteSelection => {
                self.delete_selection
                    .push(KeyBinding::new(keystroke, GpuiDeleteSelection, ctx));
            }
            Action::SelectAll => {
                self.select_all
                    .push(KeyBinding::new(keystroke, GpuiSelectAll, ctx));
            }
            Action::DeselectAll => {
                self.deselect_all
                    .push(KeyBinding::new(keystroke, GpuiDeselectAll, ctx));
            }
            Action::ActivatePenTool => {
                self.activate_pen_tool
                    .push(KeyBinding::new(keystroke, GpuiActivatePenTool, ctx));
            }
            Action::ActivateSelectTool => {
                self.activate_select_tool.push(KeyBinding::new(
                    keystroke,
                    GpuiActivateSelectTool,
                    ctx,
                ));
            }
            Action::Undo => self.undo.push(KeyBinding::new(keystroke, GpuiUndo, ctx)),
            Action::Redo => self.redo.push(KeyBinding::new(keystroke, GpuiRedo, ctx)),
            Action::SelectionUndo => {
                self.selection_undo
                    .push(KeyBinding::new(keystroke, GpuiSelectionUndo, ctx));
            }
            Action::SelectionRedo => {
                self.selection_redo
                    .push(KeyBinding::new(keystroke, GpuiSelectionRedo, ctx));
            }
            Action::InsertAnchorOnSegment => {
                self.insert_anchor_on_segment.push(KeyBinding::new(
                    keystroke,
                    GpuiInsertAnchorOnSegment,
                    ctx,
                ));
            }
            Action::DeleteSelectedAnchors => {
                self.delete_selected_anchors.push(KeyBinding::new(
                    keystroke,
                    GpuiDeleteSelectedAnchors,
                    ctx,
                ));
            }
            Action::RaiseSelection => {
                self.raise_selection
                    .push(KeyBinding::new(keystroke, GpuiRaiseSelection, ctx));
            }
            Action::LowerSelection => {
                self.lower_selection
                    .push(KeyBinding::new(keystroke, GpuiLowerSelection, ctx));
            }
            Action::ToggleSegmentKind => {
                self.toggle_segment_kind.push(KeyBinding::new(
                    keystroke,
                    GpuiToggleSegmentKind,
                    ctx,
                ));
            }
            _ => {
                debug_assert!(false, "unsupported future model action binding: {action:?}");
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
        app.bind_keys(self.insert_anchor_on_segment);
        app.bind_keys(self.delete_selected_anchors);
        app.bind_keys(self.raise_selection);
        app.bind_keys(self.lower_selection);
        app.bind_keys(self.toggle_segment_kind);
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
/// This should be called during application initialization, typically from
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
/// This helper is used by Phase 0 views to apply mode-specific key contexts.
pub(crate) const fn context_for_tool_mode(mode: super::phase0_shell::draw::ToolMode) -> KeyContext {
    use super::phase0_shell::draw::ToolMode;
    match mode {
        ToolMode::Draw => KeyContext::DrawMode,
        ToolMode::Manipulate => KeyContext::ManipulateMode,
    }
}
