//! Harness facade for the `tests/gpui_history_command_grouping_undo.rs` integration test.
//!
//! Re-exports app setup, document history controls, history inspection, and
//! vector assertions for grouped undo.

#[path = "document.rs"]
mod document;
#[path = "document_undo.rs"]
mod document_undo;
#[path = "history.rs"]
mod history;
#[path = "init_app.rs"]
mod init_app;
#[path = "initial_draw.rs"]
mod initial_draw;
#[path = "vec2_assertion.rs"]
mod vec2_assertion;

/// Provides the harness with an owned snapshot of the shell's current document.
pub use document::read_document;
/// Dispatches document undo and waits for the GPUI event loop to become parked.
pub use document_undo::simulate_document_undo;
/// Provides the harness with the shell's current document-history entry count.
pub use history::read_history_len;
/// Initializes the Gauss application in the harness's GPUI test context.
pub use init_app::init_test_app;
/// Performs the initial draw and waits for GPUI to become parked.
pub use initial_draw::ensure_initial_draw;
/// Checks that harness-observed vectors are within the helper's tolerance.
pub use vec2_assertion::assert_vec2_close;

use gauss::ui::Phase0Shell;
use gpui::{Entity, VisualTestContext};

/// Re-export the test-support document-history snapshot for BDD consumers.
pub use gauss::ui::DocumentHistoryState;

/// Returns the current entries and cursor availability in the document history.
pub fn read_history_state(
    visual_cx: &VisualTestContext,
    view: &Entity<Phase0Shell>,
) -> DocumentHistoryState {
    visual_cx.read(|app| view.read(app).document_history_state_for_tests())
}
