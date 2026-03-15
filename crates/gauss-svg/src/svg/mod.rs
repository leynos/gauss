//! SVG import/export.
//!
//! Phase 0 uses SVG as the on-disk interchange format. The exporter aims to be
//! deterministic (stable attribute ordering and command formatting), and the
//! importer is intentionally limited to the subset of SVG we produce:
//!
//! - Elements: `<path ... />` plus selected `<defs>` resources
//! - Path commands: `M`, `L`, `C`, `Z` (absolute only)
//! - Styling attributes: `stroke`, `stroke-width`, `stroke-opacity`, `fill`,
//!   `fill-opacity`
//!
//! This is sufficient for round-tripping Gauss-produced files and importing
//! many simple SVGs.

pub mod export;
pub mod import;
pub mod metadata;

#[cfg(test)]
mod import_tests;
