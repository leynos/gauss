//! Narrow support surface for `gpui_i18n_module.rs`.

mod catalog;
mod init_app;

pub use catalog::make_test_catalog;
pub use catalog::test_french_catalog;
pub use init_app::init_test_app;
