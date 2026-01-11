//! Unit tests for Command dispatch.
//!
//! These tests validate that commands correctly apply and undo document
//! mutations. Tests are organised into submodules by category.

mod anchor_error_tests;
mod delete_tests;
mod error_tests;
mod insert_tests;
mod inverse_error_tests;
mod success_tests;

use std::any::Any;

use gauss::model::{Command, CommandInverse, Document, SelItem, Selection, ShapeId, UserError};
use rstest::fixture;
use test_support::shapes::{sample_shape, shape_id};

#[fixture]
pub fn empty_doc() -> Document {
    Document::default()
}

#[fixture]
pub fn doc_with_two_shapes() -> Document {
    Document {
        shapes: vec![sample_shape(shape_id(1), 0), sample_shape(shape_id(2), 1)],
    }
}

#[fixture]
pub fn selection_with_first_shape() -> Selection {
    let mut selection = Selection::default();
    selection.toggle(SelItem::Shape(shape_id(1)));
    selection
}

/// Helper function to assert that a command returns a specific error and validate its fields.
/// This reduces duplication across error-condition tests whilst preserving field-level assertions.
pub fn assert_command_error<F>(mut doc: Document, cmd: &Command, error_validator: F)
where
    F: FnOnce(UserError),
{
    let Err(err) = cmd.apply(&mut doc) else {
        panic!("expected command {cmd:?} to fail");
    };
    error_validator(err);
}

/// Helper function to assert that an inverse returns a specific error and validate its fields.
/// This reduces duplication across inverse error-condition tests whilst preserving field-level assertions.
pub fn assert_inverse_error<F>(mut doc: Document, inverse: &CommandInverse, error_validator: F)
where
    F: FnOnce(UserError),
{
    let Err(err) = inverse.apply(&mut doc) else {
        panic!("expected inverse {inverse:?} to fail");
    };
    error_validator(err);
}

/// Assert that a command fails with `ShapeNotFound` for a specific shape ID.
pub fn assert_fails_with_shape_not_found(doc: Document, cmd: &Command, expected_id: ShapeId) {
    assert_command_error(doc, cmd, |err| match err {
        UserError::ShapeNotFound(id) => assert_eq!(id, expected_id),
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Assert that an inverse fails with `ShapeNotFound` for a specific shape ID.
pub fn assert_inverse_fails_with_shape_not_found(
    doc: Document,
    inverse: &CommandInverse,
    expected_id: ShapeId,
) {
    assert_inverse_error(doc, inverse, |err| match err {
        UserError::ShapeNotFound(id) => assert_eq!(id, expected_id),
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Assert that a command fails with `AnchorNotFound` for a specific shape and anchor index.
pub fn assert_fails_with_anchor_not_found(
    doc: Document,
    cmd: &Command,
    expected_shape_id: ShapeId,
    expected_anchor_idx: usize,
) {
    assert_command_error(doc, cmd, |err| match err {
        UserError::AnchorNotFound(sid, idx) => {
            assert_eq!(sid, expected_shape_id);
            assert_eq!(idx, expected_anchor_idx);
        }
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Assert that a command fails with `SegmentNotFound` for a specific shape and segment index.
pub fn assert_fails_with_segment_not_found(
    doc: Document,
    cmd: &Command,
    expected_shape_id: ShapeId,
    expected_segment_idx: usize,
) {
    assert_command_error(doc, cmd, |err| match err {
        UserError::SegmentNotFound(sid, idx) => {
            assert_eq!(sid, expected_shape_id);
            assert_eq!(idx, expected_segment_idx);
        }
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Assert that a command fails with `InvalidOperation` containing a specific message fragment.
pub fn assert_fails_with_invalid_operation(
    doc: Document,
    cmd: &Command,
    expected_message_fragment: &str,
) {
    assert_command_error(doc, cmd, |err| match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains(expected_message_fragment),
                "expected '{expected_message_fragment}' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Assert that an inverse fails with `InvalidOperation` containing a specific message fragment.
pub fn assert_inverse_fails_with_invalid_operation(
    doc: Document,
    inverse: &CommandInverse,
    expected_message_fragment: &str,
) {
    assert_inverse_error(doc, inverse, |err| match err {
        UserError::InvalidOperation(msg) => {
            assert!(
                msg.contains(expected_message_fragment),
                "expected '{expected_message_fragment}' in message: {msg}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    });
}

/// Extract a panic message from a panic payload.
///
/// Downcasts the payload to either `&str` or `String` and returns the message.
/// Returns a placeholder string for unrecognised payload types.
pub fn extract_panic_message(payload: &Box<dyn Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<unknown panic payload>".to_owned())
}
