# Architectural Decision Record (ADR) 001: Clarify Command and document operation (DocOp) roles

## Status

Accepted (2026-01-12): Adopt a layered Command/DocOp model where Commands
express user intent and DocOps remain the atomic mutation layer. DocOps may be
used for transient previews, with final commits captured as Commands.

## Date

2026-01-12

## Context and Problem Statement

The architecture document §7.1 describes Commands as small, composable changes
but does not state how they relate to DocOps. Phase 0 implemented DocOp and
DocChange as the atomic, invertible mutation layer, while newer work adds
Commands for user intent and undo/redo. This ambiguity risks inconsistent
patterns for contributors and makes it unclear how preview operations should be
implemented. Issue #19 tracks the documentation gap.[^issue]

## Terminology

- **DocOp (document operation; plural DocOps)**: An atomic, invertible document
  mutation.
- **DocChange (plural DocChanges)**: An ordered batch of DocOps applied as a
  single unit.
- **Command**: A user-intent operation that is recorded for undo/redo.

## Decision Drivers

- Preserve the existing Phase 0 DocOp investment.
- Keep undo/redo entries aligned with user intent, not every atomic change.
- Allow transient previews for drag interactions without polluting history.
- Minimize churn in the current command implementation.

## Options Considered

### Option A: Command-only (deprecate DocOps)

Commands would be the only mutation layer. DocOps would be removed or relegated
to tests.

### Option B: DocOp-only (promote DocOps to commands)

DocOps would become the user-facing API. Undo/redo would operate at the DocOp
level, and higher-level intent would be reconstructed elsewhere.

### Option C: layered Commands over DocOps (coexist)

Commands remain the unit of undo/redo and user intent. DocOps remain the atomic
mutation layer, optionally batched as DocChange. Commands may emit DocOps or
mutate documents directly for simple cases.

| Topic       | Command-only | DocOp-only | Layered |
| ----------- | ------------ | ---------- | ------- |
| User intent | Strong       | Weak       | Strong  |
| Undo/redo   | Command      | DocOp      | Command |
| Preview use | Hard         | Possible   | Best    |
| Migration   | High churn   | High churn | Low     |

_Table 1: Trade-offs between Command-only, DocOp-only, and layered options._

## Decision Outcome / Proposed Direction

Adopt Option C. Commands represent user intent and are the unit of undo/redo.
DocOps remain the atomic, invertible document operations and can be batched as
DocChange. Command implementations may emit DocOps/DocChanges or mutate the
Document directly for simple cases, but history always records Commands. DocOps
may be used directly for transient previews (for example, drag interactions)
against a scratch document or preview layer; these previews never enter history
and are committed as Commands that may apply DocChange payloads.

Rule of thumb:

- Prefer DocOps/DocChanges when a mutation is already expressed as a DocOp, or
  when multiple atomic edits need batching or reuse across Commands.
- Prefer direct document mutation when the change is simple, local to one
  Command, the inverse is trivial to capture, and expressing it as DocOps would
  add boilerplate without reuse.

  If a Command mutates the Document directly, it must still capture enough data
  to produce a correct inverse.

## Known Risks and Limitations

- Divergence if some commands bypass DocOps without clear guidance.
- Preview implementation details are still undefined.

## Outstanding Decisions

- Define the preview pipeline (scratch document vs preview layer) and how it
  reconciles with the final Command.
- Decide whether DocChange should be exposed as a public preview interface.

[^issue]: <https://github.com/leynos/gauss/issues/19>
