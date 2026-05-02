//! World state for shell mode indicator BDD tests.
//!
//! Uses a thread-local `RefCell` rather than a `StepContext` fixture to avoid
//! `FixtureRefMut` borrow conflicts when steps need both
//! `&mut TestAppContext` (from the harness) and mutable world state.

use std::cell::RefCell;

use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity};

/// World holding the shell entity and its parent window handle.
///
/// Populated during the Given step.  `VisualTestContext` is reconstructed
/// from the window handle in each step that needs it via
/// `VisualTestContext::from_window`.
#[derive(Default)]
pub(crate) struct ShellWorld {
    pub(crate) shell: Option<Entity<Phase0Shell>>,
    pub(crate) window: Option<AnyWindowHandle>,
}

thread_local! {
    static WORLD: RefCell<ShellWorld> = RefCell::new(ShellWorld::default());
}

/// Returns a reference to the thread-local world.
///
/// Each GPUI integration test runs on a single thread, so thread-local
/// storage gives full isolation without `Send + Sync` constraints.
pub(crate) fn with_world<F, R>(f: F) -> R
where
    F: FnOnce(&RefCell<ShellWorld>) -> R,
{
    WORLD.with(f)
}
