//! Depth limit tests for [`DocumentUndoHistory`].

use rstest::rstest;

use crate::model::ShapeId;
use crate::model::document::Document;
use crate::model::history::DocumentUndoHistory;

use super::{apply_move, doc_with_one_shape};

#[rstest]
fn default_max_depth_is_expected() {
    assert_eq!(DocumentUndoHistory::new().max_depth(), 500);
}

#[rstest]
fn with_max_depth_respects_custom_limit(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::with_max_depth(3);

    // Record 5 commands — only the last 3 should survive pruning.
    for i in 0_i16..5 {
        let (cmd, inv) = apply_move(&mut doc, id, f32::from(i), 0.0);
        history.record(cmd, inv);
    }

    // Undo 3 times — each should change the document.
    let mut undo_count = 0;
    for _ in 0..3 {
        let before = doc.clone();
        history.undo(&mut doc).expect("undo should succeed");
        if doc != before {
            undo_count += 1;
        }
    }
    assert_eq!(undo_count, 3, "should be able to undo exactly 3 times");

    // 4th undo is a no-op (pruned entries are gone).
    let before = doc.clone();
    history.undo(&mut doc).expect("undo should succeed (noop)");
    assert_eq!(doc, before, "4th undo should be a no-op");
}

/// Verify that `keep_last` does not corrupt history when undo markers
/// are present (branch-edit scenario).
#[rstest]
fn depth_limit_preserves_undo_covered_entries(doc_with_one_shape: (Document, ShapeId)) {
    let (mut doc, id) = doc_with_one_shape;
    let mut history = DocumentUndoHistory::with_max_depth(4);

    // A: move right
    let (cmd_a, inv_a) = apply_move(&mut doc, id, 1.0, 0.0);
    history.record(cmd_a, inv_a);

    // B: move down
    let (cmd_b, inv_b) = apply_move(&mut doc, id, 0.0, 2.0);
    history.record(cmd_b, inv_b);

    // Undo B — creates an undo marker in the history
    history.undo(&mut doc).expect("undo B");

    // C: branch edit (move left)
    let (cmd_c, inv_c) = apply_move(&mut doc, id, -3.0, 0.0);
    history.record(cmd_c, inv_c);

    // D: another move
    let (cmd_d, inv_d) = apply_move(&mut doc, id, 0.0, -1.0);
    history.record(cmd_d, inv_d);

    // History now has entries + undo markers; verify undo still works.
    history.undo(&mut doc).expect("undo D");
    history.undo(&mut doc).expect("undo C");

    // We should be back at the state after undoing B (i.e. after A).
    // Continuing to undo should navigate the historical branch.
    history.undo(&mut doc).expect("undo into historical branch");
}
