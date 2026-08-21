//! Durable GPUI handles shared by the history BDD step libraries.
use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, TestAppContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

/// Handles that may safely outlive an individual BDD step.
///
/// The handle stores the shell entity and window handle, so a later step can
/// rebuild its short-lived visual context without retaining a borrow from an
/// earlier `TestAppContext`.
///
/// # Examples
///
/// A step can retain a clone for a later step:
///
/// ```rust,no_run
/// # use gpui::TestAppContext;
/// # use crate::history_bdd_support::DurableShell;
/// # fn open_window(cx: &mut TestAppContext) {
/// let shell = DurableShell::open(cx);
/// let shell_for_next_step = shell.clone();
/// # let _ = shell_for_next_step;
/// # }
/// ```
#[derive(Clone)]
pub struct DurableShell {
    pub(crate) entity: Entity<Phase0Shell>,
    pub(crate) window: AnyWindowHandle,
}

impl DurableShell {
    /// Rebuild a visual context for the current harness step.
    ///
    /// The closure receives a context valid for the duration of the call and
    /// the shell entity. Its result is returned to the step unchanged.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gpui::TestAppContext;
    /// # use crate::history_bdd_support::DurableShell;
    /// # use test_support::TestSupportResult;
    /// # fn inspect(shell: &DurableShell, cx: &mut TestAppContext) -> TestSupportResult<()> {
    /// shell.with_visual(cx, |_visual_cx, _shell_entity| Ok(()))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_visual<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
    ) -> TestSupportResult<R> {
        let mut visual_cx = VisualTestContext::from_window(self.window, cx);
        f(&mut visual_cx, &self.entity)
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
