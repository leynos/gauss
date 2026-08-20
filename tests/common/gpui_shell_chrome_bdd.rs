//! Focused structural helpers for shell-chrome BDD scenarios.
//!
//! The scenarios use these canvas and document helpers while `shell_bdd`
//! owns their durable shell lifecycle.

mod canvas_bounds;
mod canvas_click;
mod document;
mod draw_shape;

/// Locates the canvas bounds in the rendered shell.
pub use canvas_bounds::canvas_bounds;
/// Clicks a canvas position and waits for processing to settle.
pub use canvas_click::click_canvas_and_wait;
/// Reads the current document from the shell.
pub use document::read_document;
/// Obtains the current drawing shape with contextual failure reporting.
pub use draw_shape::require_draw_shape;
