# Test classification inventory for test move execplan

## Purpose

This document classifies all current integration tests by their owning crate to
guide the test move implementation in `build-time-move-model-and-svg-tests.md`.

## Classification criteria

- **Pure model**: Tests that only exercise `gauss-core` functionality (document,
  commands, tools, history)  without GPUI
- **Pure SVG**: Tests that only exercise `gauss-svg` functionality (import,
  export, metadata) without GPUI
- **App integration**: Tests that require GPUI, window interaction, or
  cross-crate integration

## Test inventory

### Pure model tests → Move to `crates/gauss-core/tests/`

- `tests/action_bdd.rs` - Action dispatch (model layer)
- `tests/command_bdd.rs` - Command system (model layer)
- `tests/command_editing_helpers.rs` - Helper for command tests
- `tests/command_editing_unit.rs` - Command editing unit tests
- `tests/command_unit.rs` - Command unit tests
- `tests/hit_test_bdd.rs` - Hit testing (model layer)
- `tests/pen_tool_bdd.rs` - Pen tool state machine (model layer)
- `tests/select_tool_bdd.rs` - Select tool state machine (model layer)
- `tests/stable_id_bdd.rs` - Stable ID generation (model layer)
- `tests/tool_fsm_bdd.rs` - Tool FSM (model layer)

Supporting directories:

- `tests/command_unit_tests/` - Command unit test fixtures
- `tests/command_editing_unit_helpers/` - Command editing helpers
- `tests/features/` - BDD feature files (if model-specific)

### Pure SVG tests → Move to `crates/gauss-svg/tests/`

- `tests/gauss_model_ops_bdd.rs` - Model operations BDD (tests both model and SVG)
- `tests/golden_round_trip.rs` - SVG round-trip validation
- `tests/metadata_round_trip_bdd.rs` - Metadata preservation in SVG
- `tests/resource_store_bdd.rs` - Resource store (tests both model and SVG)
- `tests/web_ready_export_bdd.rs` - Web-ready SVG export

Supporting directories:

- `tests/golden/` - Golden test fixtures

### App integration tests → Stay in `tests/`

All GPUI-dependent tests stay in the app crate:

#### Accessibility tests

- `tests/a11y_service_bdd.rs` - AccessKit service (requires GPUI)
- `tests/a11y_service_routing_bdd.rs` - AccessKit routing (requires GPUI)
- `tests/gpui_shell_a11y_service.rs` - Shell accessibility integration

Supporting directories:

- `tests/a11y_service_bdd/` - AccessKit BDD fixtures

#### File I/O tests

- `tests/gpui_file_io_click_save_button.rs` - Save button interaction
- `tests/gpui_file_io_metadata_round_trip.rs` - Metadata via file dialogs
- `tests/gpui_file_io_open_dialog.rs` - Open dialog integration
- `tests/gpui_file_io_save_dialog.rs` - Save dialog integration

#### History tests

- `tests/gpui_history_anchor_edit_undo.rs` - Anchor editing with undo
- `tests/gpui_history_close_path_undo.rs` - Path closing with undo
- `tests/gpui_history_command_grouping_undo.rs` - Command grouping with undo
- `tests/gpui_history_drag_anchor_undo.rs` - Anchor dragging with undo
- `tests/gpui_history_drag_handle_undo.rs` - Handle dragging with undo
- `tests/gpui_history_drag_shape_undo.rs` - Shape dragging with undo
- `tests/gpui_history_draw_undo.rs` - Drawing with undo
- `tests/gpui_history_multi_shape_drag_undo.rs` - Multi-shape drag with undo
- `tests/gpui_history_open_history_reset.rs` - History reset on open
- `tests/gpui_history_reorder_undo.rs` - Shape reordering with undo
- `tests/gpui_history_selection_history.rs` - Selection history

#### Selection tests

- `tests/gpui_selection_bbox_drag_requires_selection.rs` - Bbox drag behavior
- `tests/gpui_selection_clear_selection.rs` - Selection clearing
- `tests/gpui_selection_multi_select.rs` - Multi-selection
- `tests/gpui_selection_multi_shape_drag.rs` - Multi-shape dragging
- `tests/gpui_selection_select_shape_by_bbox.rs` - Bbox selection
- `tests/gpui_selection_select_tool_noop_paths.rs` - Select tool edge cases

#### Shell/UI tests

- `tests/gpui_shell_canvas_layout.rs` - Canvas layout
- `tests/gpui_shell_chrome_layout.rs` - Chrome layout
- `tests/gpui_shell_mode_indicator.rs` - Mode indicator
- `tests/gpui_shell_navigation_buttons.rs` - Navigation buttons
- `tests/gpui_shell_quit_button.rs` - Quit button
- `tests/gpui_shell_resize_borders.rs` - Resize borders
- `tests/gpui_shell_style_controls.rs` - Style controls
- `tests/gpui_shell_tool_rail.rs` - Tool rail
- `tests/gpui_shell_viewport_input.rs` - Viewport input
- `tests/gpui_shell_window_controls.rs` - Window controls

#### Tooling tests

- `tests/gpui_tooling_close_path.rs` - Path closing tool integration
- `tests/gpui_tooling_draw_bezier_auto.rs` - Bezier auto-drawing
- `tests/gpui_tooling_draw_escape_commits_open_path.rs` - Escape commits path
- `tests/gpui_tooling_escape_returns_to_draw.rs` - Escape returns to draw mode
- `tests/gpui_tooling_hit_test_service.rs` - Hit test service integration
- `tests/gpui_tooling_keybinding_integration.rs` - Keybinding integration
- `tests/gpui_tooling_toggle_segment_kind.rs` - Segment kind toggling

Supporting directories:

- `tests/common/` - Common test helpers (likely stays)

#### Undo entry count tests

- `tests/undo_entry_count_bdd/` - BDD tests for undo entry counting (multi-file)

## Summary

**To move:**

- 10 pure model tests → `crates/gauss-core/tests/`
- 5 pure SVG tests → `crates/gauss-svg/tests/`

**To stay:**

- 41 GPUI-dependent tests → `tests/` (app crate)

## Notes

- All tests currently compile and pass with
  `cargo test --workspace --all-targets --all-features`
- The `test-support` feature must remain enabled for integration tests to access
  Phase0Shell test helper methods
- Common test helpers in `tests/common/` may need to be refactored into
  `crates/test_support` or duplicated as needed

## Regenerating this inventory

To verify or update this inventory after adding/removing/renaming tests, run:

```bash
# List all test files by location
echo "=== gauss-core tests ===" && ls crates/gauss-core/tests/*.rs 2>/dev/null | wc -l && \
echo "=== gauss-svg tests ===" && ls crates/gauss-svg/tests/*.rs 2>/dev/null | wc -l && \
echo "=== App crate tests ===" && ls tests/*.rs 2>/dev/null | wc -l

# Detailed listing
echo -e "\n=== gauss-core test files ===" && ls crates/gauss-core/tests/*.rs 2>/dev/null && \
echo -e "\n=== gauss-svg test files ===" && ls crates/gauss-svg/tests/*.rs 2>/dev/null && \
echo -e "\n=== App crate test files ===" && ls tests/*.rs 2>/dev/null
```

The test files listed in this document should match the output of these
commands.
