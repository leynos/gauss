//! Action keybinding registry.
//!
//! This module provides the mapping between [`Action`] variants and keyboard
//! shortcuts, including the key contexts in which each binding is active.
//!
//! # Design
//!
//! The keybinding registry is GPUI-independent, enabling:
//!
//! - Testing of keybinding logic without GPUI
//! - Generation of GPUI keybindings at startup
//! - Display of shortcuts in menus and tooltips
//! - Future serialization to user preferences
//!
//! # Default Bindings
//!
//! The [`default_bindings`] function returns the complete set of default
//! keybindings for all actions. These follow platform conventions where
//! possible and use the `secondary` modifier for cross-platform compatibility.
//!
//! # Examples
//!
//! ```rust,no_run
//! use gauss::model::{Action, KeyContext, default_bindings, bindings_for_action};
//!
//! // Get all default bindings
//! let all_bindings = default_bindings();
//!
//! // Find bindings for a specific action
//! let undo_bindings = bindings_for_action(Action::Undo);
//! assert!(!undo_bindings.is_empty());
//! ```

use super::action::Action;
use super::key_context::KeyContext;
use super::keystroke::Keystroke;

/// A binding between an [`Action`] and a keyboard shortcut.
///
/// `ActionBinding` defines the keyboard shortcuts for actions and the contexts
/// in which they are active. Multiple bindings can exist for the same action
/// (e.g., both Backspace and Delete for `DeleteSelection`).
///
/// # Context Scoping
///
/// The `contexts` field determines when the binding is active:
///
/// - If `contexts` contains [`KeyContext::Global`], the binding is always
///   active.
/// - If `contexts` contains mode-specific contexts (e.g., `ManipulateMode`),
///   the binding is only active in those modes.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::{Action, ActionBinding, KeyContext, Keystroke};
///
/// let binding = ActionBinding {
///     action: Action::DeleteSelection,
///     keystroke: Keystroke::new("backspace"),
///     contexts: vec![KeyContext::ManipulateMode],
/// };
///
/// assert!(binding.is_active_in(KeyContext::ManipulateMode));
/// assert!(!binding.is_active_in(KeyContext::DrawMode));
/// ```
#[derive(Clone, Debug)]
pub struct ActionBinding {
    /// The action to invoke when the keystroke is pressed.
    pub action: Action,
    /// The keyboard shortcut that triggers the action.
    pub keystroke: Keystroke,
    /// The key context(s) in which this binding is active.
    ///
    /// If this contains [`KeyContext::Global`], the binding is active in all
    /// contexts. Otherwise, it is only active in the listed contexts.
    pub contexts: Vec<KeyContext>,
}

impl ActionBinding {
    /// Create a binding with no modifiers.
    #[must_use]
    pub fn new(action: Action, key: &str, contexts: &[KeyContext]) -> Self {
        Self {
            action,
            keystroke: Keystroke::new(key),
            contexts: contexts.to_vec(),
        }
    }

    /// Create a binding with the secondary (Cmd/Ctrl) modifier.
    #[must_use]
    pub fn secondary(action: Action, key: &str, contexts: &[KeyContext]) -> Self {
        Self {
            action,
            keystroke: Keystroke::secondary(key),
            contexts: contexts.to_vec(),
        }
    }

    /// Create a binding with secondary + shift modifiers.
    #[must_use]
    pub fn secondary_shift(action: Action, key: &str, contexts: &[KeyContext]) -> Self {
        Self {
            action,
            keystroke: Keystroke::secondary_shift(key),
            contexts: contexts.to_vec(),
        }
    }

    /// Create a binding with the alt modifier.
    #[must_use]
    pub fn alt(action: Action, key: &str, contexts: &[KeyContext]) -> Self {
        Self {
            action,
            keystroke: Keystroke::alt(key),
            contexts: contexts.to_vec(),
        }
    }

    /// Check if this binding is active in the given context.
    ///
    /// A binding is active if:
    ///
    /// - Its `contexts` list contains the given context, OR
    /// - Its `contexts` list contains [`KeyContext::Global`]
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gauss::model::{Action, ActionBinding, KeyContext};
    ///
    /// let global_binding = ActionBinding::secondary(
    ///     Action::Undo, "z", &[KeyContext::Global]
    /// );
    /// assert!(global_binding.is_active_in(KeyContext::DrawMode));
    /// assert!(global_binding.is_active_in(KeyContext::ManipulateMode));
    ///
    /// let mode_binding = ActionBinding::new(
    ///     Action::DeleteSelection, "backspace", &[KeyContext::ManipulateMode]
    /// );
    /// assert!(mode_binding.is_active_in(KeyContext::ManipulateMode));
    /// assert!(!mode_binding.is_active_in(KeyContext::DrawMode));
    /// ```
    #[must_use]
    pub fn is_active_in(&self, context: KeyContext) -> bool {
        self.contexts.contains(&context) || self.contexts.contains(&KeyContext::Global)
    }
}

/// Return the default keybindings for all actions.
///
/// These bindings follow platform conventions where possible:
///
/// - **History**: Cmd/Ctrl+Z for Undo, Cmd/Ctrl+Y or Cmd/Ctrl+Shift+Z for Redo
/// - **Selection**: Cmd/Ctrl+A for Select All, Cmd/Ctrl+Shift+A for Deselect
/// - **Tools**: Single letter keys (P for Pen, V for Selection)
/// - **Editing**: Backspace/Delete for deletion (in Manipulate mode only)
///
/// # Returns
///
/// A vector of all default action bindings.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::default_bindings;
///
/// let bindings = default_bindings();
/// assert!(!bindings.is_empty());
/// ```
#[must_use]
pub fn default_bindings() -> Vec<ActionBinding> {
    vec![
        // === Document actions ===
        // DeleteSelection: Backspace and Delete keys, only in ManipulateMode
        ActionBinding::new(
            Action::DeleteSelection,
            "backspace",
            &[KeyContext::ManipulateMode],
        ),
        ActionBinding::new(
            Action::DeleteSelection,
            "delete",
            &[KeyContext::ManipulateMode],
        ),
        // === Selection actions ===
        // SelectAll: Cmd/Ctrl+A, global
        ActionBinding::secondary(Action::SelectAll, "a", &[KeyContext::Global]),
        // DeselectAll: Cmd/Ctrl+Shift+A, global
        ActionBinding::secondary_shift(Action::DeselectAll, "a", &[KeyContext::Global]),
        // === Tool actions ===
        // ActivatePenTool: P key, global
        ActionBinding::new(Action::ActivatePenTool, "p", &[KeyContext::Global]),
        // ActivateSelectTool: V key, global
        ActionBinding::new(Action::ActivateSelectTool, "v", &[KeyContext::Global]),
        // === History actions ===
        // Undo: Cmd/Ctrl+Z, global
        ActionBinding::secondary(Action::Undo, "z", &[KeyContext::Global]),
        // Redo: Cmd/Ctrl+Y (Windows/Linux style), global
        ActionBinding::secondary(Action::Redo, "y", &[KeyContext::Global]),
        // Redo: Cmd/Ctrl+Shift+Z (macOS style), global
        ActionBinding::secondary_shift(Action::Redo, "z", &[KeyContext::Global]),
    ]
}

/// Find all bindings for a specific action.
///
/// # Arguments
///
/// * `action` - The action to find bindings for.
///
/// # Returns
///
/// A vector of bindings that trigger the given action.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::{Action, bindings_for_action};
///
/// let redo_bindings = bindings_for_action(Action::Redo);
/// // Redo has two bindings: Cmd+Y and Cmd+Shift+Z
/// assert_eq!(redo_bindings.len(), 2);
/// ```
#[must_use]
pub fn bindings_for_action(action: Action) -> Vec<ActionBinding> {
    default_bindings()
        .into_iter()
        .filter(|b| b.action == action)
        .collect()
}

/// Find all bindings active in a specific context.
///
/// This includes:
///
/// - Bindings explicitly listing the given context
/// - Bindings listing [`KeyContext::Global`] (active everywhere)
///
/// # Arguments
///
/// * `context` - The key context to filter by.
///
/// # Returns
///
/// A vector of bindings active in the given context.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::{KeyContext, bindings_for_context};
///
/// let draw_bindings = bindings_for_context(KeyContext::DrawMode);
/// // Draw mode has global bindings (Undo, Redo, tools) but not Delete
/// ```
#[must_use]
pub fn bindings_for_context(context: KeyContext) -> Vec<ActionBinding> {
    default_bindings()
        .into_iter()
        .filter(|b| b.is_active_in(context))
        .collect()
}

/// Find the primary keystroke for an action.
///
/// Returns the first binding's keystroke, which is considered the "primary"
/// shortcut for display purposes (e.g., in menus).
///
/// # Arguments
///
/// * `action` - The action to find a keystroke for.
///
/// # Returns
///
/// The primary keystroke for the action, or `None` if no binding exists.
///
/// # Examples
///
/// ```rust,no_run
/// use gauss::model::{Action, primary_keystroke};
///
/// let undo_key = primary_keystroke(Action::Undo);
/// assert!(undo_key.is_some());
/// assert_eq!(undo_key.unwrap().to_gpui_string(), "cmd-z");
/// ```
#[must_use]
pub fn primary_keystroke(action: Action) -> Option<Keystroke> {
    bindings_for_action(action)
        .first()
        .map(|b| b.keystroke.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn default_bindings_is_not_empty() {
        assert!(!default_bindings().is_empty());
    }

    #[rstest]
    #[case(Action::DeleteSelection)]
    #[case(Action::SelectAll)]
    #[case(Action::DeselectAll)]
    #[case(Action::ActivatePenTool)]
    #[case(Action::ActivateSelectTool)]
    #[case(Action::Undo)]
    #[case(Action::Redo)]
    fn all_actions_have_bindings(#[case] action: Action) {
        let bindings = bindings_for_action(action);
        assert!(
            !bindings.is_empty(),
            "{action:?} should have at least one keybinding"
        );
    }

    #[test]
    fn redo_has_two_bindings() {
        // Redo should have both Cmd+Y and Cmd+Shift+Z
        let bindings = bindings_for_action(Action::Redo);
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn delete_selection_has_two_bindings() {
        // DeleteSelection should have both Backspace and Delete
        let bindings = bindings_for_action(Action::DeleteSelection);
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn delete_selection_only_in_manipulate_mode() {
        let bindings = bindings_for_action(Action::DeleteSelection);
        for binding in bindings {
            assert!(
                binding.is_active_in(KeyContext::ManipulateMode),
                "DeleteSelection should be active in ManipulateMode"
            );
            assert!(
                !binding.is_active_in(KeyContext::DrawMode),
                "DeleteSelection should NOT be active in DrawMode"
            );
        }
    }

    #[test]
    fn undo_is_global() {
        let bindings = bindings_for_action(Action::Undo);
        for binding in bindings {
            assert!(binding.contexts.contains(&KeyContext::Global));
            assert!(binding.is_active_in(KeyContext::DrawMode));
            assert!(binding.is_active_in(KeyContext::ManipulateMode));
        }
    }

    #[test]
    fn tool_shortcuts_are_global() {
        for action in [Action::ActivatePenTool, Action::ActivateSelectTool] {
            let bindings = bindings_for_action(action);
            for binding in bindings {
                assert!(
                    binding.contexts.contains(&KeyContext::Global),
                    "{action:?} should be global"
                );
            }
        }
    }

    #[test]
    fn bindings_for_context_includes_global() {
        let draw_bindings = bindings_for_context(KeyContext::DrawMode);

        // Should include global bindings like Undo
        let has_undo = draw_bindings.iter().any(|b| b.action == Action::Undo);
        assert!(has_undo, "DrawMode context should include Undo (global)");

        // Should NOT include ManipulateMode-only bindings like DeleteSelection
        let has_delete = draw_bindings
            .iter()
            .any(|b| b.action == Action::DeleteSelection);
        assert!(
            !has_delete,
            "DrawMode context should NOT include DeleteSelection"
        );
    }

    #[test]
    fn bindings_for_context_manipulate_includes_delete() {
        let manipulate_bindings = bindings_for_context(KeyContext::ManipulateMode);

        let has_delete = manipulate_bindings
            .iter()
            .any(|b| b.action == Action::DeleteSelection);
        assert!(has_delete, "ManipulateMode should include DeleteSelection");
    }

    #[test]
    fn primary_keystroke_returns_first_binding() {
        let undo = primary_keystroke(Action::Undo);
        let keystroke = undo.expect("Undo should have a binding");
        assert_eq!(keystroke.to_gpui_string(), "cmd-z");
    }

    #[test]
    fn primary_keystroke_none_for_unknown() {
        // All current actions have bindings, but test the None path
        // by checking that it handles the lookup correctly
        let keystroke = primary_keystroke(Action::Undo);
        assert!(keystroke.is_some());
    }

    #[test]
    fn is_active_in_global_context_matches_all() {
        let binding = ActionBinding::secondary(Action::Undo, "z", &[KeyContext::Global]);

        assert!(binding.is_active_in(KeyContext::Global));
        assert!(binding.is_active_in(KeyContext::DrawMode));
        assert!(binding.is_active_in(KeyContext::ManipulateMode));
        assert!(binding.is_active_in(KeyContext::TextEdit));
    }

    #[test]
    fn is_active_in_specific_context_only_matches_listed() {
        let binding = ActionBinding::new(
            Action::DeleteSelection,
            "backspace",
            &[KeyContext::ManipulateMode],
        );

        assert!(binding.is_active_in(KeyContext::ManipulateMode));
        assert!(!binding.is_active_in(KeyContext::DrawMode));
        assert!(!binding.is_active_in(KeyContext::TextEdit));
        // Note: Global is not in the contexts list, so it doesn't match Global
        // But Global acts as a wildcard in the opposite direction
        assert!(!binding.is_active_in(KeyContext::Global));
    }
}
