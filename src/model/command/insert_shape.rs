//! Insert shape command implementation.

use crate::model::Document;

use super::CommandInverse;
use super::types::ShapeInsertion;

/// Apply the `InsertShape` command, returning the inverse for undo.
pub fn apply_insert_shape(
    doc: &mut Document,
    insertion: &ShapeInsertion,
    command_name: &'static str,
) -> CommandInverse {
    debug_assert!(
        insertion.index <= doc.shapes.len(),
        "apply_insert_shape: insertion index {} out of range (len = {}), \
         this likely indicates a logic bug",
        insertion.index,
        doc.shapes.len()
    );

    if insertion.index <= doc.shapes.len() {
        doc.shapes.insert(insertion.index, insertion.shape.clone());
    }

    CommandInverse::RemoveShape {
        command_name,
        insertion: insertion.clone(),
    }
}

/// Apply the `RemoveShape` inverse command.
pub fn apply_remove_shape(doc: &mut Document, insertion: &ShapeInsertion) {
    debug_assert!(
        insertion.index < doc.shapes.len(),
        "apply_remove_shape: insertion index {} out of range (len = {}), \
         this likely indicates a logic bug",
        insertion.index,
        doc.shapes.len()
    );

    if insertion.index < doc.shapes.len() {
        doc.shapes.remove(insertion.index);
    }
}
