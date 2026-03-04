//! Scenario entrypoints bound to `tests/features/select_tool.feature`.

use super::*;
use rstest_bdd_macros::scenario;

macro_rules! select_tool_scenario {
    ($fn_name:ident, $name:literal) => {
        #[scenario(path = "tests/features/select_tool.feature", name = $name)]
        fn $fn_name(world: SelectToolWorld) {
            let _ = world;
        }
    };
}

#[rustfmt::skip]
select_tool_scenario!(pointer_down_selects_and_stays_idle, "Pointer down on shape selects it and stays idle");
#[rustfmt::skip]
select_tool_scenario!(shift_pointer_down_toggles_and_stays_idle, "Shift pointer down toggles selection and stays idle");
#[rustfmt::skip]
select_tool_scenario!(pointer_move_emits_preview, "Pointer move with dragging state emits preview command");
#[rustfmt::skip]
select_tool_scenario!(pointer_up_no_delta_restores_and_idles, "Pointer up with no movement restores preview and returns idle");
#[rustfmt::skip]
select_tool_scenario!(pointer_up_delta_emits_move_and_idles, "Pointer up with movement emits move command and returns idle");
#[rustfmt::skip]
select_tool_scenario!(pointer_move_in_draw_is_noop, "Pointer move in draw mode emits nothing");
#[rustfmt::skip]
select_tool_scenario!(pointer_move_in_marquee_is_noop, "Pointer move in Marquee state emits nothing");
#[rustfmt::skip]
select_tool_scenario!(pointer_move_in_transforming_is_noop, "Pointer move in Transforming state emits nothing");
#[rustfmt::skip]
select_tool_scenario!(pointer_move_dragging_without_primary_is_noop, "Pointer move with dragging state and released primary button emits nothing");
#[rustfmt::skip]
select_tool_scenario!(pointer_up_in_marquee_is_noop, "Pointer up in Marquee state emits nothing");
#[rustfmt::skip]
select_tool_scenario!(pointer_up_in_transforming_is_noop, "Pointer up in Transforming state emits nothing");
#[rustfmt::skip]
select_tool_scenario!(pointer_up_dragging_non_primary_is_noop, "Pointer up with dragging state and non-primary button emits nothing");
#[rustfmt::skip]
select_tool_scenario!(pointer_up_anchor_drag_emits_move_anchor, "Pointer up with anchor drag emits move anchor command and returns idle");
#[rustfmt::skip]
select_tool_scenario!(pointer_up_handle_drag_emits_move_handle, "Pointer up with handle drag emits move handle command and returns idle");
