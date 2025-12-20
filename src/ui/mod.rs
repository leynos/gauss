//! User interface views and wiring.
//!
//! Phase 0 intentionally keeps UI minimal. The intent is to create a place for
//! GPUI-specific code (views, actions, and platform interactions) while keeping
//! the data model in `crate::model` usable without a running window.

mod canvas_paint;
mod icon_assets;
pub mod phase0_shell;
mod phase0_support;
mod selection_overlays;
mod selection_utils;
mod viewport_input;

pub(crate) use icon_assets::{UiIcon, icon_element};
pub use phase0_shell::{OpenSvg, Phase0Shell, SaveSvg};

/// Initialise GPUI integrations used by Gauss.
///
/// Phase 0 relies on:
/// - `gpui-component` services (colour pickers, history primitives, root wrapper)
/// - key bindings for editor actions (for example, `Tab` in draw mode).
pub fn init(app: &mut gpui::App) {
    gpui_component::init(app);
    phase0_shell::bind_keymap(app);
}
