//! Unit tests for the action module.

use rstest::rstest;

use super::{Action, ActionKind};

#[rstest]
#[case(Action::DeleteSelection, ActionKind::Document)]
#[case(Action::InsertAnchorOnSegment, ActionKind::Document)]
#[case(Action::DeleteSelectedAnchors, ActionKind::Document)]
#[case(Action::RaiseSelection, ActionKind::Document)]
#[case(Action::LowerSelection, ActionKind::Document)]
#[case(Action::ToggleSegmentKind, ActionKind::Document)]
#[case(Action::SelectAll, ActionKind::Editor)]
#[case(Action::DeselectAll, ActionKind::Editor)]
#[case(Action::ActivatePenTool, ActionKind::Editor)]
#[case(Action::ActivateSelectTool, ActionKind::Editor)]
#[case(Action::Undo, ActionKind::Editor)]
#[case(Action::Redo, ActionKind::Editor)]
#[case(Action::SelectionUndo, ActionKind::Editor)]
#[case(Action::SelectionRedo, ActionKind::Editor)]
fn action_kind_is_correct(#[case] action: Action, #[case] expected: ActionKind) {
    assert_eq!(action.kind(), expected);
}

#[rstest]
#[case(Action::DeleteSelection, "Delete Selection")]
#[case(Action::InsertAnchorOnSegment, "Insert Anchor")]
#[case(Action::DeleteSelectedAnchors, "Delete Anchors")]
#[case(Action::RaiseSelection, "Raise")]
#[case(Action::LowerSelection, "Lower")]
#[case(Action::ToggleSegmentKind, "Toggle Segment")]
#[case(Action::SelectAll, "Select All")]
#[case(Action::DeselectAll, "Deselect All")]
#[case(Action::ActivatePenTool, "Pen Tool")]
#[case(Action::ActivateSelectTool, "Select Tool")]
#[case(Action::Undo, "Undo")]
#[case(Action::Redo, "Redo")]
#[case(Action::SelectionUndo, "Selection Undo")]
#[case(Action::SelectionRedo, "Selection Redo")]
fn action_name_is_correct(#[case] action: Action, #[case] expected: &str) {
    assert_eq!(action.name(), expected);
}

#[rstest]
#[case(Action::DeleteSelection)]
#[case(Action::InsertAnchorOnSegment)]
#[case(Action::DeleteSelectedAnchors)]
#[case(Action::RaiseSelection)]
#[case(Action::LowerSelection)]
#[case(Action::ToggleSegmentKind)]
#[case(Action::SelectAll)]
#[case(Action::DeselectAll)]
#[case(Action::ActivatePenTool)]
#[case(Action::ActivateSelectTool)]
#[case(Action::Undo)]
#[case(Action::Redo)]
#[case(Action::SelectionUndo)]
#[case(Action::SelectionRedo)]
fn actions_have_nonempty_names(#[case] action: Action) {
    assert!(!action.name().is_empty());
}

#[rstest]
#[case(Action::DeleteSelection)]
#[case(Action::InsertAnchorOnSegment)]
#[case(Action::DeleteSelectedAnchors)]
#[case(Action::RaiseSelection)]
#[case(Action::LowerSelection)]
#[case(Action::ToggleSegmentKind)]
fn document_actions_require_selection(#[case] action: Action) {
    assert!(action.requires_selection());
}

#[rstest]
#[case(Action::SelectAll)]
#[case(Action::DeselectAll)]
#[case(Action::ActivatePenTool)]
#[case(Action::ActivateSelectTool)]
#[case(Action::Undo)]
#[case(Action::Redo)]
#[case(Action::SelectionUndo)]
#[case(Action::SelectionRedo)]
fn non_document_actions_do_not_require_selection(#[case] action: Action) {
    assert!(!action.requires_selection());
}

#[test]
fn document_actions_are_all_accounted_for() {
    // Ensure that any action requiring selection is a Document action.
    //
    // NOTE: This list is intentionally hardcoded rather than generated.
    // When adding a new Action variant, the developer must explicitly add
    // it here, forcing consideration of the selection-requires-Document
    // invariant. A compile error from an unmatched variant in the match
    // arms above will remind you to update this test.
    let all_actions = [
        Action::DeleteSelection,
        Action::InsertAnchorOnSegment,
        Action::DeleteSelectedAnchors,
        Action::RaiseSelection,
        Action::LowerSelection,
        Action::ToggleSegmentKind,
        Action::SelectAll,
        Action::DeselectAll,
        Action::ActivatePenTool,
        Action::ActivateSelectTool,
        Action::Undo,
        Action::Redo,
        Action::SelectionUndo,
        Action::SelectionRedo,
    ];

    for action in all_actions {
        if action.requires_selection() {
            assert_eq!(
                action.kind(),
                ActionKind::Document,
                "{action:?} requires selection but is not Document kind"
            );
        }
    }
}

#[test]
fn action_is_copy() {
    // Verify Action implements Copy (important for ergonomics)
    fn assert_copy<T: Copy>(_: T) {}

    assert_copy(Action::Undo);
}

#[test]
fn action_kind_is_copy() {
    // Verify ActionKind implements Copy
    fn assert_copy<T: Copy>(_: T) {}

    assert_copy(ActionKind::Document);
}
