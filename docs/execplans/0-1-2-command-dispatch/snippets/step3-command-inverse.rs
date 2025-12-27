/// The inverse of an applied command, used for undo.
///
/// CommandInverse captures everything needed to reverse a command.
/// It is produced by `Command::apply` and stored in the undo stack.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum CommandInverse {
    /// Restore deleted shapes to their original positions.
    RestoreShapes {
        /// Name of the original command (for "Undo {name}" menu entries).
        command_name: &'static str,
        /// Shapes to restore, with their original indices.
        targets: Vec<DeletedShape>,
    },
}

impl CommandInverse {
    /// Return a human-readable name for this inverse command.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::RestoreShapes { command_name, .. } => command_name,
        }
    }

    /// Apply the inverse command to restore previous state.
    ///
    /// # Errors
    ///
    /// Returns `UserError` if the inverse cannot be applied.
    pub fn apply(&self, doc: &mut Document) -> Result<(), UserError> {
        match self {
            Self::RestoreShapes { targets, .. } => apply_restore_shapes(doc, targets),
        }
    }
}
