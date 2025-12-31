# Architectural decision record (ADR) 002: GPUI-independent undo history

## Status

Proposed.

## Date

2025-12-30.

## Context and problem statement

The UI layer currently uses `gpui_component::History` to manage undo and redo
for document edits and selection changes. This introduces a GPUI dependency
that forces history stacks to remain in the UI layer instead of the model layer.

The architecture document requires the document and editor state to live in the
engine state (see architecture design section 2.2). Keeping history outside the
engine creates an architectural split that blocks a unified model state, makes
scripting harder, and reduces testability.

Issue [#20](https://github.com/leynos/gauss/issues/20) and PR
[#17](https://github.com/leynos/gauss/pull/17) highlight the need to move undo
history into `EngineState` without pulling GPUI into the model layer.

## Decision drivers

- Preserve the architecture principle that engine state is the single source of
  truth and remains GPUI-independent.
- Keep undo and redo behaviour consistent with current expectations in the UI.
- Minimise migration effort and avoid wide-ranging code changes.
- Keep the history implementation small, testable, and easy to reason about.

## Requirements

### Functional requirements

- Support undo and redo for document edits and selection changes.
- Allow grouping multiple edits into a single undo step.
- Provide a way to query undo and redo availability for UI controls.

### Technical requirements

- No GPUI dependencies in the model layer.
- Compatible with existing change types (`DocChange`, `SelectionChange`).
- Avoid borrowing constraints that complicate storage or ownership.
- Straightforward unit tests and predictable behaviour.

## Options considered

### Option A: Use the `undo` crate

The [`undo` crate](https://docs.rs/undo/) implements the command pattern with
`Edit` commands. It provides a linear `Record` (which discards undone commands
after a new edit) and a tree-based `History` (which keeps all edits in a
branching history). It supports merging edits, checkpoints, and configurable
history limits.

Pros:

- Mature API with both linear and tree-based history options.
- Built-in support for merging edits and limiting history length.

Cons:

- Requires converting changes into command objects and wiring `Edit` traits.
- API does not match `gpui_component::History`, increasing migration effort.
- Extra features (history trees, events) are not currently required.

### Option B: Use the `undo_2` crate

The [`undo_2` crate](https://docs.rs/undo_2/) provides historical undo where
edits are never discarded. When a user edits after undoing, the undo sequence
is merged and replayed instead of truncating redo history. Undo and redo return
action sequences that the application interprets and applies.

Pros:

- Simple, lightweight API that avoids borrowing.
- Keeps all commands, enabling historical traversal of edits.
- Supports merge and splice operations.

Cons:

- Undo semantics differ from the current linear model, which may surprise users.
- Requires an adapter layer to translate action sequences into state changes.
- Makes it harder to match existing `gpui_component::History` behaviour.

### Option C: Re-implement a stand-alone history stack

Implement a small, GPUI-independent history module that matches the current
`gpui_component::History` API (`HistoryItem`, `History<T>`, `push`, `undo`,
`redo`, `undos`, and `redos`). This stack would move into the model layer and
preserve existing semantics.

Pros:

- Matches the existing API, minimising migration and behavioural change.
- Allows `EngineState` to own history without GPUI dependencies.
- Keeps the implementation focused on the project's needs.

Cons:

- Adds maintenance burden and requires careful testing.
- Risks diverging from upstream fixes or features.

| Criterion                      | `undo` crate           | `undo_2` crate       | Stand-alone stack |
| ------------------------------ | ---------------------- | -------------------- | ----------------- |
| GPUI independence              | Yes                    | Yes                  | Yes               |
| Matches current undo semantics | Partial (via `Record`) | No (historical undo) | Yes               |
| API compatibility              | Low                    | Low                  | High              |
| Migration effort               | Medium                 | Medium to high       | Low               |
| Ongoing maintenance            | Low                    | Low                  | Medium            |

_Table 1: Comparison of undo history options._

## Decision outcome / proposed direction

Proceed with a stand-alone, GPUI-independent history stack that matches the
current `gpui_component::History` API. This option preserves current undo
behaviour while allowing history to live inside `EngineState`, aligning with
the architecture principle of a single source of truth.

## Goals and non-goals

Goals:

- Remove the GPUI dependency from history to allow unified model state.
- Preserve the existing undo and redo user experience.

Non-goals:

- Introduce tree-based undo or historical undo semantics at this stage.
- Expand history into a persistence or collaboration feature.

## Known risks and limitations

- A custom history implementation may introduce subtle regressions.
- The stand-alone stack may lag behind external crates in features or fixes.
- Behavioural parity with `gpui_component::History` must be verified.

## Outstanding decisions

- Confirm the expected undo behaviour when new edits follow an undo.
- Define grouping and transaction boundaries for complex operations.
- Decide if history should be persisted or remain purely in-memory.
- Determine history size limits or eviction policies.

## Architectural rationale

This approach restores the model layer as the single source of truth by moving
history into `EngineState` without GPUI dependencies. It keeps the UI as a
projection of state while preserving testability, scripting, and future
front-end flexibility.
