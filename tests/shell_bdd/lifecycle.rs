//! Focused GPUI lifecycle facade for shell BDD scenario binaries.
//!
//! Shell BDD binaries need application initialisation and one completed initial
//! draw without depending on the broad legacy `common` helper surface.

#[path = "../common/init_app.rs"]
mod init_app;
#[path = "../common/initial_draw.rs"]
mod initial_draw;

/// Initializes the application for shell BDD lifecycle tests.
pub use init_app::init_test_app;
/// Completes the initial draw for shell BDD lifecycle tests.
pub use initial_draw::ensure_initial_draw;
