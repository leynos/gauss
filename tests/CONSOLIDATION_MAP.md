# Integration Test Organization Mapping

This document describes the organization of 55 top-level integration test files
using a flat naming convention grouped by feature area.

## GPUI (General Purpose UI) Tests (38 files organized into 5 feature areas)

The 38 GPUI integration tests use a flat naming pattern
`gpui_{group}_{test_name}.rs` to organize tests by feature area while
maintaining each test as an independent Cargo target.

### Shell/Chrome - Window and Chrome Behaviour (10 tests)

- gpui_shell_a11y_service.rs
- gpui_shell_canvas_layout.rs
- gpui_shell_chrome_layout.rs
- gpui_shell_mode_indicator.rs
- gpui_shell_navigation_buttons.rs
- gpui_shell_resize_borders.rs
- gpui_shell_style_controls.rs
- gpui_shell_tool_rail.rs
- gpui_shell_viewport_input.rs
- gpui_shell_window_controls.rs

### History - Undo/Redo and History (11 tests)

- gpui_history_anchor_edit_undo.rs
- gpui_history_close_path_undo.rs
- gpui_history_command_grouping_undo.rs
- gpui_history_drag_anchor_undo.rs
- gpui_history_drag_handle_undo.rs
- gpui_history_drag_shape_undo.rs
- gpui_history_draw_undo.rs
- gpui_history_multi_shape_drag_undo.rs
- gpui_history_open_history_reset.rs
- gpui_history_reorder_undo.rs
- gpui_history_selection_history.rs

### File I/O - Open/Save and Metadata Round-trip (4 tests)

- gpui_file_io_click_save_button.rs
- gpui_file_io_metadata_round_trip.rs
- gpui_file_io_open_dialog.rs
- gpui_file_io_save_dialog.rs

### Selection - Selection, Drag, Resize, Reorder (6 tests)

- gpui_selection_bbox_drag_requires_selection.rs
- gpui_selection_clear_selection.rs
- gpui_selection_multi_select.rs
- gpui_selection_multi_shape_drag.rs
- gpui_selection_select_shape_by_bbox.rs
- gpui_selection_select_tool_noop_paths.rs

### Tooling - Tool Activation, Drawing, Keybindings (7 tests)

- gpui_tooling_close_path.rs
- gpui_tooling_draw_bezier_auto.rs
- gpui_tooling_draw_escape_commits_open_path.rs
- gpui_tooling_escape_returns_to_draw.rs
- gpui_tooling_hit_test_service.rs
- gpui_tooling_keybinding_integration.rs
- gpui_tooling_toggle_segment_kind.rs

## Non-GPUI Tests (17 files, unchanged)

### BDD (Behaviour-Driven Development) Tests (13 files)

- a11y_service_bdd.rs - Accessibility service behaviour
- a11y_service_routing_bdd.rs - Accessibility routing behaviour
- action_bdd.rs - Action system behaviour
- command_bdd.rs - Command behaviour
- gauss_model_ops_bdd.rs - Model operations behaviour
- hit_test_bdd.rs - Hit testing behaviour
- metadata_round_trip_bdd.rs - Metadata round-trip behaviour
- pen_tool_bdd.rs - Pen tool behaviour
- resource_store_bdd.rs - Resource store behaviour
- select_tool_bdd.rs - Select tool behaviour
- stable_id_bdd.rs - Stable ID behaviour
- tool_fsm_bdd.rs - Tool FSM behaviour
- web_ready_export_bdd.rs - Web-ready export behaviour

### Unit Tests (3 files)

- command_editing_helpers.rs - Command editing helper unit tests
- command_editing_unit.rs - Command editing unit tests
- command_unit.rs - Command unit tests

### Integration Tests (1 file)

- golden_round_trip.rs - Golden file round-trip test

## Summary

These counts are the historical consolidation snapshot affected by the removal
of `gpui_shell_quit_button.rs`, rather than a live inventory of this directory.

**Before**: 56 top-level integration test files (39 GPUI + 17 non-GPUI)

**After**: 55 top-level integration test files (38 GPUI + 17 non-GPUI)

**Change**: GPUI tests reorganized using flat naming pattern
`gpui_{group}_{name}.rs` to group tests by feature area (shell, history,
file_io, selection, tooling) while maintaining each test as an independent
Cargo target for parallel execution and test isolation.

## Rationale for Flat Naming vs Nested Modules

The implementation uses flat file naming (`gpui_shell_foo.rs`) rather than
nested modules (`gpui_shell/foo.rs`) because:

- Flat naming keeps simple `mod common` imports vs complex
  `use super::super::common`
- Each test remains an independent Cargo target for parallel execution
- Simpler module structure aligns with Rust test ecosystem expectations
- Naming convention provides same discoverability as nested structure
