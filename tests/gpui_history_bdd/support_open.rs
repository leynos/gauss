//! Normal-shell construction for history scenario binaries that drive the UI.
//!
//! This extension opens and initially draws a normal Phase 0 shell for the
//! interaction-driven anchor, path, drag, and reorder scenarios. The parent
//! integration binaries invoke it through `GpuiHarness` and combine it with
//! the durable-shell and shared canvas support modules.

use gauss::ui::Phase0Shell;
use gpui::TestAppContext;

use crate::{
    common::{ensure_initial_draw, init_test_app},
    history_bdd_support::DurableShell,
};

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
        Self::new(entity, visual_cx)
    }
}
