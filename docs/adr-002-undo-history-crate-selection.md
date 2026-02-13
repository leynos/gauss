# Architectural decision record (ADR) 002: Undo history crate selection

## Status

Accepted (2026-02-12) — `undo_2` selected for document history.

## Date

2026-01-17.

## Context and Problem Statement

Gauss currently relies on `gpui_component::History` for undo/redo. This
introduces a dependency on the GPUI (GPU-accelerated UI) framework that keeps
history out of `EngineState`, forcing the model layer and view layer to remain
split. The architecture document explicitly seeks a GPUI-independent model
layer, so an alternative undo crate is required to keep history inside
`EngineState` without bringing in GPUI.

## Decision Drivers

- Keep `EngineState` GPUI-independent and model-focused.
- Align undo/redo with the Command and document operation (DocOp) architecture.
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

- Release metadata is published on docs.rs alongside a GitHub repository.
  [^undo-versions]

### Option B: `undo_2` (historical undo actions)

`undo_2` stores a historical undo sequence where commands are not lost after
new edits. It records commands and returns a list of actions for the caller to
apply rather than mutating state itself.[^undo2-docs]

Maintenance and sustainability notes:

- Release metadata is published on docs.rs alongside a GitLab repository.
  [^undo2-versions]

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
state by replaying commands, with snapshots reducing replay costs and offering
ways to tune the replay burden.[^gur-docs]

Maintenance and sustainability notes:

- Release metadata is published on docs.rs alongside a GitHub repository.
  [^gur-versions]

| Topic               | undo                                                                 | undo_2                                                              | undo_stack                                                                                              | gur                                                           |
| ------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| Undo model          | Command objects with `UndoCmd`, `Record`, and `History`.[^undo-docs] | Command log returning actions to apply.[^undo2-docs]                | Value stack with `Undoable` restore.[^undo-stack-docs]                                                  | Action replay with snapshots or clone-based state.[^gur-docs] |
| History semantics   | Merging, queueing, and checkpoints.[^undo-docs]                      | Historical undo sequence retains commands after edits.[^undo2-docs] | Linear stack with buffer for continuous edits.[^undo-stack-docs]                                        | Replay with periodic snapshots.[^gur-docs]                    |
| Maintenance signals | Release metadata on docs.rs.[^undo-versions]                         | Release metadata on docs.rs.[^undo2-versions]                       | Release metadata on docs.rs and work in progress (WIP) warning.[^undo-stack-docs][^undo-stack-versions] | Release metadata on docs.rs.[^gur-versions]                   |

_Table 1: Trade-offs between candidate undo crates._

## Decision Outcome / Proposed Direction

`undo_2` is accepted for document history. The spike demonstrated that its
action-iterator API maps cleanly to the existing `Command`/`CommandInverse`
model, and the adapter is small enough to replace if
needed.[^undo2-docs][^undo2-versions]

`undo` (option A) is deferred — `undo_2` meets all current requirements and its
historical undo semantics are acceptable for Gauss.[^undo-docs][^undo-versions]

Selection history remains on `gpui_component::History` because it is a separate
concern with different ownership and lifecycle requirements.

The `undo_2` historical undo model is fashioned after the models used by Cocoa
(`NSUndoManager`), Emacs, and Vim. This provides a familiarity argument — users
of these environments already expect the "never lose work" behaviour. This
strengthens the decision to accept historical undo rather than classical redo
truncation.

A/B testing of undo semantics is possible without changing the adapter's public
interface. `unbuild()` provides classical undo that skips branches, and
`remove_all_undone()` strips all `Undo` markers, giving redo-truncation
semantics. These can be exposed through the adapter if user testing reveals a
preference for classical semantics.

`undo_stack` and `gur` remain deprioritized for the reasons documented
above.[^undo-stack-docs][^gur-docs][^gur-versions]

## Spike Findings

- **Suitability**: `undo_2` successfully wraps command/inverse pairs and
  provides historical undo. The adapter (`DocumentUndoHistory`) is 157 lines
  and provides `record`/`undo`/`redo`/`clear`/`can_undo`/`can_redo`.
- **Historical undo behaviour**: `undo_2` retains all commands after branch
  edits (do A, do B, undo, do C — A, B, C all remain navigable). This differs
  from classical redo truncation, where B would be lost. For Gauss, this is
  acceptable and arguably beneficial — users never lose work.
- **Integration effort**: The adapter required a single stored type
  (`HistoryEntry` containing `Command` + `CommandInverse`). The `undo_2` action
  iterator returns `(Action::Do, &entry)` or `(Action::Undo, &entry)` pairs
  that map cleanly to `command.apply()` and `inverse.apply()`.
- **Error handling**: First error from batched actions is surfaced as a
  formatted string. This preserves the existing `last_history_error` semantics.
- **Performance**: No measurable overhead. The adapter is a thin wrapper.

## Known Risks and Limitations

- Command-based histories require careful inverse capture; memory usage
  depends on the size of command payloads. `undo_2` provides `keep_last(n)`,
  `remove_first(i)`, and `remove_until(pred)` for undo-safe pruning. A
  configurable depth limit is implemented in the adapter via `keep_last()` (see
  `DocumentUndoHistory::record()`).
- Serialization: `Commands<T>` and `CommandItem<T>` derive
  `Serialize`/`Deserialize` by default via the `serde` feature. Persistent undo
  is available once `Command` and `CommandInverse` also implement the serde
  traits (Gauss-side work).
- Replay-oriented approaches such as `gur` may incur higher CPU or
  memory costs for large documents (inference based on its design
  description).[^gur-docs]
- The adapter surfaces only the first error from a batched undo/redo
  action sequence. If future commands produce multiple distinct errors, a
  richer error-collection strategy may be needed.

## Outstanding Decisions

- ~~Decide between `undo` and `undo_2` after a prototype in `EngineState`.~~
  Resolved: `undo_2` accepted after spike implementation and testing.
- ~~Define history pruning policy.~~ Resolved: depth limit implemented
  via `keep_last()` in the adapter. Merge policies (for example, drag edits)
  remain future work.
- Plan user testing for historical undo UX. A/B testing is available
  via `unbuild()` (classical undo skipping branches) and `remove_all_undone()`
  (redo-truncation semantics).
- Specify how history interacts with transient preview operations.
  Remains future work.

## Architectural Rationale

Moving history into `EngineState` preserves the GPUI-independent model layer
and aligns with the Command and document operation (DocOp) architecture, while
keeping the view layer focused on rendering and interaction.

## Vendor or Fork Contingency

Forking `undo_2` remains an option but should be avoided unless a core
requirement cannot be met by the library as-is. The adapter boundary
(`DocumentUndoHistory`, ~165 lines) isolates crate semantics — only
`src/model/history/mod.rs` would change if the underlying crate were replaced.
The crate itself is ~1,600 lines with no transitive dependencies; vendoring is
trivial if upstream becomes unmaintained.

[^undo-docs]:
  undo crate documentation. <https://docs.rs/undo/latest/undo/>.
[^undo-versions]:
  undo versions on docs.rs. <https://docs.rs/crate/undo>.
[^undo2-docs]:
  undo_2 crate documentation. <https://docs.rs/undo_2/latest/undo_2/>.
[^undo2-versions]:
  undo_2 versions on docs.rs. <https://docs.rs/crate/undo_2>.
[^undo-stack-docs]:
  undo_stack crate documentation.
  <https://docs.rs/undo_stack/latest/undo_stack/>.
[^undo-stack-versions]:
  undo_stack versions on docs.rs. <https://docs.rs/crate/undo_stack>.
[^gur-docs]:
  gur crate documentation. <https://docs.rs/gur/latest/gur/>.
[^gur-versions]:
  gur versions on docs.rs. <https://docs.rs/crate/gur>.
