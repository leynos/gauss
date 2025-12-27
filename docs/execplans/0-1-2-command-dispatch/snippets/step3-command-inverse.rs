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
        /// Shapes to restore, with their original indices.
        targets: Vec<DeletedShape>,
    },
}

impl CommandInverse {
    /// Return a human-readable name for this inverse command.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::RestoreShapes { .. } => "Delete",
        }
    }

    /// Apply the inverse command to restore previous state.
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if the inverse cannot be applied.
    pub fn apply(&self, doc: &mut Document) -> Result<(), CommandError> {
        match self {
            Self::RestoreShapes { targets } => apply_restore_shapes(doc, targets),
        }
    }
}
