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
#[case(Action::SetStrokeColor, ActionKind::Document)]
#[case(Action::SetStrokeWidth, ActionKind::Document)]
#[case(Action::SetStrokeOpacity, ActionKind::Document)]
#[case(Action::SetFillColor, ActionKind::Document)]
#[case(Action::SetFillOpacity, ActionKind::Document)]
#[case(Action::ToggleNoFill, ActionKind::Document)]
#[case(Action::SetObjectPosition, ActionKind::Document)]
#[case(Action::SetObjectSize, ActionKind::Document)]
#[case(Action::SetObjectRotation, ActionKind::Document)]
#[case(Action::SelectAll, ActionKind::Editor)]
#[case(Action::DeselectAll, ActionKind::Editor)]
#[case(Action::ActivatePenTool, ActionKind::Editor)]
#[case(Action::ActivateSelectTool, ActionKind::Editor)]
#[case(Action::ActivateMyNewTool, ActionKind::Editor)]
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
#[case(Action::ActivateMyNewTool, "My New Tool")]
#[case(Action::Undo, "Undo")]
#[case(Action::Redo, "Redo")]
#[case(Action::SelectionUndo, "Selection Undo")]
#[case(Action::SelectionRedo, "Selection Redo")]
#[case(Action::SetStrokeColor, "Set Stroke Colour")]
#[case(Action::SetStrokeWidth, "Set Stroke Width")]
#[case(Action::SetStrokeOpacity, "Set Stroke Opacity")]
#[case(Action::SetFillColor, "Set Fill Colour")]
#[case(Action::SetFillOpacity, "Set Fill Opacity")]
#[case(Action::ToggleNoFill, "Toggle No Fill")]
#[case(Action::SetObjectPosition, "Set Position")]
#[case(Action::SetObjectSize, "Set Size")]
#[case(Action::SetObjectRotation, "Set Rotation")]
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
#[case(Action::ActivateMyNewTool)]
#[case(Action::Undo)]
#[case(Action::Redo)]
#[case(Action::SelectionUndo)]
#[case(Action::SelectionRedo)]
#[case(Action::SetStrokeColor)]
#[case(Action::SetStrokeWidth)]
#[case(Action::SetStrokeOpacity)]
#[case(Action::SetFillColor)]
#[case(Action::SetFillOpacity)]
#[case(Action::ToggleNoFill)]
#[case(Action::SetObjectPosition)]
#[case(Action::SetObjectSize)]
#[case(Action::SetObjectRotation)]
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
#[case(Action::SetStrokeColor)]
#[case(Action::SetStrokeWidth)]
#[case(Action::SetStrokeOpacity)]
#[case(Action::SetFillColor)]
#[case(Action::SetFillOpacity)]
#[case(Action::ToggleNoFill)]
#[case(Action::SetObjectPosition)]
#[case(Action::SetObjectSize)]
#[case(Action::SetObjectRotation)]
fn document_actions_require_selection(#[case] action: Action) {
    assert!(action.requires_selection());
}

#[rstest]
#[case(Action::SelectAll)]
#[case(Action::DeselectAll)]
#[case(Action::ActivatePenTool)]
#[case(Action::ActivateSelectTool)]
#[case(Action::ActivateMyNewTool)]
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
        Action::SetStrokeColor,
        Action::SetStrokeWidth,
        Action::SetStrokeOpacity,
        Action::SetFillColor,
        Action::SetFillOpacity,
        Action::ToggleNoFill,
        Action::SetObjectPosition,
        Action::SetObjectSize,
        Action::SetObjectRotation,
        Action::SelectAll,
        Action::DeselectAll,
        Action::ActivatePenTool,
        Action::ActivateSelectTool,
        Action::ActivateMyNewTool,
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

#[rstest]
#[case(Action::DeleteSelection, "DeleteSelection")]
#[case(Action::InsertAnchorOnSegment, "InsertAnchorOnSegment")]
#[case(Action::DeleteSelectedAnchors, "DeleteSelectedAnchors")]
#[case(Action::RaiseSelection, "RaiseSelection")]
#[case(Action::LowerSelection, "LowerSelection")]
#[case(Action::ToggleSegmentKind, "ToggleSegmentKind")]
#[case(Action::SelectAll, "SelectAll")]
#[case(Action::DeselectAll, "DeselectAll")]
#[case(Action::ActivatePenTool, "ActivatePenTool")]
#[case(Action::ActivateSelectTool, "ActivateSelectTool")]
#[case(Action::ActivateMyNewTool, "ActivateMyNewTool")]
#[case(Action::Undo, "Undo")]
#[case(Action::Redo, "Redo")]
#[case(Action::SelectionUndo, "SelectionUndo")]
#[case(Action::SelectionRedo, "SelectionRedo")]
#[case(Action::SetStrokeColor, "SetStrokeColor")]
#[case(Action::SetStrokeWidth, "SetStrokeWidth")]
#[case(Action::SetStrokeOpacity, "SetStrokeOpacity")]
#[case(Action::SetFillColor, "SetFillColor")]
#[case(Action::SetFillOpacity, "SetFillOpacity")]
#[case(Action::ToggleNoFill, "ToggleNoFill")]
#[case(Action::SetObjectPosition, "SetObjectPosition")]
#[case(Action::SetObjectSize, "SetObjectSize")]
#[case(Action::SetObjectRotation, "SetObjectRotation")]
fn action_identifier_is_correct(#[case] action: Action, #[case] expected: &str) {
    assert_eq!(action.identifier(), expected);
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
