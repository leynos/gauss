// NOTE: This is a simplified illustration. The actual implementation in
// src/model/command.rs includes debug_assert! statements and different
// return types.

fn apply_delete_shapes(doc: &mut Document, targets: &[DeletedShape]) -> CommandInverse {
    // Remove shapes in reverse index order to preserve indices during removal
    let mut sorted_indices: Vec<usize> = targets.iter().map(|t| t.index).collect();
    sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

    for &index in &sorted_indices {
        debug_assert!(
            index < doc.shapes.len(),
            "apply_delete_shapes: index {index} out of range"
        );
        if index < doc.shapes.len() {
            doc.shapes.remove(index);
        }
    }

    CommandInverse::RestoreShapes {
        targets: targets.to_vec(),
    }
}

fn apply_restore_shapes(doc: &mut Document, targets: &[DeletedShape]) {
    // Insert shapes in forward index order to preserve indices during insertion
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_by_key(|t| t.index);

    for target in sorted_targets {
        debug_assert!(
            target.index <= doc.shapes.len(),
            "apply_restore_shapes: index {} out of range",
            target.index
        );
        if target.index <= doc.shapes.len() {
            doc.shapes.insert(target.index, target.shape);
        }
    }
}
