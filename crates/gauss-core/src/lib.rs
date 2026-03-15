//! Pure editor model and deterministic test helpers for Gauss.
//!
//! `gauss-core` owns the GPUI-independent editor domain: document state,
//! commands, history, hit testing, selection, tools, and related fixtures.

pub mod model;
#[cfg(any(test, feature = "test-support"))]
pub mod test_helpers;
