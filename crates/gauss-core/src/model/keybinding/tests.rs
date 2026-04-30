//! Tests for keybinding module.

use super::*;
use crate::model::Modifiers;
use rstest::rstest;

fn assert_keystroke_eq(keystroke: &Keystroke, key: &str, expected_modifiers: Modifiers) {
    assert_eq!(
        keystroke.key, key,
        "expected key '{key}', got '{}'",
        keystroke.key
    );
    assert_eq!(
        keystroke.modifiers.secondary, expected_modifiers.secondary,
        "expected secondary modifier {} for key '{key}'",
        expected_modifiers.secondary
    );
    assert_eq!(
        keystroke.modifiers.shift, expected_modifiers.shift,
        "expected shift modifier {} for key '{key}'",
        expected_modifiers.shift
    );
    assert_eq!(
        keystroke.modifiers.control, expected_modifiers.control,
        "expected control modifier {} for key '{key}'",
        expected_modifiers.control
    );
    assert_eq!(
        keystroke.modifiers.alt, expected_modifiers.alt,
        "expected alt modifier {} for key '{key}'",
        expected_modifiers.alt
    );
}

fn assert_active_contexts(binding: &ActionBinding, active: &[KeyContext], inactive: &[KeyContext]) {
    for context in active {
        assert!(
            binding.is_active_in(*context),
            "expected binding {:?} to be active in {:?}",
            binding.action,
            context
        );
    }
    for context in inactive {
        assert!(
            !binding.is_active_in(*context),
            "expected binding {:?} to be inactive in {:?}",
            binding.action,
            context
        );
    }
}

/// Helper to assert that an action has exactly one binding with specific keystroke and modifiers
fn assert_action_has_single_binding_with_modifiers(
    action: Action,
    expected_key: &str,
    expected_modifiers: Modifiers,
) {
    let bindings = bindings_for_action(action);
    let [binding] = bindings.as_slice() else {
        panic!("expected exactly one binding for {action:?}");
    };
    assert_keystroke_eq(&binding.keystroke, expected_key, expected_modifiers);
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

#[rstest]
#[case(Action::SelectionUndo, "z")]
#[case(Action::SelectionRedo, "y")]
fn selection_actions_have_shift_modifier(#[case] action: Action, #[case] key: &str) {
    // Selection undo/redo should be Cmd+Shift+[Z/Y]
    assert_action_has_single_binding_with_modifiers(
        action,
        key,
        Modifiers {
            secondary: true,
            shift: true,
            control: false,
            alt: false,
        },
    );
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
    assert_keystroke_eq(
        &keystroke,
        "z",
        Modifiers {
            secondary: true,
            shift: false,
            control: false,
            alt: false,
        },
    );
}

#[test]
fn is_active_in_global_context_matches_all() {
    let binding = ActionBinding::secondary(Action::Undo, "z", &[KeyContext::Global]);

    assert_active_contexts(
        &binding,
        &[
            KeyContext::Global,
            KeyContext::DrawMode,
            KeyContext::ManipulateMode,
            KeyContext::TextEdit,
        ],
        &[],
    );
}

#[test]
fn is_active_in_specific_context_only_matches_listed() {
    let binding = ActionBinding::new(
        Action::DeleteSelection,
        "backspace",
        &[KeyContext::ManipulateMode],
    );

    // Note: Global is not in the contexts list, so it doesn't match Global
    // But Global acts as a wildcard in the opposite direction
    assert_active_contexts(
        &binding,
        &[KeyContext::ManipulateMode],
        &[
            KeyContext::DrawMode,
            KeyContext::TextEdit,
            KeyContext::Global,
        ],
    );
}
