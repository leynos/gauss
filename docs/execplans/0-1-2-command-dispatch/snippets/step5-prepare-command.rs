// NOTE: This snippet reflects the actual implementation pattern.
// See src/model/command.rs for the full version.

/// Panic message for dispatcher bugs where editor actions reach `prepare_command`.
const DISPATCHER_BUG_MSG: &str = "dispatcher bug: this action does not produce a command \
                                  and should be routed directly";

/// Prepare a command from an action and current editor state.
///
/// This function bridges user intent (Action) to concrete command (Command).
/// It captures required context (selection, document state) at the moment
/// the action is invoked.
///
/// # Errors
///
/// Returns `UserError` if the action cannot produce a valid command
/// (e.g., DeleteSelection with empty selection).
///
/// # Panics
///
/// Panics if an editor action (e.g., `Undo`, `Redo`, `SelectAll`) is passed.
/// These actions do not produce commands and should be routed directly by the
/// dispatcher. A panic here indicates a dispatcher bug.
#[expect(
    clippy::panic_in_result_fn,
    reason = "Panic is intentional fail-fast for dispatcher bugs"
)]
pub fn prepare_command(
    action: Action,
    doc: &Document,
    selection: &Selection,
) -> Result<Command, UserError> {
    match action {
        Action::DeleteSelection => prepare_delete_selection(doc, selection),
        // Editor actions do not produce commands; this is a dispatcher bug.
        Action::SelectAll
        | Action::DeselectAll
        | Action::ActivatePenTool
        | Action::ActivateSelectTool
        | Action::Undo
        | Action::Redo => panic!("{DISPATCHER_BUG_MSG} (got {action:?})"),
    }
}

fn prepare_delete_selection(doc: &Document, selection: &Selection) -> Result<Command, UserError> {
    if selection.is_empty() {
        return Err(UserError::EmptySelection);
    }

    // Collect selected shape IDs
    let shape_ids: Vec<ShapeId> = selection.selected_shapes().collect();

    if shape_ids.is_empty() {
        // Selection contains only anchors/handles/segments, no whole shapes
        return Err(UserError::EmptySelection);
    }

    // Build DeletedShape entries with indices and data
    let mut targets = Vec::with_capacity(shape_ids.len());
    for &id in &shape_ids {
        let Some(index) = doc.find_index(id) else {
            return Err(UserError::ShapeNotFound(id));
        };
        // find_index guarantees valid index; if violated, treat as shape not found
        // (defensive: avoids panic in production while preserving error semantics)
        let Some(shape) = doc.shapes.get(index).cloned() else {
            return Err(UserError::ShapeNotFound(id));
        };
        targets.push(DeletedShape { index, shape });
    }

    Ok(Command::DeleteShapes { targets })
}
