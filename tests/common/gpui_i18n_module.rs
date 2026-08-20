//! Support facade for the `gpui_i18n_module` integration test.
//!
//! It exposes app initialization and owned test-catalogue construction for the
//! internationalization harness.

mod catalog;
mod init_app;

/// Builds an owned catalogue from harness-provided message pairs.
pub use catalog::make_test_catalog;
/// Provides the standard owned French catalogue used by this harness.
pub use catalog::test_french_catalog;
/// Initializes Gauss for the internationalization harness.
pub use init_app::init_test_app;
