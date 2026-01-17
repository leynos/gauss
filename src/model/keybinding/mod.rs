//! Action keybinding registry.
//!
//! This module provides the mapping between [`Action`] variants and keyboard
//! shortcuts, including the key contexts in which each binding is active.
//!
//! # Design
//!
//! The keybinding registry is UI-framework independent, enabling:
//!
//! - Testing of keybinding logic without a UI framework
//! - Generation of framework-specific keybindings at startup
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

#[cfg(test)]
mod tests;

use std::sync::LazyLock;

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

/// Cached default bindings using `LazyLock` to avoid repeated allocations.
static DEFAULT_BINDINGS: LazyLock<Vec<ActionBinding>> = LazyLock::new(|| {
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
        // InsertAnchorOnSegment: I key, only in ManipulateMode
        ActionBinding::new(
            Action::InsertAnchorOnSegment,
            "i",
            &[KeyContext::ManipulateMode],
        ),
        // RaiseSelection: Cmd/Ctrl+], only in ManipulateMode
        ActionBinding::secondary(Action::RaiseSelection, "]", &[KeyContext::ManipulateMode]),
        // LowerSelection: Cmd/Ctrl+[, only in ManipulateMode
        ActionBinding::secondary(Action::LowerSelection, "[", &[KeyContext::ManipulateMode]),
        // ToggleSegmentKind: Tab, only in ManipulateMode
        ActionBinding::new(
            Action::ToggleSegmentKind,
            "tab",
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
        // Document Undo: Cmd/Ctrl+Z, global
        ActionBinding::secondary(Action::Undo, "z", &[KeyContext::Global]),
        // Document Redo: Cmd/Ctrl+Y, global
        ActionBinding::secondary(Action::Redo, "y", &[KeyContext::Global]),
        // Selection Undo: Cmd/Ctrl+Shift+Z, global
        // Note: This deviates from standard macOS Redo to support separate
        // selection history traversal.
        ActionBinding::secondary_shift(Action::SelectionUndo, "z", &[KeyContext::Global]),
        // Selection Redo: Cmd/Ctrl+Shift+Y, global
        ActionBinding::secondary_shift(Action::SelectionRedo, "y", &[KeyContext::Global]),
    ]
});

/// Return the default keybindings for all actions.
///
/// These bindings follow platform conventions where possible:
///
/// - **Document History**: Cmd/Ctrl+Z for Undo, Cmd/Ctrl+Y for Redo
/// - **Selection History**: Cmd/Ctrl+Shift+Z for Selection Undo,
///   Cmd/Ctrl+Shift+Y for Selection Redo
/// - **Selection**: Cmd/Ctrl+A for Select All, Cmd/Ctrl+Shift+A for Deselect
/// - **Tools**: Single letter keys (P for Pen, V for Selection)
/// - **Editing**: Backspace/Delete for deletion, I for insert anchor, Tab for
///   segment toggling, Cmd/Ctrl+[ and Cmd/Ctrl+] for z-order (Manipulate only)
///
/// Note: Gauss maintains separate undo/redo stacks for document edits and
/// selection changes, enabling independent traversal of edit and selection
/// states. This deviates from the macOS convention of Cmd+Shift+Z for Redo.
///
/// # Returns
///
/// A slice of all default action bindings. The bindings are cached using
/// `LazyLock` to avoid repeated allocations.
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
pub fn default_bindings() -> &'static [ActionBinding] {
    &DEFAULT_BINDINGS
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
/// // Redo has one binding: Cmd+Y
/// assert_eq!(redo_bindings.len(), 1);
/// ```
#[must_use]
pub fn bindings_for_action(action: Action) -> Vec<ActionBinding> {
    default_bindings()
        .iter()
        .filter(|b| b.action == action)
        .cloned()
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
        .iter()
        .filter(|b| b.is_active_in(context))
        .cloned()
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
/// let undo_key = undo_key.expect("expected primary_keystroke(Action::Undo) to be Some(...) in doc example");
/// assert_eq!(undo_key.key, "z");
/// assert!(undo_key.modifiers.secondary);
/// ```
#[must_use]
pub fn primary_keystroke(action: Action) -> Option<Keystroke> {
    bindings_for_action(action)
        .first()
        .map(|b| b.keystroke.clone())
}
