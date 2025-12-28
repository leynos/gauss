//! Tests for `action_bridge` module.

use super::*;
use crate::model::{Action, default_bindings};
use crate::ui::phase0_shell::draw::ToolMode;

// === context_for_tool_mode tests ===

#[test]
fn context_for_tool_mode_maps_draw_to_draw_mode() {
    assert_eq!(context_for_tool_mode(ToolMode::Draw), KeyContext::DrawMode);
}

#[test]
fn context_for_tool_mode_maps_manipulate_to_manipulate_mode() {
    assert_eq!(
        context_for_tool_mode(ToolMode::Manipulate),
        KeyContext::ManipulateMode
    );
}

// === CollectedBindings tests ===

/// Helper to verify a binding vector is populated.
fn assert_bindings_populated(bindings: &[KeyBinding], action_name: &str) {
    assert!(
        !bindings.is_empty(),
        "expected {action_name} bindings to be populated"
    );
}

#[test]
fn collected_bindings_populates_undo_redo() {
    let bindings = CollectedBindings::from_default_bindings();
    assert_bindings_populated(&bindings.undo, "undo");
    assert_bindings_populated(&bindings.redo, "redo");
    assert_bindings_populated(&bindings.selection_undo, "selection_undo");
    assert_bindings_populated(&bindings.selection_redo, "selection_redo");
}

#[test]
fn collected_bindings_populates_selection_actions() {
    let bindings = CollectedBindings::from_default_bindings();
    assert_bindings_populated(&bindings.select_all, "select_all");
    assert_bindings_populated(&bindings.deselect_all, "deselect_all");
    assert_bindings_populated(&bindings.delete_selection, "delete_selection");
}

#[test]
fn collected_bindings_populates_tool_actions() {
    let bindings = CollectedBindings::from_default_bindings();
    assert_bindings_populated(&bindings.activate_pen_tool, "activate_pen_tool");
    assert_bindings_populated(&bindings.activate_select_tool, "activate_select_tool");
}

#[test]
fn global_bindings_are_expanded_to_all_contexts() {
    let collected = CollectedBindings::from_default_bindings();

    // Global bindings (like Undo) should be expanded to all contexts.
    // The undo action has 1 binding with Global context, so it should
    // be expanded to one binding per context.
    let context_count = KeyContext::all().len();
    assert!(
        collected.undo.len() >= context_count,
        "expected undo bindings to be expanded to all {} contexts, got {}",
        context_count,
        collected.undo.len()
    );
}

#[test]
fn delete_selection_binding_is_registered() {
    let collected = CollectedBindings::from_default_bindings();
    assert!(
        !collected.delete_selection.is_empty(),
        "expected delete_selection bindings to be registered"
    );
}

// === Action mapping coverage ===

/// Helper to get the binding vector for an action.
fn bindings_for_action(collected: &CollectedBindings, action: Action) -> &[KeyBinding] {
    match action {
        Action::DeleteSelection => &collected.delete_selection,
        Action::SelectAll => &collected.select_all,
        Action::DeselectAll => &collected.deselect_all,
        Action::ActivatePenTool => &collected.activate_pen_tool,
        Action::ActivateSelectTool => &collected.activate_select_tool,
        Action::Undo => &collected.undo,
        Action::Redo => &collected.redo,
        Action::SelectionUndo => &collected.selection_undo,
        Action::SelectionRedo => &collected.selection_redo,
    }
}

#[test]
fn all_model_actions_have_corresponding_gpui_bindings() {
    let collected = CollectedBindings::from_default_bindings();

    // Verify that every Action variant used in default_bindings() has bindings.
    for binding in default_bindings() {
        let gpui_bindings = bindings_for_action(&collected, binding.action);
        assert!(
            !gpui_bindings.is_empty(),
            "missing bindings for {:?}",
            binding.action
        );
    }
}
