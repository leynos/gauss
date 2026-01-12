//! Helpers for command editing unit tests.
//!
//! These helpers assert shape replacement behaviour with undo/redo and verify
//! that prepared commands match expected variants.

use std::sync::Arc;

use gauss::model::{
    Action, Command, CommandInverse, Document, EngineState, SelItem, Shape, ShapeReplacement,
};

use crate::command_editing_helpers::{shape_at, shape_with_handles};

#[derive(Debug, thiserror::Error)]
pub(super) enum CommandEditingTestError {
    #[error("apply failed")]
    Apply(#[source] gauss::model::UserError),
    #[error("undo failed")]
    Undo(#[source] gauss::model::UserError),
    #[error("prepare failed")]
    Prepare(#[source] gauss::model::UserError),
    #[error("shape missing")]
    ShapeMissing,
    #[error("shape was not updated: expected {expected:?}, got {actual:?}")]
    ShapeNotUpdated {
        expected: Arc<Shape>,
        actual: Arc<Shape>,
    },
    #[error("shape was not restored: expected {expected:?}, got {actual:?}")]
    ShapeNotRestored {
        expected: Arc<Shape>,
        actual: Arc<Shape>,
    },
    #[error(
        "unexpected anchor/segment counts: expected anchors {expected_anchors}, segments {expected_segments}, got anchors {actual_anchors}, segments {actual_segments}"
    )]
    UnexpectedShapeCounts {
        expected_anchors: usize,
        expected_segments: usize,
        actual_anchors: usize,
        actual_segments: usize,
    },
    #[error("command did not match expected variant")]
    CommandMismatch,
}

pub(super) fn assert_shape_replacement_applies_and_undoes(
    shape_index: usize,
    old_shape: gauss::model::Shape,
    new_shape: gauss::model::Shape,
    create_command: impl Fn(ShapeReplacement) -> Command,
) -> Result<(), CommandEditingTestError> {
    let expected_old = old_shape.clone();
    let expected_new = new_shape.clone();
    let mut doc = Document {
        shapes: vec![expected_old.clone()],
    };
    let cmd = create_command(ShapeReplacement {
        shape_index,
        old_shape,
        new_shape,
    });

    let inverse = cmd
        .apply(&mut doc)
        .map_err(CommandEditingTestError::Apply)?;
    let updated = shape_at(&doc, 0).ok_or(CommandEditingTestError::ShapeMissing)?;
    if updated != &expected_new {
        return Err(CommandEditingTestError::ShapeNotUpdated {
            expected: Arc::new(expected_new.clone()),
            actual: Arc::new(updated.clone()),
        });
    }

    inverse
        .apply(&mut doc)
        .map_err(CommandEditingTestError::Undo)?;
    let restored = shape_at(&doc, 0).ok_or(CommandEditingTestError::ShapeMissing)?;
    if restored != &expected_old {
        return Err(CommandEditingTestError::ShapeNotRestored {
            expected: Arc::new(expected_old.clone()),
            actual: Arc::new(restored.clone()),
        });
    }

    Ok(())
}

pub(super) fn assert_prepare_command_returns_variant(
    shape_id: gauss::model::ShapeId,
    selection_item: SelItem,
    action: Action,
    matches_pattern: impl Fn(&Command) -> bool,
) -> Result<(), CommandEditingTestError> {
    let shape = shape_with_handles(shape_id);
    let mut state = EngineState::with_document(Document {
        shapes: vec![shape],
    });
    state.selection.items = vec![selection_item];

    let cmd =
        gauss::model::prepare_command(action, &state).map_err(CommandEditingTestError::Prepare)?;
    if !matches_pattern(&cmd) {
        return Err(CommandEditingTestError::CommandMismatch);
    }

    Ok(())
}

pub(super) fn assert_insert_anchor_on_segment_effect(
    cmd: &Command,
    doc: &mut Document,
) -> Result<(CommandInverse, Shape), CommandEditingTestError> {
    let inverse = cmd.apply(doc).map_err(CommandEditingTestError::Apply)?;
    if !matches!(inverse, CommandInverse::RemoveAnchorFromSegment { .. }) {
        return Err(CommandEditingTestError::CommandMismatch);
    }

    let updated = shape_at(doc, 0).ok_or(CommandEditingTestError::ShapeMissing)?;
    Ok((inverse, updated.clone()))
}

pub(super) fn assert_insert_anchor_on_segment_command(
    state: &EngineState,
) -> Result<Command, CommandEditingTestError> {
    let cmd = gauss::model::prepare_command(Action::InsertAnchorOnSegment, state)
        .map_err(CommandEditingTestError::Prepare)?;
    if !matches!(cmd, Command::InsertAnchorOnSegment { .. }) {
        return Err(CommandEditingTestError::CommandMismatch);
    }

    Ok(cmd)
}

pub(super) const fn assert_insert_anchor_on_segment_lengths(
    original_shape: &Shape,
    updated: &Shape,
) -> Result<(), CommandEditingTestError> {
    let expected_anchors = original_shape.path.anchors.len() + 1;
    let expected_segments = original_shape.path.segments.len() + 1;
    let actual_anchors = updated.path.anchors.len();
    let actual_segments = updated.path.segments.len();
    if expected_anchors != actual_anchors || expected_segments != actual_segments {
        return Err(CommandEditingTestError::UnexpectedShapeCounts {
            expected_anchors,
            expected_segments,
            actual_anchors,
            actual_segments,
        });
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ExpectedCommand {
    InsertAnchorOnSegment,
    DeleteAnchors,
}

impl ExpectedCommand {
    pub(super) const fn matches(self, cmd: &Command) -> bool {
        match self {
            Self::InsertAnchorOnSegment => matches!(cmd, Command::InsertAnchorOnSegment { .. }),
            Self::DeleteAnchors => matches!(cmd, Command::DeleteAnchors { .. }),
        }
    }
}
