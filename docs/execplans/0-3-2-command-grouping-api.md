# 0.3.2 Command grouping API for document history

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT

## Purpose / big picture

Roadmap item 0.3.2 requires a begin/end transaction API on
`DocumentUndoHistory` so that multi-command operations collapse into a single
undo step. Today each `record()` call creates one history entry, so there is no
first-class grouping surface.

After this work:

- `DocumentUndoHistory` exposes explicit grouping boundaries.
- Commands recorded inside one open group collapse to one undoable step.
- Grouping behaviour is validated at three levels: unit (`rstest`),
  behavioural (`rstest-bdd` v0.5.0), and GPU-accelerated UI (GPUI) integration
  tests.
- Architecture and user docs describe grouping semantics and error handling.
- Roadmap 0.3.2 is marked done only after all gates pass.

Success is observable by passing `make check-fmt`, `make lint`, and
`make test`, plus updated documentation in `docs/gauss-architecture-design.md`,
`docs/users-guide.md`, and `docs/roadmap.md`.

## Agent team

Implementation will run as a small agent team with explicit ownership:

- Explorer A (docs): roadmap + architecture + architecture decision record
  (ADR) constraints and doc updates.
- Explorer B (history code): `DocumentUndoHistory` design, call sites, and
  integration seams.
- Explorer C (tests): unit, `rstest-bdd`, and GPUI coverage patterns.
- Worker A (core API): grouping state machine and model-layer integration in
  `src/model/history/`.
- Worker B (verification/docs): layered tests, docs updates, roadmap completion,
  and final gate evidence.

Workers will treat unrelated file changes as out of scope and leave them
untouched.

## Constraints

- Keep document undo/redo GPUI-independent in the model layer.
  `DocumentUndoHistory` remains in `src/model/history/mod.rs`.
- Preserve existing non-grouped behaviour: simple commands still produce one
  entry and undo/redo semantics must not regress.
- Do not introduce new dependencies for grouping.
- Keep `rstest-bdd` compatibility on v0.5.0 patterns already used in this repo.
- Validate happy, unhappy, and edge paths with unit, behavioural, and GPUI
  tests.
- Record design decisions in `docs/gauss-architecture-design.md`.
- Update `docs/users-guide.md` for user-visible undo/redo behaviour changes.
- Mark `docs/roadmap.md` item 0.3.2 done only after all validation gates pass.

## Tolerances (exception triggers)

- Scope: if grouping implementation requires touching more than 16 files or
  ~900 net lines, stop and reassess slicing.
- Interface: if this requires changing command payload types or public action
  names, stop and escalate.
- Semantics: if grouping forces a behavioural change in selection history,
  stop and escalate.
- Iteration: if full `make test` fails more than 3 fix cycles due to this
  change, stop and ask for direction.
- Runtime: if any gate exceeds the 300-second command ceiling, split into
  deterministic chunks and continue with logged evidence.

## Risks

- Risk: nested or unterminated groups leave history in an inconsistent state.
  Mitigation: explicit begin/end state machine with unhappy-path tests.

- Risk: grouped undo applies only part of a compound operation if replay fails
  mid-batch. Mitigation: reuse existing batch-apply policy in history adapter
  and assert first-error surfacing behaviour.

- Risk: grouping entry count is validated in model tests but not reflected in
  GPUI flows. Mitigation: add a GPUI test that records grouped commands through
  `Phase0Shell` test helpers and verifies one-step undo.

- Risk: documentation drift between architecture, roadmap, and user guide.
  Mitigation: docs updates are a dedicated milestone gated before completion.

## Progress

- [x] (2026-02-18 23:41Z) Gathered roadmap/architecture requirements for 0.3.2
  and confirmed dependency on completed 0.3.1.
- [x] (2026-02-18 23:47Z) Mapped `DocumentUndoHistory` API and all relevant
  references with `grepai` and `leta`.
- [x] (2026-02-18 23:53Z) Audited existing unit, BDD, and GPUI history tests to
  define extension points for grouped-command coverage.
- [ ] Implement grouping API in `DocumentUndoHistory`.
- [ ] Integrate grouped recording path in model-layer usage surfaces.
- [ ] Add/extend unit, behavioural, and GPUI tests for happy/unhappy/edge
  grouping paths.
- [ ] Update architecture design, user guide, ADR follow-up notes, and roadmap.
- [ ] Run full gates and record pass evidence.

## Surprises & Discoveries

- `DocumentUndoHistory` currently has no transaction/grouping API despite
  architecture section 7.3 explicitly calling grouping a required invariant.
- The undo entry-count BDD suite (`tests/undo_entry_count_bdd`) is already
  model-layer focused and is the best behavioural seam for grouped-entry tests.
- GPUI tests already expose history length and command application hooks via
  `Phase0Shell` test helpers, so grouped GPUI coverage can be added without
  introducing UI-only test shims.

## Decision Log

- Decision: Keep this turn plan-only. Implementation will begin only after plan
  approval in this thread. Rationale: explicit ExecPlan request and repo
  history favour plan-first delivery. Date/Author: 2026-02-18 / Codex

- Decision: centre grouping on `DocumentUndoHistory` rather than introducing a
  UI transaction layer. Rationale: roadmap item 0.3.2 explicitly scopes
  grouping to model-layer history. Date/Author: 2026-02-18 / Codex

## Outcomes & Retrospective

Pending implementation.

## Plan of work

### Stage A: group semantics and API contract

Define the grouping contract in `src/model/history/mod.rs` rustdoc:

- `begin_group()` starts a transaction boundary.
- `record()` inside an active group accumulates commands for one future entry.
- `end_group()` commits the grouped sequence as one undoable step.
- Empty groups are no-ops.
- Invalid boundaries (nested begin, end without begin) return deterministic
  errors.

Go/no-go: proceed only when API semantics are documented and compile.

### Stage B: implement grouping in `DocumentUndoHistory`

Implement the internal grouping state in `DocumentUndoHistory` and wire it into
`record()`, `undo()`, `redo()`, `clear()`, and `len()` invariants.

Implementation notes:

- Keep grouped and non-grouped code paths explicit rather than implicit flags.
- Ensure depth limiting (`keep_last(max_depth)`) still applies at commit time.
- Ensure group commit produces one realized entry that undoes/redoes as an
  atomic batch.

Go/no-go: proceed only when new unit tests for grouping core mechanics pass.

### Stage C: integrate grouped recording usage

Integrate grouped recording where model-layer consumers can use it now:

- Extend model-facing helper paths used by behavioural tests
  (`tests/undo_entry_count_bdd/main.rs` and steps) to exercise begin/end
  grouping around multiple command applications.
- Extend Phase0Shell test helper surface if needed so GPUI tests can execute a
  grouped command sequence without bypassing model history semantics.

Go/no-go: proceed only when grouped usage compiles cleanly with unchanged
non-grouped call sites.

### Stage D: layered validation

Add coverage for happy, unhappy, and edge cases.

Unit tests (`rstest`) in `src/model/history/tests/`:

- Happy: two or more commands in one group produce one history entry and one
  undo reverts the whole sequence.
- Unhappy: `end_group` without begin and nested begin fail with stable errors.
- Edge: empty group, clear while grouping (or after group), branch-edit after a
  grouped entry.

Behaviour tests (`rstest-bdd` v0.5.0) in `tests/undo_entry_count_bdd/` and
`tests/features/undo_entry_count.feature`:

- Scenario proving grouped commands produce one undo entry.
- Scenario proving invalid boundary calls keep history unchanged.

GPUI tests (`#[gpui::test]`) under `tests/`:

- Add a grouped-command undo test that performs a compound operation sequence,
  verifies history increases by one entry, and verifies a single undo restores
  the pre-group document state.
- Add one unhappy/edge GPUI assertion path if grouping boundary misuse is
  exposed through test helpers.

Go/no-go: proceed only when all new tests are stable and deterministic.

### Stage E: documentation and roadmap closure

Update docs in the same change set:

- `docs/gauss-architecture-design.md`: add shipped grouping design decisions
  under section 7.3.
- `docs/users-guide.md`: explain grouped compound undo behaviour and any error
  surface users may observe.
- `docs/execplans/adr-002-undo-2-spike.md`: close the deferred grouping
  follow-up note.
- `docs/roadmap.md`: mark 0.3.2 and its sub-bullets done once code/tests/gates
  are complete.

Go/no-go: proceed only when docs accurately match implemented behaviour.

### Stage F: gates and evidence capture

Run required quality gates with `tee` logs using branch-safe filenames:

- `make check-fmt | tee`
  `/tmp/check-fmt-$(get-project)-$(git branch --show | tr '/' '-').out`
- `make lint | tee`
  `/tmp/lint-$(get-project)-$(git branch --show | tr '/' '-').out`
- `make test | tee`
  `/tmp/test-$(get-project)-$(git branch --show | tr '/' '-').out`

If `get-project` is unavailable, substitute a stable project slug manually. If
a gate exceeds runtime limits, split and rerun deterministically.

Completion criteria:

- Grouping API exists on `DocumentUndoHistory` and is used in tests.
- Grouped commands collapse to one undo step across unit, BDD, and GPUI tests.
- Docs and roadmap are updated.
- `make check-fmt`, `make lint`, and `make test` all pass.
