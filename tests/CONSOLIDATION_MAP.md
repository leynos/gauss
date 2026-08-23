# Integration Test Organization Mapping

## Current authoritative inventory

An **integration test target** is a target of kind `test` in the root `gauss`
package reported by `cargo metadata --no-deps --format-version 1`. It is not a
filename discovered by a glob. A **GPUI target** is either a harness-backed
GPUI BDD target or a raw structural GPUI target.

_Table: Current authoritative root `gauss` integration-test inventory derived
from Cargo metadata._

| Classification                   | Definition                                                                                   | Current targets |
| -------------------------------- | -------------------------------------------------------------------------------------------- | --------------: |
| Harness-backed GPUI BDD          | Contains a `#[scenario]` registration with `harness = rstest_bdd_harness_gpui::GpuiHarness`. | 29              |
| Raw structural GPUI              | Uses a direct `#[gpui::test]` registration rather than the BDD harness.                      | 5               |
| Non-GPUI BDD                     | Contains `#[scenario]` but no `GpuiHarness` registration.                                    | 5               |
| Other integration test           | Has none of the preceding registrations.                                                     | 12              |
| **All integration test targets** | **The four mutually exclusive classifications above.**                                       | **51**          |

<!-- integration-test-inventory: total=51 harness_gpui_bdd=29
raw_structural_gpui=5 non_gpui_bdd=5 other_integration=12 gpui_target=34 -->

Run `make check-integration-test-inventory` to derive these counts from Cargo
metadata and fail if this document or the linked inventories drift.

### Harness-backed GPUI BDD targets (29)

- `gpui_draw_undo_bdd`
- `gpui_file_io_click_save_button`
- `gpui_file_io_metadata_round_trip`
- `gpui_file_io_open_dialog`
- `gpui_file_io_save_dialog`
- `gpui_history_drag_shape_undo`
- `gpui_history_draw_undo`
- `gpui_history_open_history_reset`
- `gpui_history_selection_history`
- `gpui_i18n_module`
- `gpui_selection_bbox_drag_requires_selection`
- `gpui_selection_clear_selection`
- `gpui_selection_multi_select`
- `gpui_selection_multi_shape_drag`
- `gpui_selection_select_shape_by_bbox`
- `gpui_selection_select_tool_noop_paths`
- `gpui_shell_a11y_service`
- `gpui_shell_chrome_bdd`
- `gpui_shell_mode_indicator`
- `gpui_shell_navigation_buttons`
- `gpui_shell_tool_rail`
- `gpui_shell_viewport_input`
- `gpui_tooling_close_path`
- `gpui_tooling_draw_bezier_auto`
- `gpui_tooling_draw_escape_commits_open_path`
- `gpui_tooling_escape_returns_to_draw`
- `gpui_tooling_hit_test_service`
- `gpui_tooling_keybinding_integration`
- `gpui_tooling_toggle_segment_kind`

### Raw structural GPUI targets (5)

- `gpui_shell_canvas_layout`
- `gpui_shell_chrome_layout`
- `gpui_shell_resize_borders`
- `gpui_shell_style_controls`
- `gpui_shell_window_controls`

The retained window-control target is structural and native-platform-limited:
it checks geometry, element presence, and test plumbing. It does not claim to
drive a user-triggered maximize or resize operation, which the GPUI test
platform cannot exercise.

### Non-GPUI BDD targets (5)

- `a11y_service_bdd`
- `a11y_service_routing_bdd`
- `i18n_bdd`
- `undo_entry_count_bdd`
- `widget_capability_audit_bdd`

### Other integration targets (12)

- `gpui_history_anchor_edit_undo`
- `gpui_history_close_path_undo`
- `gpui_history_command_grouping_undo`
- `gpui_history_drag_anchor_undo`
- `gpui_history_drag_handle_undo`
- `gpui_history_multi_shape_drag_undo`
- `gpui_history_reorder_undo`
- `gpui_widget_audit_shell_seam`
- `temp_file_cleanup`
- `test_support_const_apis`
- `vec2_assertion`
- `widget_audit_test`

## Historical consolidation snapshot (2026-03-17)

The figures below are retained only as the dated result of the original flat
target-naming consolidation. They are not the current inventory.

- **Before**: 56 top-level integration test files (39 GPUI + 17 non-GPUI)
- **After**: 56 top-level integration test files (39 GPUI + 17 non-GPUI)

That consolidation organized targets with `gpui_{group}_{name}.rs` naming. The
directory now also contains both raw direct `#[gpui::test]` targets and
`GpuiHarness`-backed BDD targets, so filename patterns are not classifications.
