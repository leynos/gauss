//! Helpers for command editing unit tests.
//!
//! These helpers assert shape replacement behaviour with undo/redo and verify
//! that prepared commands match expected variants.

use std::sync::Arc;

use gauss_core::model::{
    Action,
    Command,
    CommandInverse,
    Document,
    EngineState,
    SelItem,
    Shape,
    ShapeId,
    ShapeReplacement,
    UserError,
    prepare_command,
};

use crate::command_editing_helpers::{shape_at, shape_with_handles};

#[derive(Debug, thiserror::Error)]
pub(super) enum CommandEditingTestError {
    #[error("apply failed")]
    Apply(#[source] UserError),
    #[error("undo failed")]
    Undo(#[source] UserError),
    #[error("prepare failed")]
    Prepare(#[source] UserError),
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
    // `thiserror` needs a literal format string, so the message is built by an
    // inner `format!("{}")` whose text can stay split across `concat!` fragments.
    #[error(
        "{}",
        format!(
            concat!(
                "unexpected anchor/segment counts: expected anchors {expected_anchors}, ",
                "segments {expected_segments}, got anchors {actual_anchors}, ",
                "segments {actual_segments}"
            ),
            expected_anchors = expected_anchors,
            expected_segments = expected_segments,
            actual_anchors = actual_anchors,
            actual_segments = actual_segments,
        )
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
    old_shape: Shape,
    new_shape: Shape,
    create_command: impl Fn(ShapeReplacement) -> Command,
) -> Result<(), CommandEditingTestError> {
    let expected_old = old_shape.clone();
    let expected_new = new_shape.clone();
    let mut doc = Document::new();
    doc.append_shape(old_shape.clone());
    let cmd = create_command(ShapeReplacement {
        shape_index,
        old_shape,
        new_shape,
    });

    let inverse = apply_and_require_shape(&mut doc, &cmd, &expected_new)?;
    undo_and_require_shape(&mut doc, &inverse, &expected_old)
}

/// Applies `command` and requires the document to hold `expected`.
fn apply_and_require_shape(
    doc: &mut Document,
    command: &Command,
    expected: &Shape,
) -> Result<CommandInverse, CommandEditingTestError> {
    let inverse = command.apply(doc).map_err(CommandEditingTestError::Apply)?;
    require_first_shape(doc, expected, ShapeMismatch::NotUpdated)?;
    Ok(inverse)
}

/// Applies `inverse` and requires the document to hold `expected` again.
fn undo_and_require_shape(
    doc: &mut Document,
    inverse: &CommandInverse,
    expected: &Shape,
) -> Result<(), CommandEditingTestError> {
    inverse.apply(doc).map_err(CommandEditingTestError::Undo)?;
    require_first_shape(doc, expected, ShapeMismatch::NotRestored)
}

/// Which comparison failed, so the caller reports the right error.
#[derive(Clone, Copy)]
enum ShapeMismatch {
    /// The command did not install the expected shape.
    NotUpdated,
    /// The inverse did not restore the original shape.
    NotRestored,
}

/// Requires the first shape to equal `expected`, reporting via `mismatch`.
fn require_first_shape(
    doc: &Document,
    expected: &Shape,
    mismatch: ShapeMismatch,
) -> Result<(), CommandEditingTestError> {
    let actual = shape_at(doc, 0).ok_or(CommandEditingTestError::ShapeMissing)?;
    if actual == expected {
        return Ok(());
    }
    let expected_shape = Arc::new(expected.clone());
    let actual_shape = Arc::new(actual.clone());
    Err(match mismatch {
        ShapeMismatch::NotUpdated => CommandEditingTestError::ShapeNotUpdated {
            expected: expected_shape,
            actual: actual_shape,
        },
        ShapeMismatch::NotRestored => CommandEditingTestError::ShapeNotRestored {
            expected: expected_shape,
            actual: actual_shape,
        },
    })
}

pub(super) fn assert_prepare_command_returns_variant(
    shape_id: ShapeId,
    selection_item: SelItem,
    action: Action,
    matches_pattern: impl Fn(&Command) -> bool,
) -> Result<(), CommandEditingTestError> {
    let shape = shape_with_handles(shape_id);
    let mut doc = Document::new();
    doc.append_shape(shape);
    let mut state = EngineState::with_document(doc);
    state.selection.items = vec![selection_item];

    let cmd = prepare_command(action, &state).map_err(CommandEditingTestError::Prepare)?;
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
    let cmd = prepare_command(Action::InsertAnchorOnSegment, state)
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
