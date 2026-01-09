//! Unit tests for Command dispatch.
//!
//! These tests validate that commands correctly apply and undo document
//! mutations.

use gauss::model::{
    Action, Anchor, AnchorMovement, Command, CommandInverse, DeletedShape, Document, EngineState,
    HandleKind, HandleMovement, PaintStyle, ReorderOp, SegmentChange, SegmentKind, SelItem,
    Selection, ShapeInsertion, ShapeMovement, StyleChange, UserError, Vec2, prepare_command,
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
    let mut state = EngineState::with_document(empty_doc);
    state.selection.toggle(SelItem::Shape(shape_id(999)));

    let result = prepare_command(Action::DeleteSelection, &state);

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
/// via `prepare_command`. Uses `#[rstest]` parameterisation to verify the
/// panic message contains "dispatcher bug" for all editor action variants.
#[rstest]
#[case::select_all(Action::SelectAll)]
#[case::deselect_all(Action::DeselectAll)]
#[case::undo(Action::Undo)]
#[case::redo(Action::Redo)]
#[case::activate_pen_tool(Action::ActivatePenTool)]
#[case::activate_select_tool(Action::ActivateSelectTool)]
#[should_panic(expected = "dispatcher bug")]
fn editor_action_panics(#[case] action: Action) {
    let state = EngineState::new();
    drop(prepare_command(action, &state));
}

// ============================================================================
// InsertShape Command Tests
// ============================================================================

/// Verify `InsertShape` inserts at the correct position across different scenarios.
#[rstest]
#[case::empty_at_zero(empty_doc(), 0, 1, shape_id(100))]
#[case::at_end(doc_with_two_shapes(), 2, 3, shape_id(100))]
#[case::at_beginning(doc_with_two_shapes(), 0, 3, shape_id(100))]
fn insert_shape_at_position(
    #[case] mut doc: Document,
    #[case] insert_index: usize,
    #[case] expected_len: usize,
    #[case] expected_id: gauss::model::ShapeId,
) {
    let cmd = Command::InsertShape {
        insertion: ShapeInsertion {
            index: insert_index,
            shape: sample_shape(expected_id, 0),
        },
    };

    let result = cmd.apply(&mut doc);
    assert!(result.is_ok());
    assert_eq!(doc.shapes.len(), expected_len);
    assert_eq!(
        doc.shapes
            .get(insert_index)
            .expect("should have shape at insert index")
            .id,
        expected_id
    );
}

#[rstest]
fn insert_shape_inverse_removes(mut empty_doc: Document) {
    let cmd = Command::InsertShape {
        insertion: ShapeInsertion {
            index: 0,
            shape: sample_shape(shape_id(1), 0),
        },
    };

    let inverse = cmd.apply(&mut empty_doc).expect("apply succeeded");
    assert_eq!(empty_doc.shapes.len(), 1);

    inverse.apply(&mut empty_doc).expect("undo succeeded");
    assert!(empty_doc.shapes.is_empty());
}

#[rstest]
fn insert_shape_full_round_trip(mut doc_with_two_shapes: Document) {
    let original = doc_with_two_shapes.clone();
    let shape = sample_shape(shape_id(100), 2);
    let cmd = Command::InsertShape {
        insertion: ShapeInsertion {
            index: 1,
            shape: shape.clone(),
        },
    };

    // Apply: insert shape at index 1
    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("apply succeeded");
    assert_eq!(doc_with_two_shapes.shapes.len(), 3);
    assert_eq!(
        doc_with_two_shapes
            .shapes
            .get(1)
            .expect("should have shape at index 1")
            .id,
        shape.id
    );

    // Undo: remove the inserted shape
    inverse
        .apply(&mut doc_with_two_shapes)
        .expect("undo succeeded");
    assert_eq!(doc_with_two_shapes, original);
}

#[rstest]
fn insert_shape_name_is_correct() {
    let shape = sample_shape(shape_id(1), 0);
    let cmd = Command::InsertShape {
        insertion: ShapeInsertion { index: 0, shape },
    };
    assert_eq!(cmd.name(), "Insert Shape");
}

#[rstest]
fn insert_shape_inverse_name_matches_command(mut empty_doc: Document) {
    let shape = sample_shape(shape_id(1), 0);
    let cmd = Command::InsertShape {
        insertion: ShapeInsertion { index: 0, shape },
    };

    let inverse = cmd.apply(&mut empty_doc).expect("apply succeeded");
    assert_eq!(inverse.name(), cmd.name());
    assert_eq!(inverse.name(), "Insert Shape");
}

// ============================================================================
// Error Condition Tests (Issue #28)
// ============================================================================

/// Helper function to assert that a command returns a specific error variant.
/// This reduces duplication across error-condition tests.
fn assert_command_fails_with<F>(mut doc: Document, cmd: &Command, error_matcher: F)
where
    F: FnOnce(&Result<CommandInverse, UserError>) -> bool,
{
    let result = cmd.apply(&mut doc);
    assert!(
        error_matcher(&result),
        "Expected command to fail with specific error, got: {result:?}",
    );
}

/// Verify `MoveShapes` returns an error for non-existent shapes.
#[rstest]
fn move_shapes_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: missing_id,
            delta: Vec2::new(10.0, 10.0),
        }],
    };

    assert_command_fails_with(
        empty_doc,
        &cmd,
        |r| matches!(r, Err(UserError::ShapeNotFound(id)) if *id == missing_id),
    );
}

/// Verify `MoveAnchor` returns an error for non-existent shapes.
#[rstest]
fn move_anchor_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: missing_id,
            anchor_index: 0,
            original: Anchor {
                pos: Vec2::new(0.0, 0.0),
                handle_in: None,
                handle_out: None,
            },
            delta: Vec2::new(10.0, 10.0),
        },
    };

    assert_command_fails_with(
        empty_doc,
        &cmd,
        |r| matches!(r, Err(UserError::ShapeNotFound(id)) if *id == missing_id),
    );
}

/// Verify `MoveAnchor` returns an error for non-existent anchors.
#[rstest]
fn move_anchor_fails_for_missing_anchor(doc_with_two_shapes: Document) {
    let target_shape = shape_id(1);
    let missing_anchor = 999;
    let cmd = Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: target_shape,
            anchor_index: missing_anchor,
            original: Anchor {
                pos: Vec2::new(0.0, 0.0),
                handle_in: None,
                handle_out: None,
            },
            delta: Vec2::new(10.0, 10.0),
        },
    };

    assert_command_fails_with(
        doc_with_two_shapes,
        &cmd,
        |r| matches!(r, Err(UserError::AnchorNotFound(sid, idx)) if *sid == target_shape && *idx == missing_anchor),
    );
}

/// Verify `MoveHandle` returns an error for non-existent shapes.
#[rstest]
fn move_handle_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::MoveHandle {
        movement: HandleMovement {
            shape_id: missing_id,
            anchor_index: 0,
            kind: HandleKind::In,
            from: None,
            to: Some(Vec2::new(10.0, 10.0)),
        },
    };

    assert_command_fails_with(
        empty_doc,
        &cmd,
        |r| matches!(r, Err(UserError::ShapeNotFound(id)) if *id == missing_id),
    );
}

/// Verify `MoveHandle` returns an error for non-existent anchors.
#[rstest]
fn move_handle_fails_for_missing_anchor(doc_with_two_shapes: Document) {
    let target_shape = shape_id(1);
    let missing_anchor = 999;
    let cmd = Command::MoveHandle {
        movement: HandleMovement {
            shape_id: target_shape,
            anchor_index: missing_anchor,
            kind: HandleKind::In,
            from: None,
            to: Some(Vec2::new(10.0, 10.0)),
        },
    };

    assert_command_fails_with(
        doc_with_two_shapes,
        &cmd,
        |r| matches!(r, Err(UserError::AnchorNotFound(sid, idx)) if *sid == target_shape && *idx == missing_anchor),
    );
}

/// Verify `SetStyle` returns an error for non-existent shapes.
#[rstest]
fn set_style_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::SetStyle {
        changes: vec![StyleChange {
            shape_id: missing_id,
            from: PaintStyle::new(None, 1.0, None),
            to: PaintStyle::new(None, 2.0, None),
        }],
    };

    assert_command_fails_with(
        empty_doc,
        &cmd,
        |r| matches!(r, Err(UserError::ShapeNotFound(id)) if *id == missing_id),
    );
}

/// Verify `Reorder` returns an error for invalid indices.
#[rstest]
fn reorder_fails_for_invalid_indices(empty_doc: Document) {
    let cmd = Command::Reorder {
        operations: vec![ReorderOp {
            shape_id: shape_id(999),
            from_index: 0,
            to_index: 1,
        }],
    };

    assert_command_fails_with(
        empty_doc,
        &cmd,
        |r| matches!(r, Err(UserError::InvalidOperation(msg)) if msg.contains("invalid reorder")),
    );
}

/// Verify `SetSegmentKind` returns an error for non-existent shapes.
#[rstest]
fn set_segment_kind_fails_for_missing_shape(empty_doc: Document) {
    let missing_id = shape_id(999);
    let cmd = Command::SetSegmentKind {
        changes: vec![SegmentChange {
            shape_id: missing_id,
            segment_index: 0,
            old_kind: SegmentKind::Line,
            new_kind: SegmentKind::Cubic,
            old_start_handle_out: None,
            new_start_handle_out: Some(Vec2::new(1.0, 1.0)),
            old_end_handle_in: None,
            new_end_handle_in: Some(Vec2::new(2.0, 2.0)),
        }],
    };

    assert_command_fails_with(
        empty_doc,
        &cmd,
        |r| matches!(r, Err(UserError::ShapeNotFound(id)) if *id == missing_id),
    );
}

/// Verify `SetSegmentKind` returns an error for non-existent segments.
#[rstest]
fn set_segment_kind_fails_for_missing_segment(doc_with_two_shapes: Document) {
    let target_shape = shape_id(1);
    let missing_segment = 999;
    let cmd = Command::SetSegmentKind {
        changes: vec![SegmentChange {
            shape_id: target_shape,
            segment_index: missing_segment,
            old_kind: SegmentKind::Line,
            new_kind: SegmentKind::Cubic,
            old_start_handle_out: None,
            new_start_handle_out: Some(Vec2::new(1.0, 1.0)),
            old_end_handle_in: None,
            new_end_handle_in: Some(Vec2::new(2.0, 2.0)),
        }],
    };

    assert_command_fails_with(
        doc_with_two_shapes,
        &cmd,
        |r| matches!(r, Err(UserError::SegmentNotFound(sid, idx)) if *sid == target_shape && *idx == missing_segment),
    );
}

/// Verify that `CommandInverse::apply` propagates errors when the document
/// has been mutated into an invalid state after the original command applied.
///
/// This ensures undo paths fail loudly instead of corrupting document state.
#[rstest]
fn inverse_apply_fails_when_document_state_changes(mut doc_with_two_shapes: Document) {
    // Apply a valid MoveShapes command and capture the inverse.
    let target_id = shape_id(1);
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: target_id,
            delta: Vec2::new(10.0, 10.0),
        }],
    };

    let inverse = cmd
        .apply(&mut doc_with_two_shapes)
        .expect("command should apply successfully");

    // Mutate the document into an invalid state: remove the shape the inverse
    // expects to operate on.
    doc_with_two_shapes.shapes.retain(|s| s.id != target_id);

    // Applying the inverse should now fail instead of silently corrupting state.
    let err = inverse
        .apply(&mut doc_with_two_shapes)
        .expect_err("expected ShapeNotFound error");
    match err {
        UserError::ShapeNotFound(id) => assert_eq!(id, target_id),
        other => panic!("unexpected error: {other:?}"),
    }
}
