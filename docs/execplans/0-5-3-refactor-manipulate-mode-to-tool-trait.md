# Refactor manipulate mode to SelectTool via Tool trait (0.5.3)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE (2026-03-01)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item `0.5.3` in `docs/roadmap.md` requires refactoring existing
manipulate-mode behaviour into a dedicated `SelectTool` finite-state machine
(FSM) behind the Tool trait boundary introduced by `0.5.1`, after `0.5.2`
completed `PenTool` extraction.

The user-visible goal is behavioural parity with clearer architecture:
selection, drag, and transform-related interaction states are represented
through deterministic tool transitions and emitted `ToolCommand` outputs,
rather than ad hoc shell-local state mutations.

Success is observable when:

- manipulate interactions are routed through `SelectTool` transitions,
  preserving existing behaviour for selection toggle, drag preview, and drag
  commit semantics;
- `Phase0Shell` remains an adapter that feeds input context to tools and
  applies emitted `ToolCommand` values centrally;
- new and updated tests cover happy, unhappy, and edge cases across unit,
  behavioural (`rstest-bdd` v0.5.0), and GPUI layers;
- `docs/gauss-architecture-design.md` records final design decisions for
  `SelectTool` extraction;
- `docs/users-guide.md` is updated where user-visible behaviour or UI wording
  changed;
- `make check-fmt`, `make lint`, and `make test` pass with durable tee logs;
  and
- roadmap entry `0.5.3` is marked done only after implementation, docs sync,
  and full green gates.

## Constraints

- Scope strictly to roadmap item `0.5.3` in `docs/roadmap.md`.
- Preserve user-visible manipulate behaviour unless a deliberate change is
  recorded and documented.
- Keep tool architecture aligned with `docs/gauss-architecture-design.md` §6
  and §20 (tools as deterministic FSMs, command-emission boundary).
- Keep GPUI concerns in the UI adapter (`Phase0Shell`), not in model-layer
  tool transition logic.
- Use `grepai` for intent discovery and `leta` for symbol-level navigation
  during implementation updates.
- Coordinate with an agent team and shared context packs (`pk_3qexgs7k`
  during planning and `pk_zodocaas` during implementation).
- Validate with `rstest` unit tests, `rstest-bdd` behavioural tests, and GPUI
  integration tests, including happy/unhappy/edge cases.
- Record architecture decisions in `docs/gauss-architecture-design.md`.
- Update `docs/users-guide.md` when behaviour/UI details changed.
- Mark roadmap `0.5.3` done only after all required gates succeed.
- Keep files under 400 lines; split modules where required.
- Do not suppress lints except as a last resort with tightly scoped rationale.

## Tolerances (exception triggers)

- Scope: if execution requires more than 24 files or 1,300 net lines changed,
  stop and re-evaluate milestone decomposition.
- Interface: if `Tool`, `ToolInputEvent`, or `ToolCommand` changes force
  simultaneous redesign of `0.5.4` shared hit-testing work, stop and capture
  options in `Decision Log` before proceeding.
- Behaviour drift: if existing manipulate GPUI regression tests fail for
  behavioural reasons (not compile errors), stop and perform parity analysis
  before continuing.
- Test churn: if BDD step rewrites exceed 12 scenario-step modifications,
  stop, and split into an explicit fixture/schema alignment pass.
- Iteration: if gates fail across three repair cycles without reducing failure
  count, stop, and escalate with evidence.
- Environment: if external contention (for example Cargo lock contention across
  worktrees) prevents reliable gate completion, stop, and record blocker
  evidence before closure.

## Risks

- Risk: dual FSM state if drag/selection interaction state remains in
  `Phase0Shell` while `SelectTool` introduces a second state source. Severity:
  high Likelihood: medium Mitigation: move authoritative interaction state
  ownership into `SelectTool` commands/state structures and keep shell as
  adapter only.

- Risk: drag preview behaviour regression if preview updates are not clearly
  represented in transition inputs/outputs. Severity: high Likelihood: medium
  Mitigation: keep deterministic preview pathways explicit and preserve current
  preview-vs-commit semantics with targeted tests.

- Risk: selection history regression (`record_selection_change` timing and
  semantics) when migrating selection decisions behind tool transitions.
  Severity: high Likelihood: medium Mitigation: preserve selection history
  command ordering; add unit and GPUI tests around selection undo/redo.

- Risk: architecture drift between documented `SelectTool` states
  (idle/dragging/marquee/transforming) and implemented states. Severity: medium
  Likelihood: medium Mitigation: document final state model in architecture
  docs and align tests and user docs to the shipped model.

- Risk: test flakiness from mixed GPUI + behavioural suites during refactor.
  Severity: medium Likelihood: low Mitigation: keep deterministic fixtures,
  maintain strict `rstest-bdd` step wording, and isolate GPUI integration
  assertions to wiring-level behaviours.

## Progress

- [x] (2026-03-01 02:25Z) Confirmed branch context:
  `0-5-3-refactor-manipulate-mode-to-tool-trait`.
- [x] (2026-03-01 02:25Z) Loaded required skills and discovery tooling
  (`execplans`, `grepai`, `leta`).
- [x] (2026-03-01 02:25Z) Gathered roadmap, architecture, code, and testing
  context with `grepai` and `leta`.
- [x] (2026-03-01 02:25Z) Ran agent-team discovery (Reimu + Ryu codenames)
  through context pack `pk_3qexgs7k`.
- [x] (2026-03-01 02:25Z) Drafted this ExecPlan at
  `docs/execplans/0-5-3-refactor-manipulate-mode-to-tool-trait.md`.
- [x] (2026-03-01 12:15Z) Implemented `SelectTool` extraction and adapter
  wiring across model and `Phase0Shell` manipulate adapters.
- [x] (2026-03-01 12:40Z) Added/adjusted unit, BDD, and GPUI test coverage for
  parity and edge cases, including reserved-state no-op paths and drag
  interruption.
- [x] (2026-03-01 12:50Z) Updated architecture and users-guide documentation
  and marked roadmap item `0.5.3` done.
- [x] (2026-03-01 13:06Z) Ran and recorded gate evidence (`make check-fmt`,
  `make lint`, `make test`, plus docs gates because docs changed).

## Surprises & Discoveries

- `ToolModeFsm`/`Tool` boundaries are already established in
  `src/model/tool.rs`; `0.5.3` is primarily extraction and parity hardening,
  not net-new interaction design.
- `Phase0Shell` currently bypasses the Tool command path for manipulate drag
  commit
  (`finish_drag(...).and_then(|command| self.apply_command(command).ok())`),
  which is the main architectural seam to refactor.
- Existing manipulate logic is already modularized across `manipulate/mod.rs`,
  `drag.rs`, `selection.rs`, and `hit_test.rs`, reducing extraction risk.
- Existing BDD and GPUI tests provide a usable baseline, but dedicated
  `SelectTool` behavioural scenarios are currently absent.
- The original worker-proposed drag module layout exceeded the 400-line file
  constraint by a narrow margin; reducing comments and adding a focused preview
  test module kept file lengths compliant without changing behaviour.
- Clippy/docstring gates were stricter than expected for test modules
  (`item in documentation is missing backticks`, `no_expect_outside_tests`),
  requiring small hygiene fixes before final green lint.

## Decision Log

- Decision: keep this milestone as a behavioural parity refactor and avoid
  introducing new interaction features (for example marquee UX expansion)
  unless required to preserve architecture consistency. Rationale: roadmap
  `0.5.3` focuses on extraction and state handling, and `0.5.4` already tracks
  shared hit-testing evolution. Date/Author: 2026-03-01 (assistant)

- Decision: plan for explicit `SelectTool` state modelling (`Idle`,
  `Dragging`, `Marquee`, `Transforming`) even if some states are initially thin
  wrappers. Rationale: matches architecture §6.1 intent and prevents repeated
  enum churn in near-term roadmap milestones. Date/Author: 2026-03-01
  (assistant)

- Decision: preserve centralized command application in `Phase0Shell`
  (`handle_tool_input_event` / `apply_tool_commands`) and remove remaining
  direct manipulate-path command application. Rationale: reduces blast radius,
  keeps adapter responsibilities consistent, and preserves known error
  propagation pathways. Date/Author: 2026-03-01 (assistant)

- Decision: keep `SelectToolState::Marquee` and
  `SelectToolState::Transforming` as explicit, tested no-op placeholders in
  this milestone rather than implementing new transform UX behaviour.
  Rationale: satisfies roadmap state-contract intent while preserving
  behavioural parity and avoiding scope creep beyond `0.5.3`. Date/Author:
  2026-03-01 (assistant)

- Decision: add a dedicated preview helper test module
  (`src/model/select_tool_preview_tests.rs`) to cover stale drag-state guards
  separately from FSM transition tests. Rationale: keeps per-file line budgets
  below 400 while expanding edge-case coverage for preview/restore safety.
  Date/Author: 2026-03-01 (assistant)

## Outcomes & Retrospective

Delivered outcome: roadmap item `0.5.3` shipped and validated.

What shipped:

- `SelectTool` FSM extraction under `src/model/select_tool/` with explicit
  pointer input contracts and command emission.
- `ToolInputEvent` / `ToolCommand` extensions for manipulate pointer flows,
  selection updates, and drag preview/restore transitions.
- `Phase0Shell` manipulate adapter refactor to route through tool transitions
  and centralized command application.
- Expanded test coverage across:
  - `rstest` unit tests (`select_tool_tests`, `select_tool_drag_tests`,
    `select_tool_preview_tests`);
  - `rstest-bdd` feature + bindings (`tests/features/select_tool.feature`,
    `tests/select_tool_bdd.rs`);
  - GPUI edge tests (`tests/gpui_escape_returns_to_draw.rs`,
    `tests/gpui_select_tool_noop_paths.rs`).
- Documentation and roadmap synchronization:
  `docs/gauss-architecture-design.md`, `docs/users-guide.md`,
  `docs/roadmap.md`, and this execplan.

Gate evidence:

- `/tmp/fmt-gauss-0-5-3-refactor-manipulate-mode-to-tool-trait.out`
- `/tmp/markdownlint-gauss-0-5-3-refactor-manipulate-mode-to-tool-trait.out`
- `/tmp/nixie-gauss-0-5-3-refactor-manipulate-mode-to-tool-trait.out`
- `/tmp/check-fmt-gauss-0-5-3-refactor-manipulate-mode-to-tool-trait.out`
- `/tmp/lint-gauss-0-5-3-refactor-manipulate-mode-to-tool-trait.out`
- `/tmp/test-gauss-0-5-3-refactor-manipulate-mode-to-tool-trait.out`

Retrospective:

- Keeping shell code as a strict adapter substantially reduced risk during
  extraction and made test layering clearer.
- Splitting tests by concern (FSM transitions vs preview stale guards) improved
  readability and helped satisfy the repository line-budget convention.
- Early targeted test runs from worker lanes shortened the final full-gate
  cycle, with only minor lint hygiene fixes needed during final convergence.

## Context and orientation

Primary roadmap and architecture references:

- `docs/roadmap.md` (`0.5.3` scope and closure criteria)
- `docs/gauss-architecture-design.md` (§6 tool system, §20 immediate steps)
- `docs/using-gpui-and-gpui-component.md` (test split and GPUI integration
  guidance)
- `docs/accesskit-based-accessibility-in-gpui.md` (interaction parity and
  accessibility expectations)
- `docs/rust-testing-with-rstest-fixtures.md`
- `docs/rust-doctest-dry-guide.md`
- `docs/reliable-testing-in-rust-via-dependency-injection.md`
- `docs/rstest-bdd-users-guide.md`

Primary code surfaces for extraction:

- `src/model/tool.rs` (`Tool`, `ToolInputEvent`, `ToolCommand`,
  `ToolModeFsm`, `PenTool`)
- `src/ui/phase0_shell/chrome.rs` (tool event routing, command application)
- `src/ui/phase0_shell/manipulate/mod.rs` (mouse down/move/up manipulate flow)
- `src/ui/phase0_shell/manipulate/drag.rs` (drag start/preview/finish logic)
- `src/ui/phase0_shell/manipulate/selection.rs` (selection decision logic)
- `src/ui/phase0_shell/manipulate/hit_test.rs` and
  `src/ui/phase0_shell/manipulate/handle_drag.rs`

Primary test surfaces:

- Unit: `src/model/tool_tests.rs`, `src/model/pen_tool_tests.rs`, plus new
  `SelectTool`-focused model tests
- BDD: `tests/tool_fsm_bdd.rs`, `tests/features/tool_fsm.feature`, plus new
  `tests/select_tool_bdd.rs` and `tests/features/select_tool.feature`
- GPUI: manipulate and selection flows in existing drag/selection-related tests
  (for example `tests/gpui_multi_shape_drag.rs`,
  `tests/gpui_drag_shape_undo.rs`, `tests/gpui_drag_handle_undo.rs`,
  `tests/gpui_tool_rail.rs`, `tests/gpui_keybinding_integration.rs`)

Documentation targets:

- `docs/gauss-architecture-design.md` (record SelectTool design decisions and
  tool-system status updates)
- `docs/users-guide.md` (only if behaviour/UI wording changed)
- `docs/roadmap.md` (mark `0.5.3` and child bullets done at closure)
- `docs/execplans/0-5-3-refactor-manipulate-mode-to-tool-trait.md`
  (this living plan)

## Plan of work

Stage A: Model the `SelectTool` boundary and state contract

- Add `SelectTool` state types in model space, capturing manipulate interaction
  states required by roadmap/architecture (`Idle`, `Dragging`, `Marquee`,
  `Transforming`).
- Define explicit select-pointer input context types (down/move/up), carrying
  deterministic data required to make transitions testable without GPUI runtime
  dependencies.
- Extend `ToolInputEvent` and `ToolCommand` only as needed to encode selection
  updates, drag-state transitions, preview semantics, and commit commands.
- Keep `ToolModeFsm` responsibilities focused on mode transitions (`Draw` /
  `Manipulate`) and route manipulate pointer flow to `SelectTool`.

Stage B: Refactor `Phase0Shell` manipulate adapter to tool commands

- Replace direct manipulate path mutations and direct `apply_command` calls with
  `SelectTool`-driven transitions + centralized command application.
- Remove redundant shell-owned interaction state where model-level tool state is
  authoritative.
- Preserve existing selection history semantics and error propagation behaviour.
- Keep UI-specific concerns (event decoding, coordinate conversion, adapter
  wiring) in `Phase0Shell`.

Stage C: Preserve parity in selection/drag/transform behaviour

- Port or wrap existing selection and drag helper logic so behaviour remains
  equivalent after extraction.
- Ensure shift-click toggle semantics, non-left-button no-op semantics,
  zero-delta drag semantics, and stale-hit guards remain deterministic.
- Confirm preview updates remain transient and commit operations remain explicit
  `ApplyDocumentCommand` outputs.

Stage D: Add comprehensive test coverage

- Unit (`rstest`): add `SelectTool` transition tests for happy/unhappy/edge
  paths, including no-op and stale-state scenarios.
- Behavioural (`rstest-bdd` v0.5.0): add feature scenarios for selection,
  shape drag commit, anchor/handle drag commit, no-op invalid events,
  shift-toggle semantics, and zero-delta drag.
- GPUI integration: preserve and expand targeted wiring tests for manipulate
  behaviour parity and undo/redo integration.
- Keep tests deterministic and dependency-injected where needed.

Stage E: Documentation and closure

- Update `docs/gauss-architecture-design.md` with final design decisions and
  status for `0.5.3`.
- Update `docs/users-guide.md` for any user-visible behaviour/UI wording changes
  discovered during implementation.
- Update this execplan sections (`Progress`, `Decision Log`,
  `Surprises & Discoveries`, `Outcomes & Retrospective`) as work proceeds.
- Mark `docs/roadmap.md` item `0.5.3` done only after full gate success and
  docs synchronization.

## Validation and acceptance strategy

Implementation must pass all three required quality gates with durable logs:

```sh
set -o pipefail
make check-fmt 2>&1 | tee /tmp/check-fmt-$(get-project)-$(git branch --show).out

set -o pipefail
make lint 2>&1 | tee /tmp/lint-$(get-project)-$(git branch --show).out

set -o pipefail
make test 2>&1 | tee /tmp/test-$(get-project)-$(git branch --show).out
```

When documentation is changed, run docs gates as well:

```sh
set -o pipefail
make fmt 2>&1 | tee /tmp/fmt-$(get-project)-$(git branch --show).out

set -o pipefail
make markdownlint 2>&1 | tee /tmp/markdownlint-$(get-project)-$(git branch --show).out

set -o pipefail
make nixie 2>&1 | tee /tmp/nixie-$(get-project)-$(git branch --show).out
```

Acceptance checklist for closure:

- unit tests verify SelectTool transition behaviour (happy/unhappy/edge);
- BDD scenarios verify manipulate user-level behaviour with strict step wiring;
- GPUI tests verify adapter wiring and undo/redo integration for manipulate
  flows;
- architecture and user docs are synchronized with shipped behaviour;
- roadmap `0.5.3` checklist is marked done only after all gates pass.

## Idempotence and rollback

- Each stage is designed to be re-runnable; if a stage partially fails,
  restore compile/test green before moving forward.
- Preserve incremental commits by stage so regressions can be isolated without
  destructive resets.
- If Tool interface churn threatens downstream milestones (`0.5.4+`), stop and
  escalate with options instead of forcing a broad speculative refactor.

## Artifacts and evidence to capture during implementation

- Gate logs in `/tmp`:
  `check-fmt-...`, `lint-...`, `test-...`, and docs gate logs when applicable.
- Test additions/updates in model, BDD, and GPUI files listed above.
- Documentation diffs for roadmap, architecture, users guide, and this execplan.
- Final closure note in `Outcomes & Retrospective` summarizing delivered
  behaviour and risk follow-ups.

## Revision note

Initial draft authored on 2026-03-01 using `grepai` + `leta` discovery and
agent-team synthesis coordinated through context pack `pk_3qexgs7k`.
Implementation completion updates recorded on 2026-03-01 using agent-team
execution synchronized via context pack `pk_zodocaas`.
