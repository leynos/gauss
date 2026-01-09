//! Error tests for anchor operations.
//!
//! These tests verify that anchor commands return appropriate errors when
//! operating on out-of-range indices.

use gauss::model::{
    AnchorDeletion, AnchorDeletionResult, AnchorRestoration, AnchorRestorationKind, Command,
    CommandInverse, Document, ShapeReplacement, UserError,
};
use rstest::rstest;
use test_support::shapes::{sample_shape, shape_id};

use super::{assert_command_error, empty_doc};

/// Verify `InsertAnchor` returns an error for out-of-range shape indices.
#[rstest]
fn insert_anchor_fails_for_invalid_shape_index(empty_doc: Document) {
    let cmd = Command::InsertAnchor {
        replacement: ShapeReplacement {
            shape_index: 999,
            old_shape: sample_shape(shape_id(1), 0),
            new_shape: sample_shape(shape_id(1), 0),
        },
    };

    assert_command_error(empty_doc, &cmd, |err| match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains("out of range"),
                "expected 'out of range' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Verify `DeleteAnchors` returns an error for out-of-range shape indices.
#[rstest]
fn delete_anchors_fails_for_invalid_shape_index(empty_doc: Document) {
    let cmd = Command::DeleteAnchors {
        deletions: vec![AnchorDeletion {
            shape_id: shape_id(999),
            shape_index: 999,
            old_shape: sample_shape(shape_id(999), 0),
            result: AnchorDeletionResult::Removed,
        }],
    };

    assert_command_error(empty_doc, &cmd, |err| match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains("out of range"),
                "expected 'out of range' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Verify `ClosePath` returns an error for out-of-range shape indices.
#[rstest]
fn close_path_fails_for_invalid_shape_index(empty_doc: Document) {
    let cmd = Command::ClosePath {
        replacement: ShapeReplacement {
            shape_index: 999,
            old_shape: sample_shape(shape_id(1), 0),
            new_shape: sample_shape(shape_id(1), 0),
        },
    };

    assert_command_error(empty_doc, &cmd, |err| match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains("out of range"),
                "expected 'out of range' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Verify `RestoreAnchors` (via inverse) returns an error for invalid indices.
#[rstest]
fn restore_anchors_fails_for_invalid_shape_index(empty_doc: Document) {
    let inverse = CommandInverse::RestoreAnchors {
        command_name: "Delete Anchors",
        restorations: vec![AnchorRestoration {
            shape_id: shape_id(999),
            shape_index: 999,
            restoration: AnchorRestorationKind::RestoreRemoved {
                shape: sample_shape(shape_id(999), 0),
            },
        }],
    };

    let Err(err) = inverse.apply(&mut empty_doc.clone()) else {
        panic!("expected error");
    };
    match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains("out of range"),
                "expected 'out of range' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

/// Verify `ReopenPath` (via inverse) returns an error for out-of-range indices.
#[rstest]
fn reopen_path_fails_for_invalid_shape_index(empty_doc: Document) {
    let inverse = CommandInverse::ReopenPath {
        command_name: "Close Path",
        replacement: ShapeReplacement {
            shape_index: 999,
            old_shape: sample_shape(shape_id(1), 0),
            new_shape: sample_shape(shape_id(1), 0),
        },
    };

    let Err(err) = inverse.apply(&mut empty_doc.clone()) else {
        panic!("expected error");
    };
    match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains("out of range"),
                "expected 'out of range' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
