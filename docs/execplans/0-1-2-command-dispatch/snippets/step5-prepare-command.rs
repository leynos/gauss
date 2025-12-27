// NOTE: This snippet reflects the actual implementation pattern.
// See src/model/command.rs for the full version.

/// Prepare a command from an action and current editor state.
///
/// This function bridges user intent (Action) to concrete command (Command).
/// It captures required context (selection, document state) at the moment
/// the action is invoked.
///
/// # Errors
///
/// Returns `CommandError` if the action cannot produce a valid command
/// (e.g., DeleteSelection with empty selection, or a non-command action).
pub fn prepare_command(
    action: Action,
    doc: &Document,
    selection: &Selection,
) -> Result<Command, CommandError> {
    match action {
        Action::DeleteSelection => prepare_delete_selection(doc, selection),
        // Editor actions do not produce commands; this is a dispatcher bug
        Action::SelectAll
        | Action::DeselectAll
        | Action::ActivatePenTool
        | Action::ActivateSelectTool
        | Action::Undo
        | Action::Redo => Err(CommandError::NotACommand(action)),
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
        // Selection contains only anchors/handles/segments, no whole shapes
        return Err(CommandError::EmptySelection);
    }

    // Build DeletedShape entries with indices and data
    let mut targets = Vec::with_capacity(shape_ids.len());
    for &id in &shape_ids {
        let Some(index) = doc.find_index(id) else {
            return Err(CommandError::ShapeNotFound(id));
        };
        // find_index guarantees valid index; if violated, treat as shape not found
        // (defensive: avoids panic in production while preserving error semantics)
        let Some(shape) = doc.shapes.get(index).cloned() else {
            return Err(CommandError::ShapeNotFound(id));
        };
        targets.push(DeletedShape { index, shape });
    }

    Ok(Command::DeleteShapes { targets })
}
