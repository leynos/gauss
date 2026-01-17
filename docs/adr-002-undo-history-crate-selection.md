# Architectural decision record (ADR) 002: Undo history crate selection

## Status

Proposed.

## Date

2026-01-17.

## Context and Problem Statement

Gauss currently relies on `gpui_component::History` for undo/redo. This
introduces a GPUI dependency that keeps history out of `EngineState`, forcing
the model layer and view layer to remain split. The architecture document
explicitly seeks a GPUI-independent model layer, so we need an alternative undo
crate that can live in `EngineState` without bringing in GPUI.

## Decision Drivers

- Keep `EngineState` GPUI-independent and model-focused.
- Align undo/redo with the Command and DocOp architecture.
- Support command grouping and merging for continuous edits.
- Maintain predictable redo behaviour without surprising history loss.
- Prefer crates with clear maintenance signals and stable APIs.

## Requirements

### Functional requirements

- Record command-level operations with enough data to compute inverses.
- Support undo and redo with grouping or merging for continuous edits.
- Allow history truncation or checkpoints to manage memory usage.

### Technical requirements

- No UI framework dependency or GPUI coupling.
- Usable from the model layer without requiring GPUI types.
- Evident maintenance and a publicly available repository.

## Options Considered

### Option A: `undo` (Command Pattern stack)

The `undo` crate implements the Command Pattern with an `UndoCmd` trait and
exposes `Record` and `History` stacks. It supports command merging, queueing,
and checkpointing for more advanced history behaviour.[^undo-docs]

Maintenance and sustainability notes:

- Latest release 0.52.0 (2025-03-08) with a GitHub repository.[^undo-versions]

### Option B: `undo_2` (historical undo actions)

`undo_2` stores a historical undo sequence where commands are not lost after
new edits. It records commands and returns a list of actions for the caller to
apply rather than mutating state itself.[^undo2-docs]

Maintenance and sustainability notes:

- Latest release 0.2.1 (2024-12-12) with a GitLab repository.[^undo2-versions]

### Option C: `undo_stack` (value stack with buffering)

`undo_stack` keeps a stack of prior values and restores them via an `Undoable`
trait. It supports buffering continuous changes with `start` and `finish`, but
the documentation describes the crate as a work in progress and not safe for
use.[^undo-stack-docs]

Maintenance and sustainability notes:

- Latest release 0.2.4 (2024-08-11) with a GitHub repository.
  [^undo-stack-versions]

### Option D: `gur` (generative undo via replay)

`gur` defines an `Action` trait and provides `Ur` and `Cur` wrappers that use
snapshots or cloning to manage state. Undoing is described as regenerating old
state by replaying commands, with snapshots reducing replay costs.[^gur-docs]

Maintenance and sustainability notes:

- Latest release 0.2.1 (2023-01-08) with a GitHub repository.[^gur-versions]

| Topic               | undo                                                                 | undo_2                                                              | undo_stack                                                                         | gur                                                           |
| ------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| Undo model          | Command objects with `UndoCmd`, `Record`, and `History`.[^undo-docs] | Command log returning actions to apply.[^undo2-docs]                | Value stack with `Undoable` restore.[^undo-stack-docs]                             | Action replay with snapshots or clone-based state.[^gur-docs] |
| History semantics   | Merging, queueing, and checkpoints.[^undo-docs]                      | Historical undo sequence retains commands after edits.[^undo2-docs] | Linear stack with buffer for continuous edits.[^undo-stack-docs]                   | Replay with periodic snapshots.[^gur-docs]                    |
| Maintenance signals | Latest 0.52.0 (2025-03-08).[^undo-versions]                          | Latest 0.2.1 (2024-12-12).[^undo2-versions]                         | Latest 0.2.4 (2024-08-11) and WIP warning.[^undo-stack-docs][^undo-stack-versions] | Latest 0.2.1 (2023-01-08).[^gur-versions]                     |

_Table 1: Trade-offs between candidate undo crates._

## Decision Outcome / Proposed Direction

Shortlist `undo` and `undo_2` for a focused spike in the model layer because
they are command-oriented and have the most recent releases of the options
reviewed.[^undo-docs][^undo2-docs][^undo-versions][^undo2-versions]
Deprioritise `undo_stack` due to its work-in-progress warning and value-stack
focus, and deprioritise `gur` because its replay-and-snapshot model implies
replay overhead for large documents (inference based on its design
description), and it has not shipped a release since 2023.
[^undo-stack-docs][^gur-docs][^gur-versions]

## Known Risks and Limitations

- Command-based histories require careful inverse capture; memory usage depends
  on the size of command payloads.
- `undo_2` keeps a historical sequence of commands after edits, which may need
  a clear policy for pruning or user-facing behaviour.[^undo2-docs]
- Replay-oriented approaches such as `gur` may incur higher CPU or memory costs
  for large documents (inference based on its design description).[^gur-docs]

## Outstanding Decisions

- Decide between `undo` and `undo_2` after a prototype in `EngineState`.
- Define history pruning and merge policies (for example, drag edits).
- Specify how history interacts with transient preview operations.

## Architectural Rationale

Moving history into `EngineState` preserves the GPUI-independent model layer
and aligns with the Command and DocOp architecture, while keeping the view
layer focused on rendering and interaction.

[^undo-docs]:
  undo crate documentation. <https://docs.rs/undo/latest/undo/>.
[^undo-versions]:
  undo versions on docs.rs. <https://docs.rs/crate/undo/0.7.0>.
[^undo2-docs]:
  undo_2 crate documentation. <https://docs.rs/undo_2/0.2.1/undo_2/>.
[^undo2-versions]:
  undo_2 versions on docs.rs. <https://docs.rs/crate/undo_2/0.2.1>.
[^undo-stack-docs]:
  undo_stack crate documentation.
  <https://docs.rs/undo_stack/0.2.4/undo_stack/>.
[^undo-stack-versions]:
  undo_stack versions on docs.rs. <https://docs.rs/crate/undo_stack/0.2.4>.
[^gur-docs]:
  gur crate documentation. <https://docs.rs/gur/0.2.1/gur/>.
[^gur-versions]:
  gur versions on docs.rs. <https://docs.rs/crate/gur/0.2.1>.
