//! Support facade for the `gpui_tooling_keybinding_integration` test.
//!
//! It exposes app lifecycle, key dispatch, and selection inspection for the
//! tooling-keybinding harness.

mod init_app;
mod initial_draw;
mod key_input;
mod selection;

/// Initializes Gauss for the keybinding integration harness.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to park.
pub use initial_draw::ensure_initial_draw;
/// Dispatches a keybinding and waits for event processing to settle.
pub use key_input::simulate_key;
/// Returns a copied selection list for observable-state assertions.
pub use selection::read_selection_items;
