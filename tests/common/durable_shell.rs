//! Durable GPUI handles that survive between BDD scenario steps.
//!
//! File-I/O, selection, and history BDD binaries include this module by path,
//! with selection scenarios doing so transitively through
//! `tests/selection_bdd/support.rs`. `DurableShell::new` and
//! `with_visual_cx` preserve the entity and window-handle pair needed to
//! reconstruct each short-lived visual context, so no consumer needs a
//! dead-code expectation.

use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, TestAppContext, VisualContext, VisualTestContext};
use test_support::TestSupportResult;

/// Durable GPUI handles that can safely survive between BDD steps.
///
/// A `VisualTestContext` borrows the `TestAppContext` it was built from, and the
/// harness hands each step a fresh borrow, so a context cannot be stored in
/// scenario state. An `Entity` and an `AnyWindowHandle` can, and together they
/// are enough to rebuild a context on demand.
#[derive(Clone)]
pub struct DurableShell {
    /// The shell entity, read directly by steps that only need `cx.read`.
    pub(crate) entity: Entity<Phase0Shell>,
    window: AnyWindowHandle,
}

impl DurableShell {
    /// Captures a durable shell handle from a live `VisualTestContext`.
    ///
    /// The entity and the context's window handle are stored so the pair can outlive the
    /// `VisualTestContext` itself, allowing scenario state to persist across BDD steps.
    pub fn new(entity: Entity<Phase0Shell>, visual_cx: &VisualTestContext) -> Self {
        Self {
            entity,
            window: visual_cx.window_handle(),
        }
    }

    /// Reconstructs a `VisualTestContext` from the stored window handle and runs `f` against it.
    ///
    /// The reconstructed context is scoped to the closure; it is not retained afterwards.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `f` returns `Err`.
    pub fn with_visual_cx<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
    ) -> TestSupportResult<R> {
        let mut visual_cx = VisualTestContext::from_window(self.window, cx);
        f(&mut visual_cx, &self.entity)
    }
}
