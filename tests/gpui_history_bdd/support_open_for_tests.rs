//! Test-seam shell construction for history scenario binaries that need it.

use gauss::ui::Phase0Shell;
use gpui::{TestAppContext, VisualContext};

use crate::{
    common::{ensure_initial_draw, init_test_app},
    history_bdd_support::DurableShell,
};

impl DurableShell {
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
}
