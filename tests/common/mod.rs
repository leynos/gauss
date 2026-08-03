//! Shared helpers for Phase 0 GPUI integration tests.
//!
//! The helpers themselves live in [`shared_helpers`] and are re-exported here so
//! existing `use common::...` paths keep working. That module carries the
//! `dead_code` expectation rather than this one, because the expectation is only
//! defensible for the legacy shared surface: see its module documentation for
//! why the suppression cannot be narrowed to individual items.
mod shared_helpers;

pub use shared_helpers::*;
