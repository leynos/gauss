# Define Tool trait and FSM command boundary (0.5.1)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE (2026-02-26)

No `PLANS.md` exists in this repository.

## Purpose / Big Picture

Roadmap item `0.5.1` in `docs/roadmap.md` defines the boundary that unlocks all
later tool work (`0.5.2` pen FSM extraction, `0.5.3` select FSM extraction, and
`0.5.4` shared hit-testing).

After this work, tool behaviour will be expressed by a model/controller trait
that treats tools as deterministic finite-state machines (FSMs) driven by input
events and producing command intents. The user-visible behaviour for Draw and
Manipulate modes must remain unchanged in this step, but the architecture
boundary must become explicit and test-backed.

Success is observable when:

- a `Tool` trait contract exists and encodes input-event-driven transitions;
- tool handling emits command intents (or command emission plans), not direct
  document mutations;
- current Draw/Manipulate mode flows still pass existing integration tests;
- new unit, behavioural (`rstest-bdd`), and GPU-accelerated UI (GPUI) tests
  prove happy and unhappy
  paths for the trait boundary;
- architecture and user documentation describe the updated boundary; and
- roadmap entry `0.5.1` (and its two child bullets) is marked done only after
  full quality gates pass.

## Constraints

- Scope strictly to roadmap item `0.5.1` in `docs/roadmap.md:156-160`.
- Follow architecture requirements in `docs/gauss-architecture-design.md` §6.1,
  §6.3, and §20.
- Keep model and controller logic GPUI-independent where possible, consistent
  with `docs/using-gpui-and-gpui-component.md:304-317`.
- Preserve existing user-visible Draw/Manipulate behaviour in this task.
- Keep Rust modules under 400 lines; split files where needed.
- Use `rstest` for unit tests, `rstest-bdd` v0.5.0 for behavioural tests, and
  `#[gpui::test]` only for wiring/platform interaction verification.
- Do not silence lints unless narrowly scoped with clear rationale.
- Update `docs/gauss-architecture-design.md` with design decisions taken for
  the Tool trait boundary.
- Update `docs/users-guide.md` for any behaviour, shortcut, or mode semantics
  changes users must know.
- Mark roadmap item `0.5.1` as done only after implementation is complete and
  `make check-fmt`, `make lint`, and `make test` all pass.

## Tolerances (Exception Triggers)

- Scope size: if the change needs more than 18 files or 950 net lines,
  stop and re-evaluate decomposition.
- API blast radius: if trait introduction requires changes outside
  `src/model`, `src/ui/phase0_shell`, `src/ui/action_bridge`, tests, and docs,
  stop and re-evaluate.
- Behaviour drift: if any existing GPUI interaction test changes behaviour
  outside tool-FSM boundary semantics, stop and investigate before proceeding.
- Dependency changes: if a new runtime crate is required, stop and seek
  direction.
- Test instability: if full gates fail more than three repair cycles without
  reducing failures, stop and escalate with findings.

## Risks

- Risk: introducing a trait too coupled to GPUI events.
  Mitigation: define model-level input event enums and tool outputs in
  `src/model`, then adapt GPUI events at the view/controller edge.
- Risk: hidden direct state mutation paths remain in view handlers.
  Mitigation: audit known mutation points in `src/ui/phase0_shell/input.rs`,
  `view.rs`, `draw/mod.rs`, and `manipulate/mod.rs`, and route through the new
  trait boundary.
- Risk: command emission boundary is underspecified for preview vs commit.
  Mitigation: explicitly model non-persistent preview transitions versus
  command-emitting commit transitions in the trait API and tests.
- Risk: regression in keybinding-driven tool switching (P/V/Escape/Tab).
  Mitigation: extend existing GPUI keybinding and mode tests rather than
  replacing them.

## Progress

- [x] (2026-02-25) Grounded branch/worktree state and confirmed this is a
  roadmap `0.5.1` implementation task on branch `0-5-1-tool-trait`.
- [x] (2026-02-25) Performed discovery with Spark explorer team:
  roadmap scope, architecture boundary, and testing/doc requirements.
- [x] (2026-02-25) Used `grepai` and `leta` to map current tool symbols,
  mutation points, and test coverage.
- [x] (2026-02-25) Drafted this ExecPlan at
  `docs/execplans/0-5-1-tool-trait.md`.
- [x] (2026-02-25) Implemented `Tool`, `ToolInputEvent`, `ToolCommand`,
  `ToolTransition`, and `ToolModeFsm` in `src/model/tool.rs`, with test module
  extraction to `src/model/tool_tests.rs`.
- [x] (2026-02-25) Refactored Phase 0 shell tool handling to use
  `handle_tool_input_event` and `apply_tool_commands` so tool transitions emit
  commands instead of mutating state ad hoc.
- [x] (2026-02-25) Added/extended unit tests (`rstest`) for happy, unhappy, and
  edge transitions, including `ActivateDraw` without edge override, `ActivateDraw`
  while already in draw, and `ClosePathCommitted` outside draw.
- [x] (2026-02-25) Added/extended behavioural tests (`rstest-bdd` v0.5.0) in
  `tests/tool_fsm_bdd.rs` and `tests/features/tool_fsm.feature` for the same
  happy/unhappy/edge transition coverage.
- [x] (2026-02-25) Added/extended GPUI integration coverage, including
  `tests/gpui_keybinding_integration.rs`, while preserving existing behaviour.
- [x] (2026-02-25) Updated architecture and user documentation, and marked
  roadmap item `0.5.1` done in `docs/roadmap.md`.
- [x] (2026-02-26) Added post-rebase edge-case coverage for
  `ActivateDraw { edge_mode: Some(current_mode) }` no-op/idempotent behaviour
  and a GPUI integration test for Tab in Manipulate mode.
- [x] (2026-02-26) Closed the tool-command error reporting gap by routing
  `apply_tool_commands` failures through `last_history_error`.
- [x] (2026-02-26) Ran full quality gates and captured passing logs:
  `/tmp/check-fmt-gauss-0-5-1-tool-trait.out`,
  `/tmp/lint-gauss-0-5-1-tool-trait.out`,
  `/tmp/test-gauss-0-5-1-tool-trait.out`.

## Surprises & Discoveries

- Current architecture text already states the desired boundary exactly:
  tools are FSMs and emit Commands
  (`docs/gauss-architecture-design.md:606-640`), so this task is mostly
  codifying an existing contract in code.
- The direct tool-state mutation paths discovered during planning were replaced
  with a single command-emitting path in `Phase0Shell`:
  `handle_tool_input_event` + `apply_tool_commands`.
- `ToolCommand::ApplyDocumentCommand` needed `Box<Command>` to satisfy
  `clippy::large_enum_variant` without weakening lint policy.
- Keeping `src/model/tool.rs` under the repository line-limit policy required
  splitting tests into `src/model/tool_tests.rs`.
- Behavioural tests initially missed `ClosePathCommitted` and singular/plural
  command-step wording; explicit scenarios and step aliases closed that gap.
- Cargo lock contention from concurrent background cargo jobs in sibling
  worktrees caused intermittent long waits in test loops; targeted runs were
  still stable once contention was reduced.
- A Spark documentation review surfaced a real wording drift: Tab was listed as
  a global shortcut even though only Draw mode maps Tab to edge toggling.

## Decision Log

- Decision: keep this task focused on contract definition and integration seam,
  not full pen/select FSM extraction. Rationale: roadmap sequencing isolates
  this in `0.5.1`, with behaviour-heavy extractions in `0.5.2` and `0.5.3`.
  Date/Author: 2026-02-25 (assistant)

- Decision: define Tool trait in the model layer near `ToolMode`/`EdgeMode`
  (`src/model/tool.rs` or a sibling submodule) and adapt GPUI events at UI
  edge. Rationale: preserves GPUI independence and testability expectations.
  Date/Author: 2026-02-25 (assistant)

- Decision: treat `rstest-bdd` scenarios as behaviour contracts for tool intent
  routing and unhappy paths, while GPUI tests remain integration wiring checks.
  Rationale: follows repository testing guidance and GPUI testing split.
  Date/Author: 2026-02-25 (assistant)

- Decision: document Tab as context-split behaviour. Draw-mode Tab routes
  through the tool FSM (`ToggleEdgeMode`) and does not add document history;
  Manipulate-mode Tab routes to segment-kind toggling and is undoable document
  state. Rationale: aligns users guide wording with implemented keybinding
  contexts and avoids shortcut ambiguity.
  Date/Author: 2026-02-26 (assistant)

## Outcomes & Retrospective

Implemented outcomes:

- A stable `Tool` trait contract exists and is wired into the current tool flow.
- Tool input handling is represented as deterministic FSM transitions via
  `ToolModeFsm`.
- Tool actions now emit explicit command intents (`ToolCommand`) that are
  applied centrally rather than mutating tool state directly in handlers.
- Existing Draw/Manipulate behaviour remains intact under GPUI regression tests.
- Documentation and roadmap are synchronized with implementation.

Retrospective notes:

- The command-output boundary made it straightforward to test transitions in
  unit, BDD, and GPUI layers with consistent assertions.
- The highest operational risk was test-runner lock contention rather than
  architecture changes; serializing/monitoring cargo jobs was necessary for
  reliable verification.
- Final gates passed on 2026-02-26 with logs recorded in `/tmp`.

## Context and Orientation

Primary current code surfaces:

- Model tool definitions:
  - `src/model/tool.rs`
  - `src/model/engine_state.rs`
- Action and command boundary:
  - `src/model/action.rs`
  - `src/model/command/prepare.rs`
- Current tool handling in view/controller:
  - `src/ui/phase0_shell/input.rs`
  - `src/ui/phase0_shell/chrome.rs`
  - `src/ui/phase0_shell/view.rs`
  - `src/ui/phase0_shell/draw/mod.rs`
  - `src/ui/phase0_shell/manipulate/mod.rs`
- GPUI action bridge and contexts:
  - `src/ui/action_bridge/mod.rs`

Primary current test surfaces:

- Unit:
  - `src/model/tool_tests.rs`
  - `src/model/action_tests.rs`
- Behavioural (`rstest-bdd`):
  - `tests/tool_fsm_bdd.rs`
  - `tests/features/tool_fsm.feature`
- GPUI integration:
  - `tests/gpui_keybinding_integration.rs`
  - `tests/gpui_tool_rail.rs`
  - `tests/gpui_escape_returns_to_draw.rs`
  - `tests/gpui_draw_escape_commits_open_path.rs`

Documentation surfaces to update:

- `docs/gauss-architecture-design.md`
- `docs/users-guide.md`
- `docs/roadmap.md`
- `docs/execplans/0-5-1-tool-trait.md`

## Plan of Work

Stage A: define model-level tool FSM contract

- Add Tool trait and supporting data types in model code. The trait should
  represent:
  - current tool state,
  - input event handling,
  - output events that include command intents.
- Keep terminology explicit so future `PenTool` and `SelectTool` extractions can
  implement the same contract without trait redesign.
- Preserve existing `ToolMode` and `EdgeMode` semantics.

Validation gate:

- Unit tests prove trait behaviour for:
  - valid transition sequences (happy path),
  - ignored/rejected events in wrong states (unhappy path), and
  - command intent emission versus non-emitting preview transitions.
  Status: complete (see `src/model/tool_tests.rs`).

Stage B: integrate trait boundary into existing controller flow

- Replace direct tool-mode mutations in input and action handlers with calls
  through a tool boundary adapter.
- Ensure Escape/Tab/tool-activation routing still behaves identically from user
  perspective.
- Ensure command-producing transitions pass through existing command application
  seams rather than ad-hoc document writes.

Validation gate:

- Existing GPUI tests for tool switching and escape/tab interactions remain
  green.
- No new direct tool/document mutation paths are added in view handlers.
  Status: complete (`src/ui/phase0_shell/chrome.rs`, `input.rs`, `view.rs`,
  `tool_rail.rs`, and `draw/mod.rs`).

Stage C: behavioural and GPUI coverage expansion

- Add/extend `rstest-bdd` scenarios to cover tool-FSM routing semantics:
  - Given tool state + input event,
  - When action/intent is processed,
  - Then expected tool state and command-intent output occur.
- Include unhappy scenarios for invalid transitions and no-op events.
- Add targeted GPUI tests that prove the integration boundary routes through the
  new tool contract while preserving external behaviour.

Validation gate:

- Behaviour scenarios pass via `cargo test` using `rstest-bdd` macros.
- GPUI tests pass with deterministic assertions around mode and edge state.
  Status: complete (`tests/tool_fsm_bdd.rs`, `tests/features/tool_fsm.feature`,
  and GPUI integration tests).

Stage D: documentation and roadmap closure

- Update architecture doc section(s) to record the implemented Tool trait
  boundary and key design choices.
- Update users guide if any tool behaviour phrasing, shortcut semantics, or mode
  guarantees changed.
- Mark roadmap item `0.5.1` as done in `docs/roadmap.md`, including both child
  bullets, only after code/tests/docs are complete and gates pass.

Validation gate:

- Documentation references match implemented symbols and behaviour.
- Roadmap checkboxes reflect actual completion state.
  Status: complete.

## Concrete Steps

Run from repository root.

1. Discovery checkpoint (already performed during draft):

    `grepai search "tool finite state machine input events commands" --json --compact`

    `leta refs src/model/tool.rs:ToolMode`

2. Implement model trait contract and adapter seams in files listed above.

3. Add or extend tests in unit, BDD, and GPUI layers.

4. Update architecture and user docs; then mark roadmap item `0.5.1` done.

5. Run quality gates with persistent logs:

    `set -o pipefail`

    `project="$(basename "$PWD")"`

    `branch="$(git branch --show)"`

    `branch_safe="$(printf '%s' "$branch" | tr '/' '-')"`

    `make check-fmt | tee "/tmp/check-fmt-${project}-${branch_safe}.out"`

    `make lint | tee "/tmp/lint-${project}-${branch_safe}.out"`

    `make test | tee "/tmp/test-${project}-${branch_safe}.out"`

Expected gate outcomes:

- `make check-fmt` exits 0.
- `make lint` exits 0.
- `make test` exits 0, including updated tool boundary tests.

## Validation and Acceptance

Acceptance checklist for roadmap item `0.5.1`:

- [x] Tool contract is trait-based and FSM-oriented.
- [x] Tool processing is input-event-driven.
- [x] Tool outputs include command intents rather than direct state mutation.
- [x] Unit tests cover happy/unhappy/edge transition cases.
- [x] Behavioural tests (`rstest-bdd`) cover happy/unhappy command-intent
  routing.
- [x] GPUI tests confirm integration wiring and unchanged external behaviour.
- [x] Architecture design doc records the implemented boundary.
- [x] Users guide reflects any user-visible behaviour semantics.
- [x] Roadmap `0.5.1` and its child bullets are marked done.
- [x] `make check-fmt`, `make lint`, and `make test` all pass.

## Idempotence and Recovery

- Changes are source-controlled and may be applied incrementally.
- If a gate fails:
  - inspect corresponding `/tmp/*.out` log,
  - apply the smallest corrective change,
  - rerun the failed gate,
  - rerun full gate sequence before marking completion.
- If behaviour drift appears during integration:
  - pause and compare against existing GPUI regression tests,
  - restore old behaviour first,
  - then reintroduce trait boundary in smaller increments.
