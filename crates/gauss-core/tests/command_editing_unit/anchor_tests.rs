//! Anchor and path editing command tests.

use gauss_core::model::{
    Anchor,
    AnchorDeletion,
    AnchorDeletionResult,
    Command,
    EngineState,
    SegmentKind,
    SelItem,
    Vec2,
};
use test_support::shapes::shape_id;

use super::{
    command_editing_helpers::{
        anchor_at,
        segment_at,
        shape_at,
        shape_with_handles,
        shape_with_three_anchors,
    },
    command_editing_unit_helpers::{
        CommandEditingTestError,
        assert_insert_anchor_on_segment_command,
        assert_insert_anchor_on_segment_effect,
        assert_insert_anchor_on_segment_lengths,
        assert_shape_replacement_applies_and_undoes,
    },
    doc_with_shapes,
};

#[test]
fn insert_anchor_replaces_shape_and_undoes() -> Result<(), CommandEditingTestError> {
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
fn insert_anchor_on_segment_undo_redo() {
    let shape_id_value = shape_id(9);
    let original_shape = shape_with_handles(shape_id_value);
    let seed_doc = doc_with_shapes(vec![original_shape.clone()]);
    let mut state = EngineState::with_document(seed_doc.clone());
    let anchor_count = original_shape.path.anchors.len();
    let segment_count = original_shape.path.segments.len();
    assert_eq!(anchor_count, 2, "expected fixture to have two anchors");
    assert_eq!(segment_count, 1, "expected fixture to have one segment");
    state.selection.items = vec![SelItem::Segment {
        shape: shape_id_value,
        seg: 0,
    }];

    let cmd =
        assert_insert_anchor_on_segment_command(&state).expect("prepare insert anchor on segment");

    let mut doc = seed_doc;
    let (inverse, after_insert) = assert_insert_anchor_on_segment_effect(&cmd, &mut doc)
        .expect("apply insert anchor on segment");
    assert_insert_anchor_on_segment_lengths(&original_shape, &after_insert)
        .expect("insert anchor should add one anchor and segment");

    inverse
        .apply(&mut doc)
        .expect("undo insert anchor on segment");
    let restored = shape_at(&doc, 0).expect("shape exists");
    assert_eq!(
        restored, &original_shape,
        "undo after InsertAnchorOnSegment should restore the original path"
    );

    let (_, redone) = assert_insert_anchor_on_segment_effect(&cmd, &mut doc)
        .expect("redo insert anchor on segment");
    assert_insert_anchor_on_segment_lengths(&original_shape, &redone)
        .expect("redo should reapply anchor insertion");
    assert_eq!(
        redone, after_insert,
        "redo after InsertAnchorOnSegment should restore the modified path"
    );
}
#[test]
fn delete_anchors_preserves_handles_and_undoes() {
    let old_shape = shape_with_three_anchors(shape_id(9));
    let mut doc = doc_with_shapes(vec![old_shape.clone()]);

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
fn close_path_replaces_shape_and_undoes() -> Result<(), CommandEditingTestError> {
    let open_shape = shape_with_handles(shape_id(10));
    let mut closed_shape = open_shape.clone();
    closed_shape.path.closed = true;
    closed_shape.path.closing_segment = SegmentKind::Cubic;

    assert_shape_replacement_applies_and_undoes(0, open_shape, closed_shape, |replacement| {
        Command::ClosePath { replacement }
    })?;

    Ok(())
}
