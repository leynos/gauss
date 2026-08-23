//! Durable GPUI handles shared by the history BDD step libraries.
//!
//! This module re-exports the common durable shell used by the Phase 0 BDD
//! binaries, while the adjacent extensions provide history-specific creation
//! and entity access for `GpuiHarness` scenario bindings.

#[path = "../common/durable_shell.rs"]
mod common_durable_shell;

use gauss::ui::Phase0Shell;
use gpui::{Entity, TestAppContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

pub use common_durable_shell::DurableShell;

impl DurableShell {
    /// Rebuild a visual context for the current history BDD step.
    ///
    /// The history scenario binaries use this compatibility name while sharing
    /// the durable handle implementation with the broader GPUI test suite.
    pub fn with_visual<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
    ) -> TestSupportResult<R> {
        self.with_visual_cx(cx, f)
    }
}

/// Report a missing scenario value without panicking in a step.
///
/// The returned error identifies the value that an earlier step should have
/// installed in scenario state.
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::history_bdd_support::{missing, DurableShell};
/// # use test_support::TestSupportError;
/// # fn shell_or_missing(shell: Option<DurableShell>) -> Result<DurableShell, TestSupportError> {
/// shell.ok_or_else(|| missing("Phase 0 shell"))
/// # }
/// ```
pub fn missing(name: &str) -> TestSupportError {
    TestSupportError::missing(name, "set by an earlier scenario step")
}
