//! Unit tests for the action module.

use rstest::rstest;

use super::{
    Action, ActionKind, Color, Degrees, Dimensions, Opacity, Point, Points, Position, Rgb8,
    Rotation, Size, StrokeWidth, UnitF32,
};

#[rstest]
#[case(Action::DeleteSelection, ActionKind::Document)]
#[case(Action::InsertAnchorOnSegment, ActionKind::Document)]
#[case(Action::DeleteSelectedAnchors, ActionKind::Document)]
#[case(Action::RaiseSelection, ActionKind::Document)]
#[case(Action::LowerSelection, ActionKind::Document)]
#[case(Action::ToggleSegmentKind, ActionKind::Document)]
#[case(Action::SetStrokeColor(Color::new(Rgb8 { r: 0, g: 0, b: 0 })), ActionKind::Document)]
#[case(Action::SetStrokeWidth(StrokeWidth::new(Points(1.0)).expect("failed to construct StrokeWidth")), ActionKind::Document)]
#[case(Action::SetStrokeOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")), ActionKind::Document)]
#[case(Action::SetFillColor(Color::new(Rgb8 { r: 255, g: 255, b: 255 })), ActionKind::Document)]
#[case(Action::SetFillOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")), ActionKind::Document)]
#[case(Action::ToggleNoFill, ActionKind::Document)]
#[case(
    Action::SetObjectPosition(
        Position::new(Point { x: 0.0, y: 0.0 }).expect("failed to construct Position")
    ),
    ActionKind::Document
)]
#[case(
    Action::SetObjectSize(
        Size::new(Dimensions { width: 100.0, height: 100.0 }).expect("failed to construct Size")
    ),
    ActionKind::Document
)]
#[case(
    Action::SetObjectRotation(
        Rotation::new(Degrees(0.0)).expect("failed to construct Rotation")
    ),
    ActionKind::Document
)]
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
#[case(Action::SetStrokeColor(Color::new(Rgb8 { r: 0, g: 0, b: 0 })), "Set Stroke Colour")]
#[case(Action::SetStrokeWidth(StrokeWidth::new(Points(1.0)).expect("failed to construct StrokeWidth")), "Set Stroke Width")]
#[case(Action::SetStrokeOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")), "Set Stroke Opacity")]
#[case(Action::SetFillColor(Color::new(Rgb8 { r: 255, g: 255, b: 255 })), "Set Fill Colour")]
#[case(Action::SetFillOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")), "Set Fill Opacity")]
#[case(Action::ToggleNoFill, "Toggle No Fill")]
#[case(
    Action::SetObjectPosition(
        Position::new(Point { x: 0.0, y: 0.0 }).expect("failed to construct Position")
    ),
    "Set Position"
)]
#[case(
    Action::SetObjectSize(
        Size::new(Dimensions { width: 100.0, height: 100.0 }).expect("failed to construct Size")
    ),
    "Set Size"
)]
#[case(
    Action::SetObjectRotation(
        Rotation::new(Degrees(0.0)).expect("failed to construct Rotation")
    ),
    "Set Rotation"
)]
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
#[case(Action::SetStrokeColor(Color::new(Rgb8 { r: 0, g: 0, b: 0 })))]
#[case(Action::SetStrokeWidth(StrokeWidth::new(Points(1.0)).expect("failed to construct StrokeWidth")))]
#[case(Action::SetStrokeOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")))]
#[case(Action::SetFillColor(Color::new(Rgb8 { r: 255, g: 255, b: 255 })))]
#[case(Action::SetFillOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")))]
#[case(Action::ToggleNoFill)]
#[case(Action::SetObjectPosition(Position::new(Point { x: 0.0, y: 0.0 }).expect("failed to construct Position")))]
#[case(Action::SetObjectSize(Size::new(Dimensions { width: 100.0, height: 100.0 }).expect("failed to construct Size")))]
#[case(Action::SetObjectRotation(Rotation::new(Degrees(0.0)).expect("failed to construct Rotation")))]
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
#[case(Action::SetStrokeColor(Color::new(Rgb8 { r: 0, g: 0, b: 0 })))]
#[case(Action::SetStrokeWidth(StrokeWidth::new(Points(1.0)).expect("failed to construct StrokeWidth")))]
#[case(Action::SetStrokeOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")))]
#[case(Action::SetFillColor(Color::new(Rgb8 { r: 255, g: 255, b: 255 })))]
#[case(Action::SetFillOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")))]
#[case(Action::ToggleNoFill)]
#[case(Action::SetObjectPosition(Position::new(Point { x: 0.0, y: 0.0 }).expect("failed to construct Position")))]
#[case(Action::SetObjectSize(Size::new(Dimensions { width: 100.0, height: 100.0 }).expect("failed to construct Size")))]
#[case(Action::SetObjectRotation(Rotation::new(Degrees(0.0)).expect("failed to construct Rotation")))]
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
        Action::SetStrokeColor(Color::new(Rgb8 { r: 0, g: 0, b: 0 })),
        Action::SetStrokeWidth(StrokeWidth::new(Points(1.0)).expect("failed to construct StrokeWidth")),
        Action::SetStrokeOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")),
        Action::SetFillColor(Color::new(Rgb8 { r: 255, g: 255, b: 255 })),
        Action::SetFillOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")),
        Action::ToggleNoFill,
        Action::SetObjectPosition(Position::new(Point { x: 0.0, y: 0.0 }).expect("failed to construct Position")),
        Action::SetObjectSize(Size::new(Dimensions { width: 100.0, height: 100.0 }).expect("failed to construct Size")),
        Action::SetObjectRotation(Rotation::new(Degrees(0.0)).expect("failed to construct Rotation")),
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
#[case(Action::Undo, "Undo")]
#[case(Action::Redo, "Redo")]
#[case(Action::SelectionUndo, "SelectionUndo")]
#[case(Action::SelectionRedo, "SelectionRedo")]
#[case(Action::SetStrokeColor(Color::new(Rgb8 { r: 0, g: 0, b: 0 })), "SetStrokeColor")]
#[case(Action::SetStrokeWidth(StrokeWidth::new(Points(1.0)).expect("failed to construct StrokeWidth")), "SetStrokeWidth")]
#[case(Action::SetStrokeOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")), "SetStrokeOpacity")]
#[case(Action::SetFillColor(Color::new(Rgb8 { r: 255, g: 255, b: 255 })), "SetFillColor")]
#[case(Action::SetFillOpacity(Opacity::new(UnitF32::try_from(1.0).expect("failed to construct UnitF32")).expect("failed to construct Opacity")), "SetFillOpacity")]
#[case(Action::ToggleNoFill, "ToggleNoFill")]
#[case(
    Action::SetObjectPosition(
        Position::new(Point { x: 0.0, y: 0.0 }).expect("failed to construct Position")
    ),
    "SetObjectPosition"
)]
#[case(
    Action::SetObjectSize(
        Size::new(Dimensions { width: 100.0, height: 100.0 }).expect("failed to construct Size")
    ),
    "SetObjectSize"
)]
#[case(
    Action::SetObjectRotation(
        Rotation::new(Degrees(0.0)).expect("failed to construct Rotation")
    ),
    "SetObjectRotation"
)]
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

#[test]
fn action_is_eq_and_hash() {
    // Verify Action implements Eq and Hash (for use as HashMap keys)
    fn assert_eq<T: Eq>(_: T) {}
    fn assert_hash<T: std::hash::Hash>(_: T) {}

    let action = Action::Undo;
    assert_eq(action);
    assert_hash(action);
}
