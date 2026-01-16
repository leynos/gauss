//! Tests for keybinding module.

use super::*;
use rstest::rstest;

#[expect(
    clippy::too_many_arguments,
    reason = "Test helper mirrors explicit per-modifier assertions required by the test cases."
)]
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "Test helper mirrors explicit per-modifier assertions required by the test cases."
)]
fn assert_single_binding_with_modifiers(
    action: Action,
    expected_key: &str,
    expected_secondary: bool,
    expected_shift: bool,
    expected_control: bool,
    expected_alt: bool,
) {
    let bindings = bindings_for_action(action);
    assert_eq!(bindings.len(), 1);
    let binding = bindings.first().expect("should have at least one binding");
    assert_eq!(binding.keystroke.key, expected_key);
    assert_eq!(binding.keystroke.modifiers.secondary, expected_secondary);
    assert_eq!(binding.keystroke.modifiers.shift, expected_shift);
    assert_eq!(binding.keystroke.modifiers.control, expected_control);
    assert_eq!(binding.keystroke.modifiers.alt, expected_alt);
}

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
#[case(Action::SelectionUndo)]
#[case(Action::SelectionRedo)]
fn all_actions_have_bindings(#[case] action: Action) {
    let bindings = bindings_for_action(action);
    assert!(
        !bindings.is_empty(),
        "{action:?} should have at least one keybinding"
    );
}

#[test]
fn redo_has_one_binding() {
    // Redo only has Cmd+Y (Cmd+Shift+Z is now SelectionUndo)
    let bindings = bindings_for_action(Action::Redo);
    assert_eq!(bindings.len(), 1);
}

#[test]
fn selection_undo_has_shift_modifier() {
    // SelectionUndo should be Cmd+Shift+Z
    assert_single_binding_with_modifiers(Action::SelectionUndo, "z", true, true, false, false);
}

#[test]
fn selection_redo_has_shift_modifier() {
    // SelectionRedo should be Cmd+Shift+Y
    assert_single_binding_with_modifiers(Action::SelectionRedo, "y", true, true, false, false);
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
    assert_eq!(keystroke.key, "z");
    assert!(keystroke.modifiers.secondary);
    assert!(!keystroke.modifiers.shift);
    assert!(!keystroke.modifiers.control);
    assert!(!keystroke.modifiers.alt);
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
