//! Test-only document-history access for engine-state assertions.
//!
//! This extension keeps production engine-state APIs focused while allowing
//! headless UI tests to inspect the complete command history.

use super::EngineState;
use crate::model::history::DocumentHistoryState;

impl EngineState {
    /// Return a complete document-history snapshot for headless tests.
    #[must_use]
    pub fn document_history_state_for_tests(&self) -> DocumentHistoryState {
        self.document_history.state_for_tests()
    }
}
