//! Tests for insert shape commands.

use gauss::model::{Command, Document, ShapeInsertion};
use rstest::rstest;
use test_support::shapes::{sample_shape, shape_id};

use super::{doc_with_two_shapes, empty_doc};

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
    assert_eq!(doc.len(), expected_len);
    assert_eq!(
        doc.shape_at(insert_index)
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
    assert_eq!(empty_doc.len(), 1);

    inverse.apply(&mut empty_doc).expect("undo succeeded");
    assert!(empty_doc.is_empty());
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
    assert_eq!(doc_with_two_shapes.len(), 3);
    assert_eq!(
        doc_with_two_shapes
            .shape_at(1)
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
