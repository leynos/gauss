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

use gauss::model::{Command, Document, SelItem, Selection, UserError};
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
        panic!("expected command to fail");
    };
    error_validator(err);
}
