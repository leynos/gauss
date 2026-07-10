//! Durable GPUI handles shared by the history BDD step libraries.
#![expect(
    dead_code,
    reason = "each integration test uses only the shared shell constructors it needs"
)]

use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, TestAppContext, VisualContext, VisualTestContext};
use test_support::{TestSupportError, TestSupportResult};

use crate::common::{ensure_initial_draw, init_test_app};

/// Handles that may safely outlive an individual BDD step.
#[derive(Clone)]
pub struct DurableShell {
    entity: Entity<Phase0Shell>,
    window: AnyWindowHandle,
}

impl DurableShell {
    /// Create a normal Phase 0 shell for interaction-driven scenarios.
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
    pub fn with_visual<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> TestSupportResult<R>,
    ) -> TestSupportResult<R> {
        let mut visual_cx = VisualTestContext::from_window(self.window, cx);
        f(&mut visual_cx, &self.entity)
    }

    /// Read state directly after an interaction that closes the visual context.
    pub const fn entity(&self) -> &Entity<Phase0Shell> {
        &self.entity
    }
}

/// Report a missing scenario value without panicking in a step.
pub fn missing(name: &str) -> TestSupportError {
    TestSupportError::missing(name, "set by an earlier scenario step")
}
