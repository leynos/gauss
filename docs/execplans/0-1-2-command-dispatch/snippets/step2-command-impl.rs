impl Command {
    /// Return a human-readable name for this command.
    ///
    /// This name is suitable for undo/redo menu entries and accessibility.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeleteShapes { .. } => "Delete",
        }
    }

    /// Apply the command to the document, returning the inverse for undo.
    ///
    /// # Errors
    ///
    /// Returns `UserError` if the command cannot be executed (e.g.,
    /// referenced shapes do not exist).
    pub fn apply(&self, doc: &mut Document) -> Result<CommandInverse, UserError> {
        let command_name = self.name();
        match self {
            Self::DeleteShapes { targets } => apply_delete_shapes(doc, targets, command_name),
        }
    }
}
