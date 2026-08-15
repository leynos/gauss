//! Durable GPUI handles shared by the history BDD step libraries.
use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, TestAppContext, VisualContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

use crate::common::{ensure_initial_draw, init_test_app};

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
    entity: Entity<Phase0Shell>,
    window: AnyWindowHandle,
}

impl DurableShell {
    /// Create a normal Phase 0 shell for interaction-driven scenarios.
    ///
    /// The returned handle owns the window and entity references needed by
    /// subsequent BDD steps.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gpui::TestAppContext;
    /// # use crate::history_bdd_support::DurableShell;
    /// # fn given_shell(cx: &mut TestAppContext) -> DurableShell {
    /// DurableShell::open(cx)
    /// # }
    /// ```
    pub fn open(cx: &mut TestAppContext) -> Self {
        init_test_app(cx);
        let (entity, visual_cx) = cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
        ensure_initial_draw(visual_cx);
        Self {
            entity,
            window: visual_cx.window_handle(),
        }
    }

    /// Create a Phase 0 shell exposing history test seams.
    ///
    /// The returned handle is ready for history-specific step definitions to
    /// use through [`Self::with_visual`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use gpui::TestAppContext;
    /// # use crate::history_bdd_support::DurableShell;
    /// # fn given_history_shell(cx: &mut TestAppContext) -> DurableShell {
    /// DurableShell::open_for_tests(cx)
    /// # }
    /// ```
    pub fn open_for_tests(cx: &mut TestAppContext) -> Self {
        init_test_app(cx);
        let (entity, visual_cx) =
            cx.add_window_view(|_window, view_cx| Phase0Shell::new_for_tests(view_cx));
        ensure_initial_draw(visual_cx);
        Self {
            entity,
            window: visual_cx.window_handle(),
        }
    }

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

const _: fn(&mut TestAppContext) -> DurableShell = DurableShell::open;
const _: fn(&mut TestAppContext) -> DurableShell = DurableShell::open_for_tests;
const _: for<'a> fn(&'a DurableShell) -> &'a Entity<Phase0Shell> = DurableShell::entity;

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
