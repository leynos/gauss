# Architectural decision record (ADR) 001: Command and DocOp relationship

## Status

Accepted (2025-12-30). Commands are the undoable unit of user intent and
compose DocOps for document mutation; DocOps may be used only for transient
previews.

## Date

2025-12-30.

## Context and Problem Statement

Architecture section 7.1 states that Commands should be small and composable
(for example, DeleteShapes, InsertNode, SetTransform). Phase 0 also implemented
DocOps in `src/model/ops.rs` to represent atomic, invertible document
mutations. This leaves ambiguity about whether DocOps are the atomic layer
beneath Commands, whether Commands should wrap DocOp batches, and whether
DocOps can be used directly for preview interactions such as drag.

This ambiguity was raised in issue
[#19](https://github.com/leynos/gauss/issues/19) by
[@leynos](https://github.com/leynos) and relates to PR
[#17](https://github.com/leynos/gauss/pull/17).

## Decision Drivers

- Keep undo/redo and scripting anchored on stable user intent units.
- Preserve low-level, invertible document mutations for tools and tests.
- Support high-frequency previews without polluting history.
- Avoid exposing internal mutation details to scripts and UI layers.

## Options Considered

### Option A: Commands are atomic, DocOps are removed or ignored

Commands directly mutate the document and provide inverses. DocOps are treated
as a temporary PoC artefact or removed from the model layer.

### Option B: DocOps are atomic; Commands wrap DocOp batches

DocOps remain the atomic, invertible mutation layer. Commands collect one or
more DocOps (a DocChange) to represent a single undoable user intent. Commands
remain the API for undo/redo, scripting, and tooling.

### Option C: DocOps are the only public mutation API

Commands are removed and tooling emits DocOps directly, with history tracking
built on DocChange groups.

| Topic               | Option A | Option B    | Option C    |
| ------------------- | -------- | ----------- | ----------- |
| Undo boundary       | Command  | Command     | DocChange   |
| Preview support     | Ad hoc   | DocOp-based | DocOp-based |
| Scripting clarity   | High     | High        | Low         |
| Implementation cost | Low      | Medium      | High        |

_Table 1: Trade-offs between Command/DocOp layering options._

## Decision Outcome / Proposed Direction

We adopt Option B: DocOps are the atomic, invertible document mutations used by
Commands, and Commands are the unit of undo/redo, scripting, and user intent.

- DocOps remain internal to the document model and focus on minimal mutations.
- Commands may apply one or many DocOps (a DocChange) and produce a single
  undo entry.
- Tools emit Commands on commit. DocOps may be applied directly only for
  transient previews (for example, drag feedback) and must be reverted or
  replaced by a Command before the gesture completes.
- Scripts and UI integrations use Actions and Commands, not DocOps.

## Goals and Non-Goals

### Goals

- Clarify layering and responsibilities between Actions, Commands, and DocOps.
- Allow high-frequency preview updates without expanding the undo history.
- Keep scripting and automation on a stable, user-level API.

### Non-Goals

- Immediate refactor of existing Command implementations to emit DocOps.
- Introducing a new history stack or transaction system in this ADR.

## Migration Plan

1. Document the layering in architecture section 7.1 and this ADR.
2. When adding new Commands, prefer implementing document mutations via DocOps
   where suitable.
3. Evaluate whether existing Commands should be refactored to use DocOps once
   the command set grows.

## Known Risks and Limitations

- Divergence between Command logic and DocOp helpers if both exist.
- Preview paths may bypass validation if DocOps are applied without Command-
  level pre-conditions.
- Some tools may need additional state to revert preview DocOps cleanly.

## Architectural Rationale

This decision preserves the Action -> Command pipeline as the external control
plane while keeping DocOps as the low-level, invertible mutation layer. It
supports scripting, undo/redo, and tool FSMs without exposing internal document
mutation details.
