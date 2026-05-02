//! BDD tests for the Phase 0 shell mode indicator.
//!
//! Validates the mode indicator transitions using rstest-bdd v0.6.0
//! with the GPUI harness. Each scenario is bound individually to
//! avoid `FixtureRefMut` borrow conflicts from macro-generated loops.
//!
//! World state is stored in a thread-local `RefCell` (see `world.rs`)
//! rather than as a `StepContext` fixture, because `StepContext::borrow_mut`
//! requires `&mut self` and a step that needs both a mutable harness
//! context and mutable world state would produce a double-mutable-borrow
//! error.

mod common;
mod gpui_shell_mode_indicator_bdd {
    pub(crate) mod steps;
    pub(crate) mod world;
}

use rstest_bdd_macros::scenario;

#[scenario(
    path = "tests/features/shell_mode_indicator.feature",
    harness = rstest_bdd_harness_gpui::GpuiHarness
)]
fn initial_draw_mode() {}

#[scenario(
    path = "tests/features/shell_mode_indicator.feature",
    harness = rstest_bdd_harness_gpui::GpuiHarness
)]
fn pressing_tab_toggles_the_draw_edge_mode() {}

#[scenario(
    path = "tests/features/shell_mode_indicator.feature",
    harness = rstest_bdd_harness_gpui::GpuiHarness
)]
fn switching_to_manipulate_mode_hides_the_edge_suffix() {}
