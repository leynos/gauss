//! Unit tests for command-based editing operations.
//!
//! These tests validate command apply and inverse behaviour for the Phase 0
//! migration to Commands.
use gauss::model::{
    Action, Anchor, AnchorDeletion, AnchorDeletionResult, AnchorMovement, Command, Document,
    EngineState, HandleKind, HandleMovement, PaintStyle, ReorderOp, SegmentChange, SegmentKind,
    SelItem, ShapeMovement, StyleChange, Vec2, prepare_command,
};
use rstest::rstest;
use test_support::shapes::shape_id;
mod command_editing_helpers;
mod command_editing_unit_helpers;

use command_editing_helpers::{
    anchor_at, segment_at, segment_mut, shape_at, shape_with_handles, shape_with_three_anchors,
};
use command_editing_unit_helpers::{
    ExpectedCommand, assert_prepare_command_returns_variant,
    assert_shape_replacement_applies_and_undoes,
};

#[test]
fn move_shapes_applies_and_undoes() {
    let shape = shape_with_handles(shape_id(1));
    let original = shape.clone();
    let mut doc = Document {
        shapes: vec![shape],
    };

    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: shape_id(1),
            delta: Vec2::new(5.0, -2.0),
        }],
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    let moved = shape_at(&doc, 0).expect("shape exists");
    let moved_anchor = anchor_at(moved, 0).expect("anchor exists");
    assert_eq!(moved_anchor.pos, Vec2::new(5.0, -2.0));
    assert_eq!(moved_anchor.handle_out, Some(Vec2::new(6.0, -1.0)));

    inverse.apply(&mut doc).expect("undo succeeded");
    assert_eq!(shape_at(&doc, 0).expect("shape exists"), &original);
}
#[test]
fn move_anchor_translates_handles_and_undoes() {
    let shape = shape_with_handles(shape_id(2));
    let original_anchor = anchor_at(&shape, 0).expect("anchor exists").clone();
    let mut doc = Document {
        shapes: vec![shape],
    };

    let cmd = Command::MoveAnchor {
        movement: AnchorMovement {
            shape_id: shape_id(2),
            anchor_index: 0,
            original: original_anchor.clone(),
            delta: Vec2::new(3.0, 4.0),
        },
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    let moved = anchor_at(shape_at(&doc, 0).expect("shape exists"), 0).expect("anchor exists");
    assert_eq!(moved.pos, Vec2::new(3.0, 4.0));
    assert_eq!(moved.handle_in, Some(Vec2::new(2.0, 3.0)));
    assert_eq!(moved.handle_out, Some(Vec2::new(4.0, 5.0)));

    inverse.apply(&mut doc).expect("undo succeeded");
    assert_eq!(
        anchor_at(shape_at(&doc, 0).expect("shape exists"), 0).expect("anchor exists"),
        &original_anchor
    );
}
#[test]
fn move_handle_updates_and_undoes() {
    let shape = shape_with_handles(shape_id(3));
    let original = anchor_at(&shape, 1).expect("anchor exists").handle_in;
    let mut doc = Document {
        shapes: vec![shape],
    };

    let cmd = Command::MoveHandle {
        movement: HandleMovement {
            shape_id: shape_id(3),
            anchor_index: 1,
            kind: HandleKind::In,
            from: original,
            to: Some(Vec2::new(8.0, -2.0)),
        },
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    assert_eq!(
        anchor_at(shape_at(&doc, 0).expect("shape exists"), 1)
            .expect("anchor exists")
            .handle_in,
        Some(Vec2::new(8.0, -2.0))
    );

    inverse.apply(&mut doc).expect("undo succeeded");
    assert_eq!(
        anchor_at(shape_at(&doc, 0).expect("shape exists"), 1)
            .expect("anchor exists")
            .handle_in,
        original
    );
}
#[test]
fn set_style_updates_and_undoes() {
    let mut doc = Document {
        shapes: vec![shape_with_handles(shape_id(4))],
    };
    let original = shape_at(&doc, 0).expect("shape exists").style.clone();
    let updated = PaintStyle::new(None, 4.0, None);

    let cmd = Command::SetStyle {
        changes: vec![StyleChange {
            shape_id: shape_id(4),
            from: original.clone(),
            to: updated.clone(),
        }],
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    assert_eq!(shape_at(&doc, 0).expect("shape exists").style, updated);

    inverse.apply(&mut doc).expect("undo succeeded");
    assert_eq!(shape_at(&doc, 0).expect("shape exists").style, original);
}
#[test]
fn reorder_moves_and_undoes() {
    let shape_a = shape_with_handles(shape_id(5));
    let shape_b = shape_with_handles(shape_id(6));
    let mut doc = Document {
        shapes: vec![shape_a.clone(), shape_b.clone()],
    };

    let cmd = Command::Reorder {
        operations: vec![ReorderOp {
            shape_id: shape_id(5),
            from_index: 0,
            to_index: 1,
        }],
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    assert_eq!(shape_at(&doc, 0).expect("shape exists").id, shape_id(6));
    assert_eq!(shape_at(&doc, 1).expect("shape exists").id, shape_id(5));

    inverse.apply(&mut doc).expect("undo succeeded");
    assert_eq!(shape_at(&doc, 0).expect("shape exists"), &shape_a);
    assert_eq!(shape_at(&doc, 1).expect("shape exists"), &shape_b);
}
#[test]
fn set_segment_kind_updates_handles_and_undoes() {
    let mut shape = shape_with_handles(shape_id(7));
    *segment_mut(&mut shape, 0).expect("segment exists") = SegmentKind::Line;
    let mut doc = Document {
        shapes: vec![shape],
    };

    let cmd = Command::SetSegmentKind {
        changes: vec![SegmentChange {
            shape_id: shape_id(7),
            segment_index: 0,
            old_kind: SegmentKind::Line,
            new_kind: SegmentKind::Cubic,
            old_start_handle_out: None,
            new_start_handle_out: Some(Vec2::new(2.0, 0.0)),
            old_end_handle_in: None,
            new_end_handle_in: Some(Vec2::new(8.0, 0.0)),
        }],
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    let updated = shape_at(&doc, 0).expect("shape exists");
    assert_eq!(
        segment_at(updated, 0).expect("segment exists"),
        &SegmentKind::Cubic
    );
    assert_eq!(
        anchor_at(updated, 0).expect("anchor exists").handle_out,
        Some(Vec2::new(2.0, 0.0))
    );
    assert_eq!(
        anchor_at(updated, 1).expect("anchor exists").handle_in,
        Some(Vec2::new(8.0, 0.0))
    );

    inverse.apply(&mut doc).expect("undo succeeded");
    let restored = shape_at(&doc, 0).expect("shape exists");
    assert_eq!(
        segment_at(restored, 0).expect("segment exists"),
        &SegmentKind::Line
    );
    assert_eq!(
        anchor_at(restored, 0).expect("anchor exists").handle_out,
        None
    );
    assert_eq!(
        anchor_at(restored, 1).expect("anchor exists").handle_in,
        None
    );
}
#[test]
fn insert_anchor_replaces_shape_and_undoes() -> Result<(), String> {
    let old_shape = shape_with_handles(shape_id(8));
    let mut new_shape = old_shape.clone();
    new_shape.path.anchors.insert(
        1,
        Anchor {
            pos: Vec2::new(5.0, 0.0),
            handle_in: None,
            handle_out: None,
        },
    );
    new_shape.path.segments.insert(1, SegmentKind::Line);

    assert_shape_replacement_applies_and_undoes(0, old_shape, new_shape, |replacement| {
        Command::InsertAnchor { replacement }
    })?;

    Ok(())
}
#[test]
fn delete_anchors_preserves_handles_and_undoes() {
    let old_shape = shape_with_three_anchors(shape_id(9));
    let mut doc = Document {
        shapes: vec![old_shape.clone()],
    };

    let cmd = Command::DeleteAnchors {
        deletions: vec![AnchorDeletion {
            shape_id: shape_id(9),
            shape_index: 0,
            old_shape: old_shape.clone(),
            result: AnchorDeletionResult::Modified({
                let mut modified = old_shape.clone();
                modified.path.anchors.remove(1);
                modified.path.segments.remove(1);
                modified
            }),
        }],
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    let updated = shape_at(&doc, 0).expect("shape exists");
    assert_eq!(updated.path.anchors.len(), 2);
    assert_eq!(updated.path.segments.len(), 1);
    assert_eq!(
        segment_at(updated, 0).expect("segment exists"),
        &SegmentKind::Cubic
    );
    assert_eq!(
        anchor_at(updated, 0).expect("anchor exists").handle_out,
        Some(Vec2::new(2.0, 0.0))
    );
    assert_eq!(
        anchor_at(updated, 1).expect("anchor exists").handle_in,
        Some(Vec2::new(19.0, 0.0))
    );

    inverse.apply(&mut doc).expect("undo succeeded");
    assert_eq!(shape_at(&doc, 0).expect("shape exists"), &old_shape);
}
#[test]
fn close_path_replaces_shape_and_undoes() -> Result<(), String> {
    let open_shape = shape_with_handles(shape_id(10));
    let mut closed_shape = open_shape.clone();
    closed_shape.path.closed = true;
    closed_shape.path.closing_segment = SegmentKind::Cubic;

    assert_shape_replacement_applies_and_undoes(0, open_shape, closed_shape, |replacement| {
        Command::ClosePath { replacement }
    })?;

    Ok(())
}
#[rstest]
#[case(
    shape_id(11),
    SelItem::Segment { shape: shape_id(11), seg: 0 },
    Action::InsertAnchorOnSegment,
    ExpectedCommand::InsertAnchor
)]
#[case(
    shape_id(12),
    SelItem::Anchor { shape: shape_id(12), anchor: 0 },
    Action::DeleteSelectedAnchors,
    ExpectedCommand::DeleteAnchors
)]
fn prepare_command_returns_expected_variant(
    #[case] shape_id_value: gauss::model::ShapeId,
    #[case] selection_item: SelItem,
    #[case] action: Action,
    #[case] expected: ExpectedCommand,
) -> Result<(), String> {
    assert_prepare_command_returns_variant(shape_id_value, selection_item, action, |cmd| {
        expected.matches(cmd)
    })?;

    Ok(())
}
#[test]
fn prepare_raise_selection_uses_anchor_selection() {
    let shape_a = shape_with_handles(shape_id(13));
    let shape_b = shape_with_handles(shape_id(14));
    let mut state = EngineState::with_document(Document {
        shapes: vec![shape_a, shape_b],
    });
    state.selection.items = vec![SelItem::Anchor {
        shape: shape_id(13),
        anchor: 0,
    }];

    let cmd = prepare_command(Action::RaiseSelection, &state).expect("prepare succeeded");
    let Command::Reorder { operations } = cmd else {
        panic!("expected Reorder command");
    };
    assert_eq!(operations.len(), 1);
    let operation = operations.first().expect("reorder op exists");
    assert_eq!(operation.from_index, 0);
    assert_eq!(operation.to_index, 1);
}
#[test]
fn prepare_toggle_segment_kind_requires_segment() {
    let shape = shape_with_handles(shape_id(15));
    let mut state = EngineState::with_document(Document {
        shapes: vec![shape],
    });
    state.selection.items = vec![SelItem::Shape(shape_id(15))];

    let result = prepare_command(Action::ToggleSegmentKind, &state);
    assert!(result.is_err());
}
#[test]
fn command_inverse_name_matches_after_apply() {
    let mut doc = Document {
        shapes: vec![shape_with_handles(shape_id(16))],
    };
    let cmd = Command::MoveShapes {
        movements: vec![ShapeMovement {
            shape_id: shape_id(16),
            delta: Vec2::new(1.0, 1.0),
        }],
    };

    let inverse = cmd.apply(&mut doc).expect("apply succeeded");
    assert_eq!(cmd.name(), inverse.name());
}
