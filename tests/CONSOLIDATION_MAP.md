# Integration Test Consolidation Mapping

This document tracks the consolidation of 56 top-level integration test files
into grouped targets.

## GPUI Tests (39 files → 5 grouped targets)

### gpui_shell.rs - Window and Chrome Behaviour (11 tests)

- gpui_canvas_layout.rs
- gpui_chrome_layout.rs
- gpui_mode_indicator.rs
- gpui_navigation_buttons.rs
- gpui_quit_button.rs
- gpui_resize_borders.rs
- gpui_style_controls.rs
- gpui_tool_rail.rs
- gpui_viewport_input.rs
- gpui_window_controls.rs
- gpui_a11y_service.rs

### gpui_history.rs - Undo/Redo and History (9 tests)

- gpui_anchor_edit_undo.rs
- gpui_close_path_undo.rs
- gpui_command_grouping_undo.rs
- gpui_drag_anchor_undo.rs
- gpui_drag_handle_undo.rs
- gpui_drag_shape_undo.rs
- gpui_draw_undo.rs
- gpui_multi_shape_drag_undo.rs
- gpui_reorder_undo.rs
- gpui_selection_history.rs
- gpui_open_history_reset.rs

### gpui_file_io.rs - Open/Save and Metadata Round-trip (4 tests)

- gpui_click_save_button.rs
- gpui_metadata_round_trip.rs
- gpui_open_dialog.rs
- gpui_save_dialog.rs

### gpui_selection.rs - Selection, Drag, Resize, Reorder (9 tests)

- gpui_bbox_drag_requires_selection.rs
- gpui_clear_selection.rs
- gpui_multi_select.rs
- gpui_multi_shape_drag.rs
- gpui_select_shape_by_bbox.rs
- gpui_select_tool_noop_paths.rs

### gpui_tooling.rs - Tool Activation, Drawing, Keybindings (6 tests)

- gpui_close_path.rs
- gpui_draw_bezier_auto.rs
- gpui_draw_escape_commits_open_path.rs
- gpui_escape_returns_to_draw.rs
- gpui_keybinding_integration.rs
- gpui_toggle_segment_kind.rs
- gpui_hit_test_service.rs

## Non-GPUI Tests (17 files → mostly unchanged)

### BDD Tests (keep separate, 10 files)

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

### Unit Tests (keep separate or consolidate modestly, 4 files)

- command_editing_helpers.rs - Command editing helper unit tests
- command_editing_unit.rs - Command editing unit tests
- command_unit.rs - Command unit tests

### Integration Tests (keep separate, 1 file)

- golden_round_trip.rs - Golden file round-trip test

## Summary

Before: 56 top-level integration test files (39 GPUI + 17 non-GPUI) After: ~22
top-level integration test files (5 GPUI + 17 non-GPUI) Reduction: 34 files
consolidated (60% reduction in GPUI test targets)
