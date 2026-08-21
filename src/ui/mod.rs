//! User interface views and wiring.
//!
//! Phase 0 intentionally keeps UI minimal. The intent is to create a place for
//! GPUI-specific code (views, actions, and platform interactions) while keeping
//! the data model in `crate::model` usable without a running window.

pub mod action_bridge;
mod canvas_paint;
mod gauss_window_border;
mod icon_assets;
pub mod phase0_shell;
mod phase0_support;
mod selection_overlays;
mod selection_utils;
mod viewport_input;
pub mod widget_audit;

pub use action_bridge::{
    GpuiActivatePenTool, GpuiActivateSelectTool, GpuiDeleteSelection, GpuiDeselectAll, GpuiRedo,
    GpuiSelectAll, GpuiSelectionRedo, GpuiSelectionUndo, GpuiUndo,
};
pub use gauss_window_border::GaussRoot;
pub(crate) use icon_assets::{UiIcon, icon_element};
#[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]
pub use phase0_shell::DocumentHistoryState;
pub use phase0_shell::{ExportSvgWebReady, OpenSvg, Phase0Shell, SaveSvg};

/// Initialise GPUI integrations used by Gauss.
///
/// Phase 0 relies on:
///
/// - `gpui-component` services (colour pickers, history primitives, root wrapper)
/// - Key bindings for editor actions (for example, `Tab` in draw mode)
/// - Action bindings from the model-layer keybinding registry
pub fn init(app: &mut gpui::App) {
    gpui_component::init(app);
    action_bridge::register_action_bindings(app);
    phase0_shell::bind_keymap(app);
}
