//! Read-only shell-entity access for history scenario binaries.
//!
//! This extension exposes the durable shell's entity to steps that must read
//! state after a visual context has been released. The parent integration
//! binaries use it alongside `GpuiHarness`, the durable-shell support module,
//! and the shared history helpers.

use gauss::ui::Phase0Shell;
use gpui::Entity;

use crate::history_bdd_support::DurableShell;

impl DurableShell {
    /// Read state directly after an interaction that closes the visual context.
    ///
    /// The reference can be passed to a read-only helper while the current
    /// step owns the `TestAppContext`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gauss::ui::Phase0Shell;
    /// # use gpui::{Entity, TestAppContext};
    /// # use crate::history_bdd_support::DurableShell;
    /// # fn read_entity(shell: &DurableShell, _cx: &mut TestAppContext) {
    /// let _: &Entity<Phase0Shell> = shell.entity();
    /// # }
    /// ```
    pub const fn entity(&self) -> &Entity<Phase0Shell> {
        &self.entity
    }
}
