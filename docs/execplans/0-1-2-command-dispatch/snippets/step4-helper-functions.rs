fn apply_delete_shapes(
    doc: &mut Document,
    targets: &[DeletedShape],
) -> Result<CommandInverse, CommandError> {
    // Remove shapes in reverse index order to preserve indices
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_by(|a, b| b.index.cmp(&a.index));

    for target in &sorted_targets {
        if target.index < doc.shapes.len() {
            doc.shapes.remove(target.index);
        }
    }

    Ok(CommandInverse::RestoreShapes {
        targets: targets.to_vec(),
    })
}

fn apply_restore_shapes(
    doc: &mut Document,
    targets: &[DeletedShape],
) -> Result<(), CommandError> {
    // Insert shapes in forward index order to preserve indices
    let mut sorted_targets = targets.to_vec();
    sorted_targets.sort_by(|a, b| a.index.cmp(&b.index));

    for target in sorted_targets {
        if target.index <= doc.shapes.len() {
            doc.shapes.insert(target.index, target.shape);
        }
    }

    Ok(())
}
