//! SVG import, export, and metadata support for Gauss.
//!
//! `gauss-svg` depends on `gauss-core` for the editor data model and keeps the
//! repository's SVG round-tripping code separate from GPUI application wiring.

pub use gauss_core::model;
pub mod svg;
#[cfg(any(test, feature = "test-support"))]
pub use gauss_core::test_helpers;
