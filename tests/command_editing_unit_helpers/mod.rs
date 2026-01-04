//! Helpers for command editing unit tests.

use gauss::model::{Action, Command, Document, EngineState, SelItem, ShapeReplacement};

use crate::command_editing_helpers::{shape_at, shape_with_handles};

pub(super) fn assert_shape_replacement_applies_and_undoes(
    shape_index: usize,
    old_shape: gauss::model::Shape,
    new_shape: gauss::model::Shape,
    create_command: impl Fn(ShapeReplacement) -> Command,
) -> Result<(), String> {
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
        .map_err(|err| format!("apply failed: {err}"))?;
    let updated = shape_at(&doc, 0).ok_or_else(|| "shape missing".to_owned())?;
    if updated != &expected_new {
        return Err("shape was not updated".to_owned());
    }

    inverse
        .apply(&mut doc)
        .map_err(|err| format!("undo failed: {err}"))?;
    let restored = shape_at(&doc, 0).ok_or_else(|| "shape missing".to_owned())?;
    if restored != &expected_old {
        return Err("shape was not restored".to_owned());
    }

    Ok(())
}

pub(super) fn assert_prepare_command_returns_variant(
    shape_id: gauss::model::ShapeId,
    selection_item: SelItem,
    action: Action,
    matches_pattern: impl Fn(&Command) -> bool,
) -> Result<(), String> {
    let shape = shape_with_handles(shape_id);
    let mut state = EngineState::with_document(Document {
        shapes: vec![shape],
    });
    state.selection.items = vec![selection_item];

    let cmd = gauss::model::prepare_command(action, &state)
        .map_err(|err| format!("prepare failed: {err}"))?;
    if !matches_pattern(&cmd) {
        return Err("command did not match expected variant".to_owned());
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ExpectedCommand {
    InsertAnchor,
    DeleteAnchors,
}

impl ExpectedCommand {
    pub(super) const fn matches(self, cmd: &Command) -> bool {
        match self {
            Self::InsertAnchor => matches!(cmd, Command::InsertAnchor { .. }),
            Self::DeleteAnchors => matches!(cmd, Command::DeleteAnchors { .. }),
        }
    }
}
