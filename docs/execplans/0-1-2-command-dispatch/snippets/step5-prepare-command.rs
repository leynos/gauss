/// Prepare a command from an action and current editor state.
///
/// This function bridges user intent (Action) to concrete command (Command).
/// It captures required context (selection, document state) at the moment
/// the action is invoked.
///
/// # Errors
///
/// Returns `CommandError` if the action cannot produce a valid command
/// (e.g., DeleteSelection with empty selection).
pub fn prepare_command(
    action: Action,
    doc: &Document,
    selection: &Selection,
) -> Result<Command, CommandError> {
    match action {
        Action::DeleteSelection => prepare_delete_selection(doc, selection),
        // Other Document actions would be handled here
        _ => unreachable!("only Document actions should be dispatched to prepare_command"),
    }
}

fn prepare_delete_selection(
    doc: &Document,
    selection: &Selection,
) -> Result<Command, CommandError> {
    if selection.is_empty() {
        return Err(CommandError::EmptySelection);
    }

    // Collect selected shape IDs
    let shape_ids: Vec<ShapeId> = selection.selected_shapes().collect();

    if shape_ids.is_empty() {
        return Err(CommandError::EmptySelection);
    }

    // Build DeletedShape entries with indices and data
    let mut targets = Vec::with_capacity(shape_ids.len());
    for &id in &shape_ids {
        let Some(index) = doc.find_index(id) else {
            return Err(CommandError::ShapeNotFound(id));
        };
        let shape = doc.shapes[index].clone();
        targets.push(DeletedShape { index, shape });
    }

    Ok(Command::DeleteShapes { targets })
}
