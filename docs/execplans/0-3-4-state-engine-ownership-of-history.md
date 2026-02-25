# Move document history ownership to EngineState (0.3.4)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE (implementation, documentation, and gates complete).

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item 0.3.4 requires document undo/redo history ownership to move from
`Phase0Shell` to `EngineState` so editor state remains engine-owned, as
required by architecture section 20 and the EngineState ownership principle.
Success is observable when:

- `EngineState` owns and operates `DocumentUndoHistory`.
- `Phase0Shell` delegates document history behaviour to `EngineState`.
- `draw/mod.rs`, `chrome.rs`, `file_dialogs.rs`, and helper call sites compile
  and preserve current undo/redo user experience (UX).
- Unit tests (`rstest`), behavioural tests (`rstest-bdd` v0.5.0), and GPUI
  tests cover happy, unhappy, and edge flows for document history.
- `docs/gauss-architecture-design.md`, `docs/users-guide.md`, and
  `docs/roadmap.md` are updated and aligned with shipped behaviour.

## Constraints

- Implement only roadmap scope for `0.3.4` in `docs/roadmap.md` lines 131-135.
- Keep selection history in the view layer (`gpui_component::History`) and move
  only document history ownership.
- Preserve model-layer GPUI independence.
- Update `EngineState` module and struct doc comments to reflect shipped
  ownership (remove future-tense deferment language).
- Update all current `Phase0Shell` document-history call sites:
  `src/ui/phase0_shell/draw/mod.rs`, `src/ui/phase0_shell/chrome.rs`,
  `src/ui/phase0_shell/file_dialogs.rs`, and
  `src/ui/phase0_shell/test_helpers.rs`.
- Preserve user-visible behaviour documented in `docs/users-guide.md` lines
  108-136 (separate document/selection stacks, single undo step per gesture,
  grouped operations).
- Keep files under 400 lines; if a touched file approaches that limit, split
  code into focused modules.
- Run required quality gates with logs:
  `make check-fmt`, `make lint`, `make test`.
- Mark roadmap item `0.3.4` as done only after code, docs, and tests pass.

## Tolerances (exception triggers)

- Scope: if this work exceeds 15 files or 700 changed lines, stop and
  re-evaluate against roadmap scope.
- Trait compatibility: if moving history into `EngineState` requires widespread
  public API breakage due to `Clone`/`Debug`/`PartialEq` constraints, stop and
  choose the least disruptive compatibility approach before proceeding.
- Test churn: if more than 12 existing tests require semantic rewrites (not
  mechanical delegation updates), stop and reassess migration boundaries.
- Gate failures: if full gate runs fail in three consecutive cycles without net
  failure reduction, pause and escalate with concrete blockers.

## Risks

- Risk: `EngineState` currently derives `Clone`, `Debug`, and `PartialEq`, but
  `DocumentUndoHistory` does not implement those traits. Mitigation: explicitly
  decide and document trait strategy before coding: either implement compatible
  traits for history or reduce derives on `EngineState` with minimal blast
  radius.
- Risk: borrow checker conflicts when mutating `document` and history together.
  Mitigation: introduce cohesive `EngineState` history methods that own both
  mutable borrows internally (`apply`, `undo`, `redo`, grouping, clear).
- Risk: UI regressions if button enablement and navigation bindings no longer
  reflect history state. Mitigation: update `chrome` predicates and cover via
  GPUI tests.
- Risk: stale architecture docs after move.
  Mitigation: update design document sections that currently state Phase0Shell
  owns history.

## Spark team strategy

Implementation will use a Spark-style parallel agent team with three streams:

- Spark-Model: engine/model ownership move (`EngineState`,
  `DocumentUndoHistory` integration, trait strategy, unit tests).
- Spark-UI: `Phase0Shell` delegation updates (`draw`, `chrome`, `file_dialogs`,
  test helpers, GPUI call paths).
- Spark-Quality: behavioural/GPUI coverage updates, docs updates, roadmap
  completion, and gate execution evidence.

Each stream lands small, reviewable commits, rebases as needed, and reruns
relevant gates before merge.

## Context and orientation

Primary code anchors:

- `src/model/engine_state.rs` (doc comments and struct ownership boundary).
- `src/model/history/mod.rs` (`DocumentUndoHistory` API surface).
- `src/ui/phase0_shell/mod.rs` (current `document_history` field owner).
- `src/ui/phase0_shell/draw/mod.rs` (record, undo, redo call sites).
- `src/ui/phase0_shell/chrome.rs` (undo/redo button enablement).
- `src/ui/phase0_shell/file_dialogs.rs` (clear history on successful open).
- `src/ui/phase0_shell/test_helpers.rs` (grouping and history length helpers).
- `tests/common/mod.rs` (`read_history_len` helper).
- `tests/gpui_command_grouping_undo.rs` and
  `tests/gpui_navigation_buttons.rs` (document-history GPUI integration).
- `tests/features/command.feature` and `tests/command_bdd.rs` (behaviour tests
  using `rstest-bdd`).

Documentation anchors:

- `docs/roadmap.md` lines 131-135 (required 0.3.4 scope).
- `docs/gauss-architecture-design.md` lines 312-323 and section 20 lines
  1382-1394 (architecture intent and immediate work item).
- `docs/users-guide.md` lines 108-136 (user-visible undo/redo behaviour).

## Implementation plan

### Milestone 1: Move ownership into EngineState

Add a `document_history: DocumentUndoHistory` field to `EngineState` and
initialize it in `new()` / `with_document()`.

Introduce model-level methods that encapsulate document-history interactions to
avoid split mutable borrows in UI code:

- apply document command and record inverse.
- undo / redo document history against `self.document`.
- begin / end grouped history transaction.
- clear document history.
- query methods for `can_undo`, `can_redo`, and history length.

Update `EngineState` module docs and struct docs to reflect current-state
ownership rather than deferred ownership.

Resolve derive compatibility explicitly and keep the public model API coherent.

### Milestone 2: Convert Phase0Shell to delegation

Remove `Phase0Shell::document_history` field and constructor initialization.

Update call sites to delegate through `self.state`:

- `draw/mod.rs`: `apply_command`, `undo_document`, `redo_document`.
- `chrome.rs`: button enablement (`can_undo`/`can_redo`).
- `file_dialogs.rs`: clear document history after successful open.
- `test_helpers.rs`: grouping and history-length helpers.

Keep `last_history_error` behaviour unchanged: failures set status text,
success clears it.

### Milestone 3: Unit tests (`rstest`) for engine ownership

Extend model-level tests to verify `EngineState`-owned history behaviour:

- Happy path: applying command records history, undo restores, redo reapplies.
- Unhappy path: grouped boundary misuse propagates `HistoryError`.
- Edge path: empty history undo/redo are safe no-ops, clear resets stack.

Use fixture/parameterized patterns from
`docs/rust-testing-with-rstest-fixtures.md` (cases and values) to avoid
duplicated setup.

### Milestone 4: Behavioural tests (`rstest-bdd` v0.5.0)

Add or extend behaviour-driven development (BDD) scenarios to exercise
engine-owned document history via observable outcomes, not private internals:

- command application is undoable through engine-owned history.
- empty-history undo remains safe.
- grouping boundary errors keep history unchanged.

Keep scenario registration with `#[scenario]` and `.feature` files, following
the documented Given/When/Then workflow.

### Milestone 5: GPUI tests and integration

Update existing GPUI tests that depend on history helpers to compile against
delegated EngineState ownership.

Ensure happy/unhappy/edge GPUI coverage remains explicit:

- Happy: navigation back/forward still performs document undo/redo.
- Unhappy: failed undo/redo still surfaces `last_history_error`.
- Edge: grouped command transaction still commits one undo step and boundary
  errors preserve history.

Add targeted GPUI coverage if existing tests do not explicitly exercise
clear-on-open behaviour after delegation.

### Milestone 6: Documentation and roadmap completion

Update design docs to match shipped architecture:

- `docs/gauss-architecture-design.md` ownership text in section 5.4.

Review `docs/users-guide.md` undo/redo section and update only where user
visible behaviour or wording changed due to this migration.

Mark roadmap item `0.3.4` done in `docs/roadmap.md` after all gates pass.

## Validation and acceptance

Run from repository root with `pipefail` and branch-safe logs:

```bash
set -o pipefail
project="$(basename "$PWD")"
branch="$(git branch --show-current | tr '/' '-')"

make check-fmt | tee "/tmp/check-fmt-${project}-${branch}.out"
make lint | tee "/tmp/lint-${project}-${branch}.out"
make test | tee "/tmp/test-${project}-${branch}.out"
```

Acceptance criteria:

- All three gates exit successfully.
- New/updated unit tests (`rstest`) pass.
- New/updated behavioural tests (`rstest-bdd` v0.5.0) pass.
- GPUI history integration tests pass.
- Design doc, user guide, and roadmap are in sync with implementation.

## Commit plan

- Commit 1: Engine ownership move and `Phase0Shell` delegation refactor.
- Commit 2: Unit, BDD, and GPUI test updates for happy/unhappy/edge coverage.
- Commit 3: Documentation updates (`design`, `users-guide`, `roadmap`) and
  final ExecPlan progress updates.

Each commit must pass relevant gates before the next commit is created.

## Idempotence and recovery

- If a gate fails, inspect the matching `/tmp/*.out` log file, apply the
  smallest fix, rerun the failed gate, then rerun the full gate stack.
- If rebase or workspace drift occurs, re-check all history call sites before
  continuing to avoid stale-field reintroduction.
- If tolerance thresholds are hit, stop implementation and record options in
  `Decision Log` before proceeding.

## Progress

- [x] (2026-02-25 00:00Z) Loaded `execplans`, `grepai`, and `leta` guidance and
  confirmed repository constraints.
- [x] (2026-02-25 00:00Z) Ran Spark planning team discovery for architecture,
  code touchpoints, and testing obligations.
- [x] (2026-02-25 00:00Z) Mapped concrete call sites where `Phase0Shell` owns
  document history today.
- [x] (2026-02-25 00:00Z) Drafted this ExecPlan in requested path.
- [x] (2026-02-25 00:00Z) Implementation approved and started.
- [x] (2026-02-25 00:00Z) Milestone 1 complete.
- [x] (2026-02-25 00:00Z) Milestone 2 complete.
- [x] (2026-02-25 00:00Z) Milestone 3 complete.
- [x] (2026-02-25 00:00Z) Milestone 4 complete.
- [x] (2026-02-25 00:00Z) Milestone 5 complete.
- [x] (2026-02-25 00:00Z) Milestone 6 complete.
- [x] (2026-02-25 00:00Z) Final gates complete and roadmap item marked done.

## Surprises & discoveries

- Discovery: `EngineState` derives `Clone`, `Debug`, and `PartialEq`
  (`src/model/engine_state.rs`), while `DocumentUndoHistory`
  (`src/model/history/mod.rs`) currently does not. Trait strategy is a first
  implementation decision, not an afterthought.
- Discovery: all document-history references in UI are concentrated in a small
  set of files (`draw`, `chrome`, `file_dialogs`, and `test_helpers`), making
  this migration bounded if API delegation is clean.
- Discovery: `make fmt` touched unrelated markdown files in `docs/execplans/`.
  Recovery: restore unrelated files from `HEAD` before final diff review.
- Discovery: one validation run stalled because a stale `make test` process
  from an earlier session still held Cargo resources. Recovery: terminate only
  the stale process tree, then continue the active gate run.

## Decision log

- Decision: keep scope limited to roadmap 0.3.4 ownership move and defer
  history error model enum work to roadmap 0.3.5. Rationale: avoids conflating
  ownership migration with API redesign. Date/Author: 2026-02-25 (assistant)

- Decision: use Spark team parallel streams (model, UI, quality/docs) for
  implementation execution. Rationale: reduces turnaround while preserving
  focused ownership. Date/Author: 2026-02-25 (assistant)

- Decision: require explicit approval before code implementation begins.
  Rationale: aligns with execplans approval gate and prevents plan drift.
  Date/Author: 2026-02-25 (assistant)

- Decision: keep `document_history` private on `EngineState` and expose explicit
  model-layer methods (`apply_document_command`, `undo_document`,
  `redo_document`, grouping/clear/query APIs) rather than a public field.
  Rationale: protects the engine/view ownership boundary and prevents bypassing
  history bookkeeping. Date/Author: 2026-02-25 (assistant)

- Decision: add GPUI coverage for open-path history reset in a dedicated test
  file (`tests/gpui_open_history_reset.rs`) instead of expanding
  `tests/gpui_open_dialog.rs`. Rationale: keeps files under project size limits
  while adding direct verification for the delegated
  `EngineState::clear_document_history()` path. Date/Author: 2026-02-25
  (assistant)

## Outcomes & retrospective

Roadmap item 0.3.4 is implemented and validated.

Delivered changes:

- `EngineState` now owns `DocumentUndoHistory` and provides model-layer
  document history APIs.
- `Phase0Shell` no longer owns document history and delegates draw/chrome/open
  document-history call sites to `EngineState`.
- Unit tests (`rstest`) cover engine-owned history round-trip, empty-stack
  no-op behaviour, and grouping boundary errors.
- Behaviour tests (`rstest-bdd` v0.5.0) exercise EngineState history
  round-trip and boundary failure paths.
- GPUI tests include grouped command flows and a new open-file regression test
  proving document history and selection reset on successful open.
- Docs are aligned: architecture, users guide, roadmap, and this ExecPlan.

Gate evidence:

- `make fmt` log:
  `/tmp/fmt-gauss-0-3-4-state-engine-ownership-of-history.out`
- `make markdownlint` log:
  `/tmp/markdownlint-gauss-0-3-4-state-engine-ownership-of-history.out`
- `make nixie` log:
  `/tmp/nixie-gauss-0-3-4-state-engine-ownership-of-history.out`
- `make check-fmt` log:
  `/tmp/check-fmt-gauss-0-3-4-state-engine-ownership-of-history.out`
- `make lint` log:
  `/tmp/lint-gauss-0-3-4-state-engine-ownership-of-history.out`
- `make test` log:
  `/tmp/test-gauss-0-3-4-state-engine-ownership-of-history.out`
