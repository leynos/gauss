//! Focused GPUI lifecycle facade for shell BDD scenario binaries.
//!
//! Shell BDD binaries need application initialisation and one completed initial
//! draw without depending on the broad legacy `common` helper surface.

#[path = "../common/init_app.rs"]
mod init_app;
#[path = "../common/initial_draw.rs"]
mod initial_draw;

pub use init_app::init_test_app;
pub use initial_draw::ensure_initial_draw;
