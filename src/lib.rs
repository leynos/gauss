//! Gauss core library.
//!
//! This crate holds the editor's pure data model and reusable logic.
//!
//! High-level structure:
//!
//! - `gauss::model`: editor data structures that are independent of GPUI and can
//!   be tested without a window.
//! - `gauss::ui`: GPUI views and platform wiring (window actions, dialogs).
//!
//! The GPUI application entrypoint lives in `src/main.rs`.

pub mod model;
pub mod svg;
pub mod ui;

/// Returns a greeting for the library.
#[must_use]
pub const fn greet() -> &'static str {
    "Hello from Gauss!"
}
