# 0.3.1 Audit undo/redo implementation

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

Audit undo/redo to verify that each gesture produces exactly one history entry.
The `undo_2` spike (PR #54) migrated document history to `DocumentUndoHistory`.
This audit adds explicit test coverage proving the single-entry invariant.

After this work:

- Every drag interaction (shape, anchor, handle) produces exactly one undo
  entry on mouse up.
- Every compound tool interaction (draw-mode click, style change, keyboard
  command) produces exactly one undo entry.
- The single-entry invariant is verified by parameterized unit tests, BDD
  scenarios, and GPUI integration tests.
- `DocumentUndoHistory::len()` exposes the realized entry count for test
  assertions.

Success is observable by running the full gates `make check-fmt`, `make lint`,
and `make test` with passing results, and by seeing updated architecture, user
guide, and roadmap documentation.

## Constraints

- No behaviour changes. This is a test-only audit.
- Must not break existing tests.
- Must not modify production logic unless strictly necessary to expose
  `len()` for test assertions.
- Keep markdown and code quality gates green before finishing:
  `make check-fmt`, `make lint`, `make test`.
- Update user-facing and architecture documentation in the same change set.

## Tolerances (exception triggers)

- Scope: if the audit reveals a gesture that does not produce a single entry,
  stop, document the finding, and escalate for direction.
- Iteration: if full `make test` still fails after 3 fix cycles attributable
  to this work, stop and ask for guidance.
- Runtime: if any single validation command exceeds the 300-second command
  limit, split the suite into smaller deterministic chunks and continue.

## Risks

- Risk: a Phase 0 gesture may unexpectedly produce multiple undo entries.
  Severity: medium. Likelihood: low. Mitigation: parameterized tests will
  surface any violation immediately; document findings and escalate if
  discovered.

- Risk: exposing `len()` on `DocumentUndoHistory` may encourage coupling
  between test assertions and internal history state. Severity: low.
  Likelihood: low. Mitigation: `len()` is a simple query method that aligns
  with existing `can_undo`/`can_redo` queries.

## Progress

- [x] (2026-02-14) Stage A: added `len()` and `is_empty()` to
  `DocumentUndoHistory`; five unit tests for entry count behaviour.
- [x] (2026-02-14) Stage B: 11 BDD scenarios covering all command
  variants plus multi-command accumulation.
- [x] (2026-02-14) Stage C: entry count assertions in 8 existing GPUI
  tests; 2 new test files (`gpui_multi_shape_drag_undo`,
  `gpui_close_path_undo`).
- [x] (2026-02-14) Stage D: architecture doc §7.3.1, user guide, roadmap
  updated; execplan created.

## Surprises & discoveries

- No surprises. All Phase 0 interactions already produce single undo
  entries by design. The preview + commit pattern (drag preview updates applied
  directly; `finish_drag()` creates one `Command` on mouse up) inherently
  prevents multi-entry pollution.
- `rustfmt` enforces multi-line function signatures even for short
  parameter lists; single-line style from AGENTS.md applies only to function
  bodies without doc comments.

## Decision log

- `len()` counts realized entries only (via
  `undo_2::Commands::iter_realized().count()`) so that undone entries are not
  included. This matches the user-visible "undo stack depth."
- Extended existing GPUI tests rather than creating parallel duplicates.
  New test files only where existing files did not cover the interaction
  (multi-shape drag undo, close path undo).
- BDD scenarios test `DocumentUndoHistory` directly at the model layer.
  GPUI-level tests use `#[gpui::test]` as the framework does not support BDD
  step definitions.

## Outcomes & retrospective

All acceptance criteria met:

- `DocumentUndoHistory::len()` and `is_empty()` implemented and tested
  (5 unit tests).
- 11 BDD scenarios verify single-entry invariant for all command
  variants.
- 10 GPUI integration tests (8 modified, 2 new) verify entry count for
  drag, draw, anchor edit, segment toggle, reorder, style, multi-shape drag,
  and close path gestures.
- Architecture doc §7.3.1 records audit findings.
- User guide documents the single-undo-step-per-gesture guarantee.
- Roadmap 0.3.1 marked complete.
- `make check-fmt`, `make lint`, `make test` all pass.

## Plan of work

### Stage A: `len()` method and unit tests

Add `len()` and `is_empty()` methods to `DocumentUndoHistory`. Write
parameterized unit tests that apply each command variant and assert the history
length is exactly 1.

Go/no-go: proceed only when unit tests pass for all command variants.

### Stage B: BDD scenarios

Add BDD scenarios to `tests/features/command.feature` that verify each gesture
produces exactly one undo entry.

Go/no-go: proceed only when BDD tests pass and `make test` is green.

### Stage C: GPUI integration tests

Add GPUI integration tests that simulate drag and compound tool interactions
and assert the history contains exactly one entry per gesture.

Go/no-go: proceed only when GPUI tests pass and full `make test` is green.

### Stage D: documentation

Update `docs/gauss-architecture-design.md` section 7.3, `docs/users-guide.md`
Undo/Redo section, and `docs/roadmap.md` to reflect the audit findings. Create
this execplan.

Go/no-go: work is complete only when docs reflect shipped behaviour and all
required gates pass.

## Validation and acceptance

Acceptance criteria:

- `DocumentUndoHistory::len()` and `is_empty()` are implemented and tested.
- Parameterized unit tests verify single-entry invariant for all command
  variants.
- BDD scenarios cover the single-entry invariant.
- GPUI integration tests cover drag and compound tool interactions.
- Architecture doc section 7.3 records the audit findings.
- User guide documents the single-undo-step-per-gesture guarantee.
- Roadmap 0.3.1 checkboxes are marked done.
- `make check-fmt`, `make lint`, and `make test` pass.
