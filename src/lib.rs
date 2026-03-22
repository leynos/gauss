//! Gauss application library.
//!
//! The repository now splits pure editor logic and SVG persistence into
//! dedicated workspace crates while keeping the user-facing GPUI application
//! in the root `gauss` package.
//!
//! High-level structure:
//!
//! - `gauss::model`: re-exported pure editor data structures and logic from
//!   `gauss-core`.
//! - `gauss::svg`: re-exported SVG import/export APIs from `gauss-svg`.
//! - `gauss::ui`: GPUI views and platform wiring for the desktop app.
//!
//! The GPUI application entrypoint lives in `src/main.rs`.

pub use gauss_core::model;
#[cfg(any(test, feature = "test-support"))]
pub use gauss_core::test_helpers;
pub use gauss_svg::svg;

pub mod i18n;
pub mod ui;
