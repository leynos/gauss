# Command Dispatch: Overview

**Parent**: [0-1-2-command-dispatch.md](../0-1-2-command-dispatch.md)

## Summary

Commands are concrete, undoable state changes that bridge user intent (Actions)
to atomic document mutations (DocOps). This task implements the command
dispatch layer described in the architecture document §7.

The relationship between Actions, Commands, and DocOps is:

```text
Action (user intent)       e.g., DeleteSelection
   │
   ▼  dispatch()
Command (undoable mutation) e.g., DeleteSelectionCommand { ids: [...] }
   │
   ▼  apply()
DocChange / DocOp          e.g., RemoveShape { index, shape }
```

## Design Decisions

| Decision               | Choice                                 | Rationale                                                                       |
| ---------------------- | -------------------------------------- | ------------------------------------------------------------------------------- |
| Command representation | **Enum with data**                     | Exhaustive matching, serialization-ready, consistent with Action design         |
| Inverse storage        | **Pre-computed at apply time**         | Undo does not require re-deriving inverse from current state                    |
| Serialization          | **`serde` derives (optional feature)** | Enables macro recording without blocking initial implementation                 |
| Error handling         | **Separated by audience**              | User-facing errors (`UserError`) for UI; internal errors for dispatcher bugs    |
| Module location        | `src/model/command.rs`                 | GPUI (Zed's UI framework)-independent for testability                           |

## Design Rationale

### Why Commands, not just DocOps?

DocOps are low-level, atomic mutations (move anchor, insert shape). Commands
operate at the user-intent level (delete selection, move selected objects). A
single Command may produce multiple DocOps grouped into one undo entry.

Commands capture:

- **Pre-conditions**: Can the command execute? (e.g., is there a selection?)
- **Context**: What data is needed? (e.g., which shape IDs are selected?)
- **Inverse**: How to undo? (store sufficient data at apply time)
- **Name**: Human-readable description for undo/redo menu

### Serialization for Macro Recording

Commands are designed for future macro recording per the architecture document.
The `serde` derives are gated behind a `serde` feature flag to avoid mandatory
serialization overhead. Initial implementation does not require macros to work.

### Error Handling Strategy

Command errors are separated by audience and recovery strategy:

- **User-facing errors** (`UserError`): Semantic issues like empty selections
  or missing shapes. The UI layer catches these and presents appropriate
  feedback (disabled menu items, error messages, accessibility announcements).

- **Internal errors**: Dispatcher bugs and invariant violations. These use
  `panic!()` or `debug_assert!()` to fail fast during development. In release
  builds, defensive checks prevent undefined behaviour while logging indicates
  bugs.

This separation follows the principle: **fail fast for programmer errors,
degrade gracefully for user errors**.

## Command Set (Initial Scope)

The initial command set mirrors the existing Document-kind Actions. See
[snippets/command-enum.rs](snippets/command-enum.rs) for the full definition.

The `DeletedShape` captures everything needed to restore the shape on undo. See
[snippets/deleted-shape.rs](snippets/deleted-shape.rs).

## File Layout

| File                             | Purpose                                            |
| -------------------------------- | -------------------------------------------------- |
| `src/model/command.rs`           | Command enum, CommandInverse, dispatch logic       |
| `src/model/command/error.rs`     | UserError enum (optional: inline in command.rs)    |
| `src/model/mod.rs`               | Add exports for Command types                      |
| `tests/features/command.feature` | BDD scenarios for command dispatch                 |
| `tests/command_bdd.rs`           | BDD step implementations                           |

## Integration Notes

### Relationship to Existing Code

- **Action** (`src/model/action.rs`): Actions represent user intent. The
  `prepare_command` function bridges Actions to Commands.

- **DocOp/DocChange** (`src/model/ops.rs`): Commands can internally use DocOps
  for complex mutations. For simple cases like DeleteShapes, direct document
  manipulation is sufficient.

- **Selection** (`src/model/selection.rs`): Commands require selection state
  to determine which objects to operate on.

- **GPUI undo/redo**: The existing Phase0Shell uses `gpui_component::History`.
  Commands and their inverses integrate with this system.

### Future Extensibility

- `#[non_exhaustive]` allows adding command variants without breaking
  downstream.
- `serde` feature flag enables macro recording when needed.
- Additional commands (MoveShapes, SetStyle, etc.) follow the same pattern.
- Command grouping (transactions) can be added in task 0.3.2.

## Critical Files

| File                                | Purpose                                 |
| ----------------------------------- | --------------------------------------- |
| `src/model/action.rs`               | Action enum (already complete)          |
| `src/model/ops.rs`                  | DocOp/DocChange for reference           |
| `src/model/selection.rs`            | Selection state for command preparation |
| `src/model/document.rs`             | Document structure                      |
| `docs/roadmap.md`                   | Update status on completion             |
| `docs/gauss-architecture-design.md` | Record design decision                  |

## Estimated Scope

~250 lines of new code in `src/model/command.rs`, ~150 lines of unit tests,
~200 lines of BDD tests.
