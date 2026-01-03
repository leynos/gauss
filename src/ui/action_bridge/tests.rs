//! Tests for the `action_bridge` module.
//!
//! Covers:
//!
//! - [`context_for_tool_mode`] mapping between tool modes and key contexts
//! - [`CollectedBindings`] population from default bindings
//! - Global binding expansion to all contexts
//! - Coverage verification ensuring all model actions have GPUI bindings

use rstest::rstest;

use super::*;
use crate::model::{Action, default_bindings};
use crate::ui::phase0_shell::draw::ToolMode;

// === context_for_tool_mode tests ===

#[rstest]
#[case(ToolMode::Draw, KeyContext::DrawMode)]
#[case(ToolMode::Manipulate, KeyContext::ManipulateMode)]
fn context_for_tool_mode_maps_correctly(#[case] mode: ToolMode, #[case] expected: KeyContext) {
    assert_eq!(context_for_tool_mode(mode), expected);
}

// === CollectedBindings tests ===

/// Binding field accessor for parameterised tests.
fn get_binding_field<'a>(collected: &'a CollectedBindings, name: &str) -> &'a [KeyBinding] {
    match name {
        "undo" => &collected.undo,
        "redo" => &collected.redo,
        "selection_undo" => &collected.selection_undo,
        "selection_redo" => &collected.selection_redo,
        "select_all" => &collected.select_all,
        "deselect_all" => &collected.deselect_all,
        "delete_selection" => &collected.delete_selection,
        "activate_pen_tool" => &collected.activate_pen_tool,
        "activate_select_tool" => &collected.activate_select_tool,
        "insert_anchor_on_segment" => &collected.insert_anchor_on_segment,
        "delete_selected_anchors" => &collected.delete_selected_anchors,
        "raise_selection" => &collected.raise_selection,
        "lower_selection" => &collected.lower_selection,
        "toggle_segment_kind" => &collected.toggle_segment_kind,
        _ => panic!("unknown binding field: {name}"),
    }
}

#[rstest]
#[case("undo")]
#[case("redo")]
#[case("selection_undo")]
#[case("selection_redo")]
#[case("select_all")]
#[case("deselect_all")]
#[case("delete_selection")]
#[case("activate_pen_tool")]
#[case("activate_select_tool")]
#[case("insert_anchor_on_segment")]
#[case("raise_selection")]
#[case("lower_selection")]
#[case("toggle_segment_kind")]
fn collected_bindings_populates_action(#[case] action_name: &str) {
    let bindings = CollectedBindings::from_default_bindings();
    let field = get_binding_field(&bindings, action_name);
    assert!(
        !field.is_empty(),
        "expected {action_name} bindings to be populated"
    );
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
        Action::InsertAnchorOnSegment => &collected.insert_anchor_on_segment,
        Action::DeleteSelectedAnchors => &collected.delete_selected_anchors,
        Action::RaiseSelection => &collected.raise_selection,
        Action::LowerSelection => &collected.lower_selection,
        Action::ToggleSegmentKind => &collected.toggle_segment_kind,
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
