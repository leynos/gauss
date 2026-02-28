# Complete history error enum migration across code, tests, and docs (0.3.5)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE (implementation, documentation, and gates complete).

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item 0.3.5 requires the document history adapter error model to be
fully migrated from `String` to a semantic enum (`HistoryError`) and reflected
consistently across call sites, tests, and documentation. The architectural
intent is explicit in `docs/roadmap.md` section `0.3.5` and architecture
section `7.3.2` / section `20`.

Success is observable when:

- All history/grouping APIs and immediate consumers use typed `HistoryError`
  semantics instead of stringly-typed error transport.
- Unit tests (`rstest`), behavioural tests (`rstest-bdd` v0.5.0), and GPUI
  integration tests validate happy paths, unhappy paths, and relevant edge
  cases for typed history errors.
- `docs/gauss-architecture-design.md`, `docs/users-guide.md`, and
  `docs/roadmap.md` match shipped behaviour.
- `make check-fmt`, `make lint`, and `make test` succeed with durable log
  evidence.

## Constraints

- Scope is roadmap item `0.3.5` only; do not blend unrelated roadmap work.
- Preserve the existing public semantic error type:
  `gauss::model::HistoryError`.
- Keep document history ownership in `EngineState` (already delivered in
  roadmap `0.3.4`); this plan must not regress ownership boundaries.
- Keep model-layer APIs GPUI-independent.
- Ensure behaviour remains deterministic for grouped history operations:
  boundary misuse must not partially commit entries.
- Keep file length under 400 lines per repository guidance.
- Execute quality gates via Make targets with branch-safe `tee` logs:
  `make check-fmt`, `make lint`, `make test`.
- Update user-facing and architecture docs when behaviour wording changes.
- Mark roadmap item `0.3.5` as done only after code, docs, and gates are all
  complete.

## Tolerances (exception triggers)

- Scope trigger: if implementation exceeds 18 files or 900 net changed lines,
  stop and re-check whether extra edits are truly required for 0.3.5.
- API trigger: if completing 0.3.5 requires changing non-history public APIs,
  stop and record options before continuing.
- Dependency trigger: if new crates are required, stop and escalate (this work
  should be achievable with current dependencies).
- Iteration trigger: if full-gate cycles fail three times consecutively without
  reducing failures, pause and reassess with concrete blockers.
- Ambiguity trigger: if requirements conflict between roadmap and architecture
  docs, pause implementation and resolve wording direction before code changes.

## Risks

- Risk: code and docs are currently out of sync (docs still describe
  `String`-based errors in key sections). Mitigation: perform documentation
  sync milestone immediately after typed call site/test updates and before
  final gate run.
- Risk: typed model APIs can still be flattened into `String` at UI and BDD
  boundaries, weakening structured handling. Mitigation: carry `HistoryError`
  until explicit presentation boundary and assert enum variants directly in
  tests.
- Risk: existing tests verify replay failures with string comparisons, which can
  hide variant-level regressions. Mitigation: convert assertions to enum
  variant pattern matching for `UndoReplayFailed` / `RedoReplayFailed`.
- Risk: coverage imbalance between model tests and GPUI tests can leave
  behavioural gaps for boundary cases. Mitigation: add GPUI unhappy-path cases
  for nested begin and undo/redo while group active.

## Agent team strategy

Implementation will use a coordinated agent team (danmaku + beat 'em up
callsigns), with context exchanged through `context_pack`.

- Reimu (code-impact stream): identify and patch residual stringly history error
  pathways and call-site mismatches.
- Haggar (quality stream): lead test migrations and coverage expansion across
  `rstest`, `rstest-bdd`, and GPUI layers.
- Marisa (docs stream): synchronize architecture/user/roadmap documentation and
  completion wording.

Integration model:

- Main agent owns sequencing, patch integration, and gate evidence.
- Streams land small, scoped changes; each change is committed only after
  relevant gates pass.
- `context_pack` remains the shared source for scope boundaries and discovered
  touchpoints.

## Context and orientation

Primary implementation anchors:

- `src/model/history/error.rs` (`HistoryError` variants and display messages).
- `src/model/history/mod.rs` (grouping and replay APIs returning
  `Result<(), HistoryError>`).
- `src/model/engine_state.rs` (public history/grouping API propagated through
  engine state).
- `src/ui/phase0_shell/mod.rs` and `src/ui/phase0_shell/draw/mod.rs`
  (history error storage and presentation path).

Primary test anchors:

- `src/model/history/tests/mod.rs` and
  `src/model/history/tests/grouping.rs` (unit coverage of replay and grouping
  boundaries).
- `tests/features/undo_entry_count.feature` and
  `tests/undo_entry_count_bdd/{main,steps}.rs` (BDD typed/untyped boundary).
- `tests/gpui_command_grouping_undo.rs` and
  `tests/gpui_open_history_reset.rs` (GPUI grouping/history integration).

Primary documentation anchors:

- `docs/roadmap.md` section `0.3.5` (currently unchecked and still references
  `Result<(), String>`).
- `docs/gauss-architecture-design.md` section `7.3.2` (currently describes
  boundary misuse as deterministic `String` errors).
- `docs/users-guide.md` undo/redo section (ensure user-visible behaviour text
  stays accurate; add concise grouped-failure invariant if absent).

Known baseline discovery before implementation:

- Core model/engine history APIs already use `HistoryError`.
- Residual string handling persists in selected UI/test boundaries and stale
  docs.

## Implementation plan

### Milestone 1: Baseline verification and red/green target definition

Confirm the exact current state before editing:

1. Re-verify all history/grouping signatures and call sites that still use
   `String` transport for history-related failures.
2. Catalogue failing or weak assertions that still compare `.to_string()`
   output where variant matching should be used.
3. Capture an initial gate baseline with logs to detect regressions early.

Deliverable: a precise edit list with before-state evidence and zero
speculative changes.

### Milestone 2: Complete typed error propagation at remaining boundaries

Refactor residual history-related `String` transport to preserve enum semantics
until presentation edge.

1. Replace history-adjacent `Result<(), String>` return types where the failure
   domain is `HistoryError`.
2. Reduce premature `.to_string()` flattening in history paths; perform string
   rendering only where user-facing text is required.
3. Keep visible UI behaviour stable while preserving structured error internals.

Deliverable: history/grouping logic flows typed from model through immediate
consumers, with presentation-only string formatting.

### Milestone 3: Unit test hardening with `rstest`

Strengthen model tests to assert semantic variants rather than string content.

1. Convert replay-failure assertions to match
   `HistoryError::UndoReplayFailed` / `HistoryError::RedoReplayFailed` and
   their payloads.
2. Ensure grouping boundary tests cover:
   `GroupAlreadyActive`, `NoActiveGroup`, `UndoWhileGroupActive`, and
   `RedoWhileGroupActive`.
3. Add or refine edge-case coverage for grouped error ordering
   (first-error-wins behaviour).

Deliverable: deterministic variant-level unit assertions for happy/unhappy/edge
paths.

### Milestone 4: Behavioural coverage with `rstest-bdd` v0.5.0

Migrate BDD world/steps to typed grouping errors and expand edge scenarios.

1. Replace `Option<String>` grouping-error state with
   `Option<HistoryError>` in BDD world state where appropriate.
2. Update step definitions to assert variant identity via explicit mappings
   instead of free-form string comparisons.
3. Add scenarios for active-group undo/redo boundary failures and verify history
   invariants remain unchanged.

Deliverable: behaviour-level verification that remains robust against message
text changes while validating semantic error outcomes.

### Milestone 5: GPUI integration coverage

Extend GPUI tests so grouped-history error semantics are validated through UI
integration paths.

1. Add unhappy-path GPUI tests for nested group begin and undo/redo while group
   active.
2. Keep existing happy-path grouped undo/redo coverage intact.
3. Ensure any history-reset/open-path helper signatures align with typed error
   model where applicable.

Deliverable: GPUI tests cover typed happy/unhappy/edge grouped history
behaviour without stringly coupling.

### Milestone 6: Documentation sync and roadmap closure

Synchronize architecture/user/roadmap documents with shipped behaviour.

1. Update architecture section `7.3.2` to describe `HistoryError` enum
   semantics (not pending `String` behaviour).
2. Update `docs/users-guide.md` only for user-visible behaviour implications,
   including grouped-failure invariants if needed.
3. Mark roadmap item `0.3.5` as complete with concise completion evidence once
   gates pass.

Deliverable: documentation and roadmap reflect actual implementation state.

### Milestone 7: Final gate evidence and completion sweep

Run full required gates with durable logs, verify clean status, and finalize
ExecPlan status updates.

Deliverable: passing `check-fmt`, `lint`, `test` logs, updated plan sections,
and readiness for final review.

## Validation and acceptance

Run from repository root with `pipefail` and branch-safe filenames:

```bash
set -o pipefail
branch="$(git branch --show | tr '/' '-')"

make check-fmt | tee "/tmp/check-fmt-$(get-project)-${branch}.out"
make lint | tee "/tmp/lint-$(get-project)-${branch}.out"
make test | tee "/tmp/test-$(get-project)-${branch}.out"
```

Acceptance criteria:

- `make check-fmt` exits `0`.
- `make lint` exits `0`.
- `make test` exits `0`.
- Unit (`rstest`) coverage validates typed history errors for happy, unhappy,
  and edge behaviour.
- Behavioural (`rstest-bdd` v0.5.0) scenarios validate grouped history error
  semantics without stringly internals.
- GPUI tests validate grouped error behaviour and history invariants.
- `docs/gauss-architecture-design.md`, `docs/users-guide.md`, and
  `docs/roadmap.md` are synchronized with implementation.

## Commit plan

- Commit 1: typed boundary propagation and related call-site updates.
- Commit 2: unit + BDD + GPUI test migrations/additions.
- Commit 3: documentation synchronization and roadmap 0.3.5 completion mark.

Each commit is gated before the next commit is created.

## Idempotence and recovery

- If a gate fails, inspect matching `/tmp/*-$(get-project)-${branch}.out`, fix
  minimally, rerun the failed gate, then rerun full gates.
- If cross-branch drift appears while implementing, refresh touchpoints with
  `grepai`/`leta` before continuing.
- If tolerance thresholds are reached, stop and document options in
  `Decision Log` before further edits.

## Progress

- [x] (2026-02-27 13:31Z) Confirmed branch context and loaded `execplans`,
  `leta`, and `grepai` skills/guidance.
- [x] (2026-02-27 13:36Z) Collected roadmap/architecture/testing document
  anchors for 0.3.5.
- [x] (2026-02-27 13:42Z) Created shared `context_pack`
  (`pk_sohquyfy`) for team coordination.
- [x] (2026-02-27 13:45Z) Ran agent-team discovery across code impact, test
  matrix, and documentation drift.
- [x] (2026-02-27 13:52Z) Drafted ExecPlan at requested path.
- [x] (2026-02-27 23:02Z) Received explicit implementation approval.
- [x] (2026-02-27 23:15Z) Milestone 2 complete: typed shell history error
  capture and test getter landed (commit `08e3be0`).
- [x] (2026-02-27 23:58Z) Milestones 3-5 complete: unit, BDD, and GPUI tests
  now assert `HistoryError` variants (commit `772ee95`).
- [x] (2026-02-28 00:31Z) Milestone 6 complete: architecture, users guide, and
  roadmap text synchronized; roadmap item `0.3.5` marked done.
- [x] (2026-02-28 00:35Z) Milestone 7 complete: `make markdownlint`,
  `make nixie`, `make check-fmt`, `make lint`, and `make test` passed with logs
  under `/tmp/*-gauss-0-3-5-evolve-history-error-model-to-enum.out`.

## Surprises & Discoveries

- Discovery: core history/grouping model APIs already expose
  `Result<(), HistoryError>`; roadmap/docs are behind implementation reality.
- Discovery: selected UI and behavioural test boundaries still store or compare
  history errors as strings, reducing semantic signal.
- Discovery: architecture section `7.3.2` still documents pending `String`
  boundary errors, creating truth drift.
- Discovery: running `make fmt` touched one unrelated historical ExecPlan file
  (`docs/execplans/0-3-2-command-grouping-api.md`) via markdown auto-fix.

## Decision Log

- Decision: treat this request as a plan-only phase and keep status in `DRAFT`
  until explicit approval. Rationale: follows execplans approval gate and
  avoids unapproved implementation drift. Date/Author: 2026-02-27 (assistant)

- Decision: include a dedicated milestone for typed error propagation beyond the
  core adapter because structured handling value is lost if immediate
  consumers/tests flatten to strings. Rationale: roadmap goal is structured
  error handling, not only signature substitution. Date/Author: 2026-02-27
  (assistant)

- Decision: require explicit unit + BDD + GPUI unhappy-path additions for
  `UndoWhileGroupActive` and `RedoWhileGroupActive` where absent. Rationale:
  these are semantic enum cases that must be proven across test layers.
  Date/Author: 2026-02-27 (assistant)

- Decision: keep users-guide updates minimal and user-facing.
  Rationale: avoid leaking internal Rust type details into user documentation
  while still documenting behaviour that users observe. Date/Author: 2026-02-27
  (assistant)

- Decision: continue after temporary worker edit-loss anomaly by reapplying the
  test patch set in-place once explicit user confirmation was received.
  Rationale: satisfies safety-stop requirement while preserving momentum.
  Date/Author: 2026-02-27 (assistant)

## Outcomes & Retrospective

Roadmap `0.3.5` is implemented and verified.

Delivered outcome:

- `HistoryError` semantics are preserved at shell boundaries via typed error
  retention, while user-visible status lines remain stable.
- Unit tests now assert `UndoReplayFailed` / `RedoReplayFailed` variants
  directly instead of matching error strings.
- Behavioural tests (`rstest-bdd`) store and assert typed grouping errors,
  including new active-group undo/redo unhappy scenarios.
- GPUI tests now cover grouped unhappy paths for nested begin and active-group
  undo/redo behaviour.
- Documentation was synchronized:
  `docs/gauss-architecture-design.md`, `docs/users-guide.md`, `docs/roadmap.md`.
- Required gates passed:
  `make check-fmt`, `make lint`, `make test`, plus markdown validations
  `make markdownlint` and `make nixie`.
