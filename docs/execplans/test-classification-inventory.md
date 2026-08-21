# Test classification inventory for test move execplan

## Purpose

This document classifies the current root `gauss` integration-test targets by
test style. It is the inventory used by the test move and consolidation plans.

### Current authoritative inventory

The current inventory is derived from root `gauss` test targets in Cargo
metadata, rather than from filename globs:

```bash
cargo metadata --no-deps --format-version 1 |
  jq -r '.packages[] | select(.name == "gauss") | .targets[] |
    select(.kind | index("test")) | .name'
```

Classify each target's source by its test declaration: a scenario with
`GpuiHarness` is harness-backed GPUI BDD; a direct `#[gpui::test]` target is
raw structural GPUI; the remaining scenario targets are non-GPUI BDD; and the
remaining targets are other integration tests.
`make check-integration-test-inventory` automates this check.

_Table: Current authoritative root `gauss` integration-test inventory derived
from Cargo metadata._

| Category                                                 | Count |
| -------------------------------------------------------- | ----: |
| Root `gauss` integration-test targets (`cargo metadata`) | 51    |
| Harness-backed GPUI BDD (scenario with `GpuiHarness`)    | 25    |
| Raw structural GPUI (direct `#[gpui::test]`)             | 16    |
| Non-GPUI BDD                                             | 5     |
| Other integration targets                                | 5     |
| GPUI targets (harness-backed + raw structural)           | 41    |

<!-- integration-test-inventory: total=51 harness_gpui_bdd=25
raw_structural_gpui=16 non_gpui_bdd=5 other_integration=5 gpui_target=41 -->

## Historical test-move classification (2026-03)

The remaining sections preserve the pre-move ownership classification. They do
not supersede the current Cargo-target inventory above.

### Classification criteria

- **Pure model**: Tests that only exercise `gauss-core` functionality (document,
  commands, tools, history)  without GPUI
- **Pure SVG**: Tests that only exercise `gauss-svg` functionality (import,
  export, metadata) without GPUI
- **App integration**: Tests that require GPUI, window interaction, or
  cross-crate integration

### Test inventory

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

- `tests/gauss_model_ops_bdd.rs` - Model operations BDD (tests both model and
  SVG)
- `tests/golden_round_trip.rs` - SVG round-trip validation
- `tests/metadata_round_trip_bdd.rs` - Metadata preservation in SVG
- `tests/resource_store_bdd.rs` - Resource store (tests both model and SVG)
- `tests/web_ready_export_bdd.rs` - Web-ready SVG export

Supporting directories:

- `tests/golden/` - Golden test fixtures

### App integration tests → Stay in `tests/`

All GPUI-dependent tests and the non-GPUI app-layer accessibility tests stay in
the app crate:

#### GPUI accessibility tests

- `tests/gpui_shell_a11y_service.rs` - Shell accessibility integration

Supporting directories:

- `tests/a11y_service_bdd/` - AccessKit BDD fixtures

#### Non-GPUI accessibility tests

- `tests/a11y_service_bdd.rs` - AccessKit service behaviour
- `tests/a11y_service_routing_bdd.rs` - AccessKit routing behaviour

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
- `tests/gpui_draw_undo_bdd.rs` - Draw-mode undo and redo behaviour
- `tests/gpui_history_multi_shape_drag_undo.rs` - Multi-shape drag with undo
- `tests/gpui_history_open_history_reset.rs` - History reset on open
- `tests/gpui_history_reorder_undo.rs` - Shape reordering with undo
- `tests/gpui_history_selection_history.rs` - Selection history

#### Selection tests

- `tests/gpui_selection_bbox_drag_requires_selection.rs` - Bbox drag behaviour
- `tests/gpui_selection_clear_selection.rs` - Selection clearing
- `tests/gpui_selection_multi_select.rs` - Multi-selection
- `tests/gpui_selection_multi_shape_drag.rs` - Multi-shape dragging
- `tests/gpui_selection_select_shape_by_bbox.rs` - Bbox selection
- `tests/gpui_selection_select_tool_noop_paths.rs` - Select tool edge cases

#### Shell/UI tests

- `tests/gpui_shell_canvas_layout.rs` - Canvas layout
- `tests/gpui_shell_chrome_bdd.rs` - Shell chrome behaviour
- `tests/gpui_shell_chrome_layout.rs` - Chrome layout
- `tests/gpui_shell_mode_indicator.rs` - Mode indicator
- `tests/gpui_shell_navigation_buttons.rs` - Navigation buttons
- `tests/gpui_shell_resize_borders.rs` - Resize borders
- `tests/gpui_shell_style_controls.rs` - Style controls
- `tests/gpui_shell_tool_rail.rs` - Tool rail
- `tests/gpui_shell_viewport_input.rs` - Viewport input
- `tests/gpui_shell_window_controls.rs` - Window controls

#### Internationalization tests

- `tests/gpui_i18n_module.rs` - Localized shell status text

#### Test-support API tests

- `tests/test_support_const_apis.rs` - Compile-time GPUI test-support APIs

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

### Historical test-move summary

**To move:**

- 10 pure model tests → `crates/gauss-core/tests/`
- 5 pure SVG tests → `crates/gauss-svg/tests/`

**To stay:**

- App integration tests → `tests/` (app crate)

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
