//! Helpers for command editing unit tests.

use gauss::model::{Action, Command, Document, EngineState, SelItem, ShapeReplacement};

use crate::command_editing_helpers::{shape_at, shape_with_handles};

pub(super) fn assert_shape_replacement_applies_and_undoes(
    shape_index: usize,
    old_shape: gauss::model::Shape,
    new_shape: gauss::model::Shape,
    create_command: impl Fn(ShapeReplacement) -> Command,
) {
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

    let Ok(inverse) = cmd.apply(&mut doc) else {
        panic!("apply succeeded");
    };
    let Some(updated) = shape_at(&doc, 0) else {
        panic!("shape exists");
    };
    assert_eq!(updated, &expected_new, "shape was updated");

    let Ok(()) = inverse.apply(&mut doc) else {
        panic!("undo succeeded");
    };
    let Some(restored) = shape_at(&doc, 0) else {
        panic!("shape exists");
    };
    assert_eq!(restored, &expected_old, "shape was restored");
}

pub(super) fn assert_prepare_command_returns_variant(
    shape_id: gauss::model::ShapeId,
    selection_item: SelItem,
    action: Action,
    matches_pattern: impl Fn(&Command) -> bool,
) {
    let shape = shape_with_handles(shape_id);
    let mut state = EngineState::with_document(Document {
        shapes: vec![shape],
    });
    state.selection.items = vec![selection_item];

    let Ok(cmd) = gauss::model::prepare_command(action, &state) else {
        panic!("prepare succeeded");
    };
    assert!(matches_pattern(&cmd), "command matches expected variant");
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
