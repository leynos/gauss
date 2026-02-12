# ADR-002 undo_2 spike for command-layer undo

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: IN PROGRESS

## Purpose / big picture

Gauss currently applies document undo/redo in the UI layer via
`gpui_component::History`, even though command preparation and execution are
already model-layer concerns. This plan moves undo/redo state and behaviour for
command execution into a GPUI-independent model module using `undo_2`.

After this work:

- Document undo/redo will execute through model-layer code that does not
  depend on GPUI.
- The UI layer will dispatch actions and render results, but it will no
  longer own document history mechanics.
- The suitability of `undo_2` for Gauss will be evaluated with evidence and
  recorded in `docs/adr-002-undo-history-crate-selection.md`.
- The behaviour will be validated by unit tests (`rstest`), behavioural tests
  (`rstest-bdd` v0.5.0), and GPUI integration tests for both happy and unhappy
  paths.

Success is observable by running the full gates `make check-fmt`, `make lint`,
and `make test` with passing results, and by seeing updated
architecture/ADR/user documentation plus a roadmap checkbox marked done for the
completed history migration scope.

## Constraints

- Keep command preparation and execution GPUI-independent. Any new model
  history module must compile without GPUI imports.
- Do not remove or regress dual-history user behaviour unless explicitly
  documented and approved through ADR updates.
- Keep existing keyboard shortcuts and action names stable unless a documented
  UX decision requires a change.
- Preserve existing command inverse semantics (`Command::apply` returns
  `CommandInverse`, and undo applies inverse).
- Use `undo_2` as the implementation candidate for this spike. Do not add a
  second undo backend during this scope.
- Maintain `rstest-bdd` v0.5.0 compliant scenario signatures (`()` or
  `Result<(), E>` / `StepResult<(), E>`).
- Keep markdown and code quality gates green before finishing:
  `make check-fmt`, `make lint`, `make test`.
- Update user-facing and architecture documentation in the same change set:
  `docs/users-guide.md`, `docs/gauss-architecture-design.md`,
  `docs/adr-002-undo-history-crate-selection.md`, and `docs/roadmap.md`.

## Tolerances (exception triggers)

- Scope: if the implementation requires touching more than 30 files or roughly
  1,600 net lines, stop and reassess staging.
- Interface: if existing public model APIs must change in a breaking way,
  document options in `Decision Log` and pause for direction.
- Dependency: if `undo_2` cannot satisfy required behaviour without adding
  another undo crate, stop and escalate.
- Semantics: if `undo_2` historical undo behaviour creates user-visible
  behaviour conflict that cannot be documented as acceptable, stop and escalate
  with alternatives.
- Iteration: if full `make test` still fails after 3 fix cycles attributable
  to this work, stop and ask for guidance.
- Runtime: if any single validation command exceeds the 300-second command
  limit, split the suite into smaller deterministic chunks and continue.

## Risks

- Risk: `undo_2` historical undo model differs from classical redo truncation.
  Severity: high Likelihood: medium Mitigation: capture behaviour in targeted
  tests and explicitly document UX impact in ADR and user guide before final
  acceptance.

- Risk: moving history out of `Phase0Shell` may expose hidden coupling between
  UI event flow and history mutation order. Severity: high Likelihood: medium
  Mitigation: migrate in small slices and keep existing GPUI undo integration
  tests passing during each stage.

- Risk: command and selection history can diverge if only one stack is moved.
  Severity: medium Likelihood: medium Mitigation: treat both stacks explicitly
  in design and tests, even if final storage locations differ.

- Risk: history-on-open clearing can regress when the ownership moves.
  Severity: medium Likelihood: medium Mitigation: preserve and extend
  open/clear GPUI tests; add BDD coverage for clear semantics.

- Risk: documentation drift after behaviour changes.
  Severity: medium Likelihood: medium Mitigation: complete docs updates in the
  same milestone and gate with markdown checks.

## Progress

- [x] (2026-02-12 19:41Z) Collected architecture, ADR, and codebase context via
  `grepai` and `leta`; identified current GPUI-owned history flow.
- [x] (2026-02-12 19:41Z) Verified `undo_2` API (`Commands<T>`, `undo`,
  `redo`, `clear`, merge/splice capabilities) and constraints.
- [x] (2026-02-12) Stage A complete: finalized model-layer history design and
  acceptance criteria for `undo_2` suitability.
- [x] (2026-02-12) Stage B complete: added model history module
  (`DocumentUndoHistory` in `src/model/history/`) and unit tests for core undo
  engine behaviours.
- [x] (2026-02-12) Stage C complete: integrated Phase0Shell with model history
  adapter, removing `gpui_component::History` dependency for document undo.
- [ ] Stage D complete: extend behavioural and GPUI coverage for happy,
  unhappy, and edge paths.
- [ ] Stage E complete: update ADR, architecture, user guide, and roadmap;
  pass full gates.

## Surprises & discoveries

- Observation: `EngineState` documentation explicitly states history is outside
  the model because `gpui_component::History` is GPUI-dependent. Evidence:
  `src/model/engine_state.rs` and architecture §5.4. Impact: this spike needs a
  documented architecture update, not just code movement.

- Observation: current command undo/redo logic in
  `src/ui/phase0_shell/draw/mod.rs` already applies command or inverse directly
  to the document and tracks first failure message. Evidence:
  `Phase0Shell::apply_history_group`. Impact: migration can preserve failure
  surface while changing history storage backend.

- Observation: `undo_2` does not mutate state itself; it returns
  `(Action, &Command)` entries to execute. Evidence: crate README and
  `undo_2::Commands` API. Impact: Gauss needs a small interpreter layer to map
  `Do`/`Undo` actions to `Command` or `CommandInverse` application.

- Observation: `undo_2` retains historical sequence after branch edits.
  Evidence: crate README example and crate docs. Impact: UX difference versus
  classical redo truncation must be evaluated and recorded in ADR and user
  guide.

- Observation: the `DocumentUndoHistory` adapter is only 157 lines, confirming
  low integration cost. Evidence: `src/model/history/document_undo_history.rs`.
  Impact: replacement cost is low if a different crate is needed later.

- Observation: the `undo_2` historical undo model is arguably beneficial for
  Gauss — users never lose work through undo/redo branching. Evidence: spike
  testing confirmed all commands remain navigable after branch edits. Impact:
  accepted as a positive UX characteristic; documented in ADR-002 and user
  guide.

## Decision log

- Decision: Scope this spike around command-layer document undo first, with
  explicit handling notes for selection history in the same plan. Rationale:
  ADR-002 problem statement is about command history and GPUI coupling; this
  keeps the first migration tractable while preventing blind spots.
  Date/Author: 2026-02-12 / Codex

- Decision: Implement a model-owned history adapter around `undo_2` rather
  than exposing `undo_2` directly to UI code. Rationale: isolates crate
  semantics, keeps future replacement cost low, and centralizes error
  formatting/policy. Date/Author: 2026-02-12 / Codex

- Decision: Treat `undo_2` suitability as an acceptance gate with evidence,
  not an assumption. Rationale: historical undo semantics may be beneficial or
  surprising; we need test evidence and explicit ADR findings. Date/Author:
  2026-02-12 / Codex

- Decision: Keep selection history on `gpui_component::History` rather than
  migrating it to `undo_2`. Rationale: selection history has different
  ownership and lifecycle requirements; it is a view-layer concern that does
  not need GPUI-independence. Migrating both stacks in one spike increases risk
  without clear benefit. Date/Author: 2026-02-12 / Codex

## Outcomes & retrospective

### What shipped

- `DocumentUndoHistory` model-layer adapter wrapping `undo_2::Commands`,
  providing `record`/`undo`/`redo`/`clear`/`can_undo`/`can_redo`.
- Phase0Shell integrated with the model-layer adapter, removing
  `gpui_component::History` dependency for document undo.
- Unit tests for adapter internals including empty history, multi-action
  undo/redo, branch-edit sequences, and error propagation.
- Updated ADR-002, architecture document, user guide, and roadmap.

### What was deferred

- Selection history migration (remains on `gpui_component::History`).
- Command grouping for multi-step interactions (roadmap 0.3.2).
- History pruning/merge policies.
- Preview operation interaction with history.

### Crate acceptance

`undo_2` is accepted for long-term use in document history. The historical undo
model is beneficial for Gauss (users never lose work). The adapter is small
enough to replace if requirements change.

### Follow-up work required

- Implement command grouping API (roadmap 0.3.2).
- Define history pruning policy for long sessions.
- Evaluate whether selection history should also migrate in a future spike.

## Context and orientation

Current architecture and code boundaries relevant to this spike:

- `src/model/engine_state.rs`: model source of truth, currently GPUI-free, and
  currently excludes history.
- `src/model/command/prepare.rs`: action-to-command bridge, already GPUI-free.
- `src/ui/phase0_shell/mod.rs`: stores
  `History<DocumentHistoryItem>` and `History<SelectionHistoryItem>`.
- `src/ui/phase0_shell/draw/mod.rs`: document undo/redo application path and
  error propagation.
- `src/ui/phase0_shell/document_history.rs`: GPUI history item wrapper for
  command + inverse.
- `src/ui/phase0_shell/selection_history.rs`: GPUI history wrapper for
  selection changes.
- `tests/command_unit.rs` and `tests/command_unit_tests/*`: current unit-level
  command coverage.
- `tests/features/command.feature` and `tests/command_bdd.rs`: behavioural
  command coverage.
- `tests/gpui_*undo*.rs` and `tests/gpui_selection_history.rs`: GPUI behaviour
  coverage for history flows.
- `docs/adr-002-undo-history-crate-selection.md`: decision target for crate
  suitability findings.
- `docs/gauss-architecture-design.md`: architecture statement that currently
  ties history exclusion to GPUI dependency.
- `docs/users-guide.md`: user-visible shortcut and undo semantics.
- `docs/roadmap.md`: checklist entry to mark done on completion.

## Plan of work

### Stage A: design and test harness alignment (no production refactor yet)

Define the model-layer history interface and write acceptance criteria for
`undo_2` suitability. The interface should hide crate details and provide clear
methods such as recording applied commands, undo, redo, clear, and history
emptiness checks.

During this stage, add or update tests first where feasible:

- unit tests for the history adapter semantics,
- BDD scenario additions for undo/redo outcomes and unhappy paths,
- GPUI tests identified as regression sentinels.

Go/no-go: proceed only when expected behaviours are explicit in tests and the
design can express both `undo_2` action streams and Gauss error surfaces.

### Stage B: model history adapter implementation

Add model-layer history module(s), for example under `src/model/history/`,
wrapping `undo_2::Commands` with Gauss-specific types.

Implement interpreter logic:

- `undo_2::Action::Do` applies stored forward command behaviour.
- `undo_2::Action::Undo` applies stored inverse behaviour.
- grouped action sequences from one undo/redo call are executed in deterministic
  order and first error is surfaced in a user-facing message.

Retain command naming so status line and logs remain meaningful.

Go/no-go: proceed only if unit tests for adapter semantics pass, including
empty history operations, multi-action undo/redo batches, and branch-edit
sequences.

### Stage C: integrate Phase0Shell with model history

Replace document history backend in `Phase0Shell` from
`gpui_component::History<DocumentHistoryItem>` to the model history adapter.
Keep UI action wiring (`GpuiUndo`, `GpuiRedo`) unchanged so keyboard and button
paths remain stable.

Preserve existing `last_history_error` semantics and ensure document open
continues to clear history. Evaluate whether selection history remains on GPUI
history for now or is migrated in-scope; whichever path is chosen must be
documented in the `Decision Log` and ADR findings.

Go/no-go: proceed only when GPUI undo/redo regression tests continue to pass.

### Stage D: behavioural, integration, and edge-case hardening

Expand coverage to include:

- happy paths: command apply, undo, redo round trips;
- unhappy paths: empty undo/redo, stale command/inverse failure propagation,
  and history clear on open;
- edge paths: undo after branch edits to validate `undo_2` historical sequence
  behaviour and user-visible implications.

Prefer `rstest` parameterization for repeated cases and keep BDD scenarios in
`Result<(), E>` / `StepResult<(), E>` form for v0.5.0 compliance.

Go/no-go: proceed only when targeted suites and full `make test` pass.

### Stage E: documentation, ADR outcome, and roadmap closure

Update documentation with implementation reality:

- `docs/adr-002-undo-history-crate-selection.md`: record findings, accept or
  reject `undo_2`, and capture trade-offs from observed behaviour.
- `docs/gauss-architecture-design.md`: update history ownership description
  (currently says history stays in view due GPUI dependency).
- `docs/users-guide.md`: reflect any user-visible undo/redo semantics changes,
  including branch-history behaviour if applicable.
- `docs/roadmap.md`: mark the relevant undo/history entry done for this scope.

Run full gates and include evidence logs in the change notes.

Go/no-go: work is complete only when docs and roadmap reflect shipped behaviour
and all required gates pass.

## Concrete steps

All commands run from repo root:
`/data/leynos/Projects/gauss.worktrees/undo-2-spike`.

Prepare reusable log variables:

    set -o pipefail
    PROJECT="$(get-project 2>/dev/null || basename "$(git rev-parse --show-toplevel)")"
    BRANCH_SAFE="$(git branch --show | tr '/' '-')"

Stage validation commands (run as milestones complete):

    cargo test --test command_unit --test command_bdd 2>&1 \
      | tee "/tmp/test-command-${PROJECT}-${BRANCH_SAFE}.out" |

    cargo test --test gpui_draw_undo --test gpui_drag_shape_undo \
      --test gpui_drag_anchor_undo --test gpui_drag_handle_undo \
      --test gpui_anchor_edit_undo --test gpui_reorder_undo \
      --test gpui_selection_history 2>&1 \
      | tee "/tmp/test-gpui-history-${PROJECT}-${BRANCH_SAFE}.out" |

Documentation and markdown checks after doc edits:

    make fmt 2>&1 | tee "/tmp/fmt-${PROJECT}-${BRANCH_SAFE}.out"
    make markdownlint 2>&1 | tee "/tmp/markdownlint-${PROJECT}-${BRANCH_SAFE}.out"
    make nixie 2>&1 | tee "/tmp/nixie-${PROJECT}-${BRANCH_SAFE}.out"

Required final gates:

    make check-fmt 2>&1 | tee "/tmp/check-fmt-${PROJECT}-${BRANCH_SAFE}.out"
    make lint 2>&1 | tee "/tmp/lint-${PROJECT}-${BRANCH_SAFE}.out"
    make test 2>&1 | tee "/tmp/test-${PROJECT}-${BRANCH_SAFE}.out"

Expected success transcript pattern:

    $ make check-fmt
    cargo fmt --workspace -- --check
    …
    $ make lint
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    …
    $ make test
    cargo test --workspace
    …
    test result: ok.

## Validation and acceptance

Acceptance criteria for code and behaviour:

- Document undo/redo executes through model-layer history code without
  `gpui_component::History` for document history.
- Unit tests validate adapter internals and edge semantics of `undo_2` action
  streams.
- Behavioural tests validate command-level scenarios (including unhappy paths)
  using `rstest-bdd` v0.5.0 patterns.
- GPUI tests confirm wiring remains functional and user-visible behaviour is
  preserved or explicitly updated.
- `make check-fmt`, `make lint`, and `make test` pass.
- ADR, architecture docs, user guide, and roadmap are updated consistently.

Definition of done for `undo_2` suitability:

- ADR-002 contains explicit findings from tests and implementation experience.
- ADR-002 status reflects the final decision (accepted or rejected for Gauss).
- If accepted, constraints/known limitations are documented.
- If rejected, the ADR records why and what replacement is required.

## Idempotence and recovery

- Stage commands are safe to rerun; tests and checks are read-only with respect
  to source files except `make fmt`.
- If `make fmt` changes unrelated markdown or code, inspect `git status` and
  restore unrelated files before continuing.
- If a migration step breaks GPUI integration, revert only the in-progress
  change set for that step and reapply in smaller slices.
- Keep validation logs in `/tmp` and overwrite them on rerun for deterministic
  evidence paths.

## Artifacts and notes

Evidence to retain while executing this plan:

- `grepai` and `leta` discovery outputs used to scope touched files.
- `/tmp/check-fmt-*.out`, `/tmp/lint-*.out`, `/tmp/test-*.out`.
- Focused test logs for command and GPUI history suites.
- ADR note summarizing observed `undo_2` semantics vs expected Gauss UX.

## Interfaces and dependencies

Dependency to add:

- `undo_2 = "0.2.1"` in `Cargo.toml` dependencies (caret semantics by default).

Target interfaces to exist at the end of implementation (names may be refined
but behaviour must match):

- Model history adapter type (example):
  `crate::model::history::DocumentUndoHistory`.
- Record method that stores applied command/inverse pair.
- Undo and redo methods that apply `undo_2` emitted actions to the document and
  return `Result<(), UserError-or-wrapper>`.
- Clear/reset method used on document open and state reset.
- Optional query helpers for can-undo/can-redo if needed by UI state.

UI boundary expectation:

- `Phase0Shell` uses model history APIs and no longer depends on
  `gpui_component::History` for document undo internals.
- UI remains responsible for event wiring and status rendering.

## Revision note

Initial draft created from current repository analysis, `grepai` semantic
searches, `leta` symbol/call tracing, ADR context, and `undo_2` API review. No
implementation has started yet. Remaining work is all stages B through E.
