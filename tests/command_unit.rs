//! Unit tests for Command dispatch.
//!
//! These tests validate that commands correctly apply and undo document
//! mutations.

use gauss::model::{
    Action, Command, CommandInverse, DeletedShape, Document, SelItem, Selection, UserError,
    prepare_command,
};
use rstest::{fixture, rstest};
use test_support::shapes::{sample_shape, shape_id};

#[fixture]
fn empty_doc() -> Document {
    Document::default()
}

#[fixture]
fn doc_with_two_shapes() -> Document {
    Document {
        shapes: vec![sample_shape(shape_id(1), 0), sample_shape(shape_id(2), 1)],
    }
}

#[fixture]
fn selection_with_first_shape() -> Selection {
    let mut selection = Selection::default();
    selection.toggle(SelItem::Shape(shape_id(1)));
    selection
}

#[rstest]
fn delete_shapes_removes_from_document(mut doc_with_two_shapes: Document) {
    let shape = doc_with_two_shapes
        .shapes
        .first()
        .cloned()
        .expect("fixture should have shapes");
    let cmd = Command::DeleteShapes {
        targets: vec![DeletedShape {
            index: 0,
            shape: shape.clone(),
        }],
    };

    let result = cmd.apply(&mut doc_with_two_shapes);
    assert!(result.is_ok());
    assert_eq!(doc_with_two_shapes.shapes.len(), 1);
}

#[rstest]
fn delete_shapes_inverse_restores(mut doc_with_two_shapes: Document) {
    let original_len = doc_with_two_shapes.shapes.len();
    let shape = doc_with_two_shapes
        .shapes
        .first()
        .cloned()
        .expect("fixture should have shapes");

    let cmd = Command::DeleteShapes {
        targets: vec![DeletedShape { index: 0, shape }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply succeeded");
    assert_eq!(doc_with_two_shapes.shapes.len(), original_len - 1);

    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("undo succeeded");
    assert_eq!(doc_with_two_shapes.shapes.len(), original_len);
}

#[rstest]
fn delete_restores_shape_at_correct_index(mut doc_with_two_shapes: Document) {
    let original_shapes = doc_with_two_shapes.shapes.clone();
    let shape = doc_with_two_shapes
        .shapes
        .first()
        .cloned()
        .expect("fixture should have shapes");

    let cmd = Command::DeleteShapes {
        targets: vec![DeletedShape {
            index: 0,
            shape: shape.clone(),
        }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply succeeded");

    // First shape should be removed, second should now be at index 0
    assert_eq!(
        doc_with_two_shapes
            .shapes
            .first()
            .expect("should have remaining shape")
            .id,
        shape_id(2)
    );

    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("undo succeeded");

    // After undo, shapes should match original order
    assert_eq!(doc_with_two_shapes.shapes, original_shapes);
}

#[rstest]
fn prepare_delete_selection_fails_with_empty_selection(doc_with_two_shapes: Document) {
    let selection = Selection::default();

    let result = prepare_command(Action::DeleteSelection, &doc_with_two_shapes, &selection);

    assert!(matches!(result, Err(UserError::EmptySelection)));
}

#[rstest]
fn prepare_delete_selection_succeeds_with_selection(
    doc_with_two_shapes: Document,
    selection_with_first_shape: Selection,
) {
    let result = prepare_command(
        Action::DeleteSelection,
        &doc_with_two_shapes,
        &selection_with_first_shape,
    );

    let cmd = result.expect("prepare_command should succeed");
    assert!(
        matches!(cmd, Command::DeleteShapes { .. }),
        "expected DeleteShapes command"
    );

    if let Command::DeleteShapes { targets } = cmd {
        assert_eq!(targets.len(), 1);
        assert_eq!(targets.first().expect("should have one target").index, 0);
    }
}

#[rstest]
fn prepare_delete_selection_fails_with_missing_shape(empty_doc: Document) {
    let mut selection = Selection::default();
    selection.toggle(SelItem::Shape(shape_id(999)));

    let result = prepare_command(Action::DeleteSelection, &empty_doc, &selection);

    assert!(matches!(result, Err(UserError::ShapeNotFound(_))));
}

#[rstest]
fn command_name_is_nonempty() {
    let cmd = Command::DeleteShapes { targets: vec![] };
    assert!(!cmd.name().is_empty());
}

#[rstest]
fn command_name_is_delete() {
    let cmd = Command::DeleteShapes { targets: vec![] };
    assert_eq!(cmd.name(), "Delete");
}

#[rstest]
fn command_inverse_name_is_delete() {
    let inverse = CommandInverse::RestoreShapes { targets: vec![] };
    assert_eq!(inverse.name(), "Delete");
}

#[rstest]
fn inverse_name_matches_command_after_apply(mut doc_with_two_shapes: Document) {
    let shape = doc_with_two_shapes
        .shapes
        .first()
        .cloned()
        .expect("fixture should have shapes");
    let cmd = Command::DeleteShapes {
        targets: vec![DeletedShape { index: 0, shape }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply succeeded");

    // Inverse name should match the command name
    assert_eq!(inverse.name(), cmd.name());
    assert_eq!(inverse.name(), "Delete");
}

#[rstest]
fn delete_multiple_shapes_preserves_order(mut doc_with_two_shapes: Document) {
    // Add a third shape
    doc_with_two_shapes
        .shapes
        .push(sample_shape(shape_id(3), 2));
    let original_shapes = doc_with_two_shapes.shapes.clone();

    // Delete first and last shapes
    let targets = vec![
        DeletedShape {
            index: 0,
            shape: original_shapes
                .first()
                .expect("should have first shape")
                .clone(),
        },
        DeletedShape {
            index: 2,
            shape: original_shapes
                .get(2)
                .expect("should have third shape")
                .clone(),
        },
    ];

    let cmd = Command::DeleteShapes { targets };
    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply succeeded");

    // Only middle shape should remain
    assert_eq!(doc_with_two_shapes.shapes.len(), 1);
    assert_eq!(
        doc_with_two_shapes
            .shapes
            .first()
            .expect("should have remaining shape")
            .id,
        shape_id(2)
    );

    // Undo should restore all shapes in original order
    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("undo succeeded");
    assert_eq!(doc_with_two_shapes.shapes, original_shapes);
}

#[rstest]
fn full_round_trip_via_action(
    mut doc_with_two_shapes: Document,
    selection_with_first_shape: Selection,
) {
    let original = doc_with_two_shapes.clone();

    // Prepare command from action
    let cmd = prepare_command(
        Action::DeleteSelection,
        &doc_with_two_shapes,
        &selection_with_first_shape,
    )
    .expect("prepare succeeded");

    // Apply command
    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply succeeded");
    assert_eq!(doc_with_two_shapes.shapes.len(), 1);

    // Undo via inverse
    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("undo succeeded");
    assert_eq!(doc_with_two_shapes, original);
}

#[rstest]
fn selection_only_anchors_returns_empty_selection_error(doc_with_two_shapes: Document) {
    let mut selection = Selection::default();
    // Select only an anchor, not the whole shape
    selection.toggle(SelItem::Anchor {
        shape: shape_id(1),
        anchor: 0,
    });

    let result = prepare_command(Action::DeleteSelection, &doc_with_two_shapes, &selection);

    assert!(matches!(result, Err(UserError::EmptySelection)));
}

#[test]
fn user_error_display_empty_selection() {
    let err = UserError::EmptySelection;
    let msg = format!("{err}");
    assert_eq!(msg, "No selection");
}

#[test]
fn user_error_display_shape_not_found() {
    let err = UserError::ShapeNotFound(shape_id(42));
    let msg = format!("{err}");
    assert_eq!(msg, "Shape not found");
}

/// Test that editor actions panic when passed to `prepare_command`.
///
/// This is a dispatcher bug: editor actions should be routed directly, not
/// via `prepare_command`. Each test uses `#[should_panic]` to verify the
/// panic message contains "dispatcher bug".
mod editor_action_panics {
    use super::*;

    #[test]
    #[should_panic(expected = "dispatcher bug")]
    fn select_all_panics() {
        let doc = Document::default();
        let selection = Selection::default();
        drop(prepare_command(Action::SelectAll, &doc, &selection));
    }

    #[test]
    #[should_panic(expected = "dispatcher bug")]
    fn deselect_all_panics() {
        let doc = Document::default();
        let selection = Selection::default();
        drop(prepare_command(Action::DeselectAll, &doc, &selection));
    }

    #[test]
    #[should_panic(expected = "dispatcher bug")]
    fn undo_panics() {
        let doc = Document::default();
        let selection = Selection::default();
        drop(prepare_command(Action::Undo, &doc, &selection));
    }

    #[test]
    #[should_panic(expected = "dispatcher bug")]
    fn redo_panics() {
        let doc = Document::default();
        let selection = Selection::default();
        drop(prepare_command(Action::Redo, &doc, &selection));
    }

    #[test]
    #[should_panic(expected = "dispatcher bug")]
    fn activate_pen_tool_panics() {
        let doc = Document::default();
        let selection = Selection::default();
        drop(prepare_command(Action::ActivatePenTool, &doc, &selection));
    }

    #[test]
    #[should_panic(expected = "dispatcher bug")]
    fn activate_select_tool_panics() {
        let doc = Document::default();
        let selection = Selection::default();
        drop(prepare_command(
            Action::ActivateSelectTool,
            &doc,
            &selection,
        ));
    }
}
