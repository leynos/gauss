//! Tests for delete shape commands.
//!
//! This module provides comprehensive test coverage for the `DeleteShapes` command
//! and its inverse `RestoreShapes` operation.
//!
//! ## Success Paths
//!
//! - Deleting a single shape via `DeleteShapes` command
//! - Bulk deletion of multiple shapes with correct index handling
//! - Delete via `prepare_command` with `DeleteSelection` action
//! - Full round-trip: action → command → apply → inverse → restore
//!
//! ## Inverse Operations
//!
//! - `RestoreShapes` inverse correctly restores deleted shapes at original indices
//! - Round-trip apply/undo preserves document state exactly
//! - Inverse name matches the originating command name
//!
//! ## Error Conditions
//!
//! - `EmptySelection` when no shapes are selected
//! - `ShapeNotFound` for missing shape IDs in selection
//! - `EmptySelection` when selection contains only anchors (not whole shapes)
//!
//! ## Display Formatting
//!
//! - Validates `UserError` display strings for `EmptySelection` and `ShapeNotFound`
//!
//! ## Edge Cases
//!
//! - Multiple deletions preserve correct index adjustments during restore
//! - `SelectionScope` filtering between `WholeShapes` and `Anchors`
//! - Editor actions (non-document actions) panic with "dispatcher bug" message
//!
//! ## Key Tests
//!
//! - `delete_shapes_removes_from_document`: Basic deletion functionality
//! - `delete_restores_shape_at_correct_index`: Index preservation on restore
//! - `prepare_delete_selection_fails_with_empty_selection`: Error on empty selection
//! - `editor_action_panics`: Verifies dispatcher routing assertions

use std::panic::{AssertUnwindSafe, catch_unwind};

use gauss::model::{
    Action, Command, CommandInverse, DeletedShape, Document, EngineState, SelItem, Selection,
    UserError, prepare_command,
};
use rstest::rstest;
use test_support::shapes::{sample_shape, shape_id};

use super::{doc_with_two_shapes, empty_doc, extract_panic_message, selection_with_first_shape};

#[rstest]
fn delete_shapes_removes_from_document(mut doc_with_two_shapes: Document) {
    let first_shape = doc_with_two_shapes
        .shapes
        .first()
        .cloned()
        .expect("fixture should have shapes");
    let second_shape_id = doc_with_two_shapes
        .shapes
        .get(1)
        .expect("fixture should have two shapes")
        .id;
    let cmd = Command::DeleteShapes {
        targets: vec![DeletedShape {
            index: 0,
            shape: first_shape.clone(),
        }],
    };

    cmd.apply(&mut doc_with_two_shapes)
        .expect("delete should succeed");
    assert_eq!(doc_with_two_shapes.shapes.len(), 1);
    assert_eq!(
        doc_with_two_shapes
            .shapes
            .first()
            .expect("should have remaining shape")
            .id,
        second_shape_id,
        "remaining shape should be the second shape"
    );
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
    let state = EngineState::with_document(doc_with_two_shapes);

    let result = prepare_command(Action::DeleteSelection, &state);

    assert!(matches!(result, Err(UserError::EmptySelection)));
}

#[rstest]
fn prepare_delete_selection_succeeds_with_selection(
    doc_with_two_shapes: Document,
    selection_with_first_shape: Selection,
) {
    let mut state = EngineState::with_document(doc_with_two_shapes);
    state.selection = selection_with_first_shape;

    let result = prepare_command(Action::DeleteSelection, &state);

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
    let missing_id = shape_id(999);
    let mut state = EngineState::with_document(empty_doc);
    state.selection.toggle(SelItem::Shape(missing_id));

    let result = prepare_command(Action::DeleteSelection, &state);

    let Err(UserError::ShapeNotFound(id)) = result else {
        panic!("expected ShapeNotFound error, got: {result:?}");
    };
    assert_eq!(
        id, missing_id,
        "error should reference the missing shape ID"
    );
}

/// Validate command and inverse naming conventions.
#[rstest]
#[case::command_name_is_nonempty("non-empty", true)]
#[case::command_name_is_delete("Delete", false)]
fn command_naming_validation(#[case] expected: &str, #[case] check_nonempty: bool) {
    let cmd = Command::DeleteShapes { targets: vec![] };
    if check_nonempty {
        assert!(!cmd.name().is_empty(), "command name should not be empty");
    } else {
        assert_eq!(cmd.name(), expected);
    }
}

/// Validate that `CommandInverse` preserves the command name.
#[rstest]
fn command_inverse_name_is_delete() {
    let inverse = CommandInverse::RestoreShapes {
        command_name: "Delete",
        targets: vec![],
    };
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
    doc_with_two_shapes: Document,
    selection_with_first_shape: Selection,
) {
    let original = doc_with_two_shapes.clone();
    let mut state = EngineState::with_document(doc_with_two_shapes);
    state.selection = selection_with_first_shape;

    // Prepare command from action
    let cmd = prepare_command(Action::DeleteSelection, &state).expect("prepare succeeded");

    // Apply command
    let inverse = cmd.apply(&mut state.document).expect("apply succeeded");
    assert_eq!(state.document.shapes.len(), 1);

    // Undo via inverse
    inverse.apply(&mut state.document).expect("undo succeeded");
    assert_eq!(state.document, original);
}

#[rstest]
fn selection_only_anchors_returns_empty_selection_error(doc_with_two_shapes: Document) {
    let mut state = EngineState::with_document(doc_with_two_shapes);
    // Select only an anchor, not the whole shape
    state.selection.toggle(SelItem::Anchor {
        shape: shape_id(1),
        anchor: 0,
    });

    let result = prepare_command(Action::DeleteSelection, &state);

    assert!(matches!(result, Err(UserError::EmptySelection)));
}

#[rstest]
fn user_error_display_empty_selection() {
    let err = UserError::EmptySelection;
    let msg = format!("{err}");
    assert_eq!(msg, "No selection");
}

#[rstest]
fn user_error_display_shape_not_found() {
    let err = UserError::ShapeNotFound(shape_id(42));
    let msg = format!("{err}");
    assert_eq!(msg, "Shape not found");
}

/// Test that editor actions panic when passed to `prepare_command`.
///
/// This is a dispatcher bug: editor actions should be routed directly, not
/// via `prepare_command`. Uses `#[rstest]` parameterisation to verify the
/// panic message contains "dispatcher bug" for all editor action variants.
#[rstest]
#[case::select_all(Action::SelectAll)]
#[case::deselect_all(Action::DeselectAll)]
#[case::undo(Action::Undo)]
#[case::redo(Action::Redo)]
#[case::activate_pen_tool(Action::ActivatePenTool)]
#[case::activate_select_tool(Action::ActivateSelectTool)]
fn editor_action_panics(#[case] action: Action) {
    let state = EngineState::new();

    let result = catch_unwind(AssertUnwindSafe(|| {
        drop(prepare_command(action, &state));
    }));

    let panic_payload = result.expect_err("expected prepare_command to panic");
    let message = extract_panic_message(&panic_payload);
    assert!(
        message.contains("dispatcher bug"),
        "expected panic message to contain 'dispatcher bug', got: {message}"
    );
}
