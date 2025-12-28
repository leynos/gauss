# Execution Plan: 0.1.3 Create Key Context System

**Status**: Complete
**Roadmap reference**: `docs/roadmap.md` section 0.1.3
**Depends on**: 0.1.1 (Action enum), 0.1.2 (Command dispatch) - both complete

## Summary

Create a key context system that binds Actions to keyboard shortcuts via
context-aware key contexts, extending the existing `KEY_CONTEXT` pattern from
Phase 0.

## Design Decisions

### KeyContext as Enum (not strings)

Following the established Action/Command pattern, key contexts are implemented
as an enum rather than raw strings:

- **Exhaustive matching**: All context variants are matched at compile time
- **Type safety**: Prevents typos in context strings
- **Testability**: Context logic testable without GPUI (the GPU-accelerated
  UI framework from Zed)
- **GPUI compatibility**: `AsRef<str>` provides string conversion

### Layered Architecture

```text
Model Layer (GPUI-independent)
+-- KeyContext enum           src/model/key_context.rs
+-- Keystroke type            src/model/keystroke.rs
+-- ActionBinding registry    src/model/keybinding.rs

UI Layer (GPUI-dependent)
+-- GPUI Action bridge        src/ui/action_bridge.rs
+-- bind_keymap() refactor    src/ui/phase0_shell/mod.rs
```

## Implementation Notes

### Context Stacking Limitation

During implementation, it was discovered that GPUI's `.key_context()` method
replaces the previous context rather than stacking multiple contexts. This
means it is not possible to apply both Global and mode-specific contexts to
the same element.

**Resolution**: The view uses only `KeyContext::Global` for now. Mode-specific
shortcuts (like Delete in ManipulateMode) are registered with the Global
context but are handled by checking the current mode in the action handler.
Future work may explore nested elements with different contexts if mode-
specific shortcut scoping becomes necessary.

### GPUI Action Bridge Pattern

GPUI requires struct-based Actions with `#[gpui::Action]` derive. The action
bridge creates thin wrapper structs (e.g., `GpuiUndo`, `GpuiSelectAll`) that
dispatch to model-layer logic. This pattern:

- Keeps model code GPUI-independent for testability
- Centralizes keybinding registration in `register_action_bindings()`
- Allows the model's `Action` enum to remain serializable

## Files Created

| File | Purpose |
|------|---------|
| `src/model/key_context.rs` | KeyContext enum with GPUI string conversion |
| `src/model/keystroke.rs` | Keystroke and Modifiers types |
| `src/model/keybinding.rs` | ActionBinding registry and helpers |
| `src/ui/action_bridge.rs` | GPUI Action structs and registration |
| `docs/users-guide.md` | User documentation for shortcuts |

## Files Modified

| File | Changes |
|------|---------|
| `src/model/mod.rs` | Added module declarations and re-exports |
| `src/ui/mod.rs` | Added action_bridge module and updated init() |
| `src/ui/phase0_shell/mod.rs` | Removed KEY_CONTEXT, use KeyContext::Global |
| `src/ui/phase0_shell/view.rs` | Added action handlers, context-aware render |
| `src/ui/phase0_shell/input.rs` | Added select_all() and deselect_all() |
| `src/ui/phase0_shell/draw/mod.rs` | Made ToolMode pub(crate) |
| `docs/gauss-architecture-design.md` | Added section 7.2 |

## Quality Gates

All gates passed:

- `make check-fmt` - Code formatted correctly
- `make lint` - No clippy warnings or errors
- `make test` - All tests pass
