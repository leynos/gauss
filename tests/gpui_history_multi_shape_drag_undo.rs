//! BDD coverage for one-entry multi-shape drag history.

#[path = "common/scenario_state.rs"]
mod scenario_state;

#[path = "common/gpui_history_multi_shape_drag_undo.rs"]
mod common;

#[path = "gpui_history_bdd/support.rs"]
mod history_bdd_support;
#[path = "gpui_history_bdd/support_open.rs"]
mod history_bdd_support_open;

#[path = "gpui_history_bdd/multi_shape.rs"]
mod multi_shape;
