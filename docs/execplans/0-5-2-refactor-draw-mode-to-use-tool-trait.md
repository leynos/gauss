# Refactor draw mode to PenTool FSM via Tool trait (0.5.2)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT (2026-02-27)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item `0.5.2` in `docs/roadmap.md` requires extracting the existing
Draw-mode behaviour into a dedicated `PenTool` finite-state machine (FSM)
implemented through the Tool trait boundary introduced in `0.5.1`.

The user-visible goal is stability: drawing, closing paths, edge toggling,
Escape behaviour, and undo semantics must remain unchanged while internal
boundaries become clearer and more testable.

Success is observable when:

- draw-mode logic is represented by a `PenTool` FSM that emits `ToolCommand`
  outputs rather than mutating shell state directly;
- `Phase0Shell` remains a UI adapter that maps GPUI events into tool inputs and
  applies emitted commands centrally;
- unit (`rstest`), behavioural (`rstest-bdd` v0.5.0), and GPUI tests cover
  happy paths, unhappy paths, and edge cases for Pen behaviour parity;
- design and user documentation reflect any behaviour or architecture updates;
- `make check-fmt`, `make lint`, and `make test` pass with durable log
  evidence; and
- roadmap entry `0.5.2` and children are marked done only after full green
  gates.

## Constraints

- Scope strictly to roadmap item `0.5.2` in `docs/roadmap.md`.
- Preserve existing Draw and Manipulate user-visible behaviour; this is a
  refactor milestone, not a feature-expansion milestone.
- Keep tool architecture aligned with `docs/gauss-architecture-design.md`
  (`Tool` as deterministic FSM, command emission boundary).
- Keep GPUI-specific concerns at the view/controller edge (`Phase0Shell`); keep
  deterministic tool decisions in model code.
- Use `grepai` and `leta` for discovery and symbol relationship mapping.
- Use Spark agent team coordination for planning and implementation updates,
  exchanging context through `context_pack`.
- Validate with `rstest` unit coverage, `rstest-bdd` v0.5.0 scenarios, and
  targeted GPUI integration tests for wiring/regression.
- Update `docs/gauss-architecture-design.md` with design decisions made during
  extraction.
- Update `docs/users-guide.md` only where behaviour or UI wording changes are
  user-visible.
- Mark roadmap `0.5.2` complete only after implementation and full gate
  success.
- Keep modules under 400 lines; split files where required.
- Do not suppress lints except as a last resort and with a tightly-scoped
  rationale.

## Tolerances (exception triggers)

- Scope: if implementation requires more than 20 files or 1,100 net lines,
  stop and re-evaluate decomposition.
- Interface: if `Tool` trait signatures must change in a way that impacts
  `0.5.3` or other roadmap milestones, stop and capture options in Decision Log
  before proceeding.
- Dependency: if a new runtime dependency is required, stop and escalate.
- Behaviour drift: if existing GPUI draw-mode regression tests fail for
  behaviour reasons (not compile errors), stop and perform parity analysis
  before continuing.
- Iteration: if gates fail across three repair cycles without reducing failure
  count, stop and escalate with evidence.
- Environment: if cargo lock contention or external process interference blocks
  reliable gate execution, stop and record blocker evidence before closure.

## Risks

- Risk: `Tool::transition` may be too narrow for Pen click context
  (active path, zoom, cursor world position, style, document snapshot).
  Severity: high Likelihood: medium Mitigation: introduce explicit Pen input
  context types and keep the trait contract coherent; document any signature
  decision in architecture docs.

- Risk: shape-ID allocation and active-path recovery can drift from existing
  semantics. Severity: high Likelihood: medium Mitigation: preserve command
  ordering and add targeted tests for stale `active_path` recovery and ID
  continuity.

- Risk: close-path behaviour (fill default and cubic handle synthesis) regresses
  during helper relocation. Severity: medium Likelihood: medium Mitigation:
  preserve helper logic semantics and extend GPUI close-path tests for both
  line and bezier modes.

- Risk: command-application failures may stop surfacing via
  `last_history_error`. Severity: medium Likelihood: low Mitigation: keep
  command application centralized in `apply_tool_commands` and retain
  unhappy-path assertions in view/tool tests.

- Risk: gate instability from concurrent cargo jobs in sibling worktrees causes
  ambiguous completion status. Severity: medium Likelihood: medium Mitigation:
  run gates with `set -o pipefail` and `tee`, monitor contention, and treat
  incomplete logs as unresolved.

## Progress

- [x] (2026-02-27 13:45Z) Confirmed branch context:
  `0-5-2-refactor-draw-mode-to-use-tool-trait`.
- [x] (2026-02-27 13:45Z) Loaded required skills and discovery tooling
  (`execplans`, `leta`, `grepai`).
- [x] (2026-02-27 13:45Z) Gathered roadmap, architecture, code, and test context
  via `grepai` and `leta`.
- [x] (2026-02-27 13:45Z) Ran Spark team analysis (Reimu/Marisa/Axel) and
  consolidated findings through context pack `pk_gvavom4z`.
- [x] (2026-02-27 13:45Z) Drafted this ExecPlan at
  `docs/execplans/0-5-2-refactor-draw-mode-to-use-tool-trait.md`.
- [x] (2026-02-27) Implemented `PenTool` extraction and adapter wiring.
- [x] (2026-02-27) Added/updated unit, BDD, and GPUI coverage for
  happy/unhappy/edge cases.
- [x] (2026-02-27) Updated architecture and user docs, and marked roadmap
  `0.5.2` done after gates passed.
- [x] (2026-02-27) Ran `make check-fmt`, `make lint`, and `make test` with
  durable logs and recorded evidence.

## Surprises & discoveries

- Existing `0.5.1` boundaries already centralize tool command application in
  `Phase0Shell`; `0.5.2` is primarily extraction and parity hardening.
- Draw-mode behaviour is concentrated in `src/ui/phase0_shell/draw/mod.rs`
  (`draw_click_world`, `start_new_open_shape`, close/append helpers), making a
  focused extraction feasible.
- The current mode FSM and draw click workflow have different input-shape needs;
  careful context modelling is required to avoid trait churn.
- Context-pack references were broadly accurate, with minor line-range drift in
  two files due to branch-local edits.
- `rstest-bdd` step text is strict; a singular/plural mismatch (`command` vs
  `commands`) caused scenario lookup failures until both forms were bound.

## Decision log

- Decision: treat this milestone as a behavioural parity refactor, not a
  feature enhancement pass. Rationale: roadmap `0.5.2` explicitly scopes to
  extraction
  - maintained functionality; later milestones handle broader behaviour.
  Date/Author: 2026-02-27 (assistant)

- Decision: keep `Phase0Shell` as the only command-application adapter and
  error-surfacing boundary during extraction. Rationale: reduces blast radius
  and preserves known integration semantics from `0.5.1`. Date/Author:
  2026-02-27 (assistant)

- Decision: require a dedicated Pen-focused behavioural test surface in addition
  to existing mode-FSM scenarios. Rationale: current `tool_fsm` coverage
  validates mode transitions, but not full click-path command sequencing.
  Date/Author: 2026-02-27 (assistant)

- Decision: keep close-path eligibility unchanged (minimum three anchors before
  closure) and document it explicitly in the user guide. Rationale: preserves
  behavioural parity while making the rule explicit for users and tests.
  Date/Author: 2026-02-27 (assistant)

## Outcomes & retrospective

Current outcome: implementation complete for roadmap item `0.5.2`.

What shipped:

- Extracted Pen draw-click FSM to model layer:
  `PenTool`, `PenToolClickInput`, and `PenToolActiveShape` in
  `src/model/tool.rs`.
- Moved draw geometry helpers from UI layer to model layer:
  `src/model/pen_geometry.rs`.
- Updated draw UI adapter (`src/ui/phase0_shell/draw/mod.rs`) to snapshot click
  context and route transitions via `Tool::transition(&PenTool, ...)`.
- Removed obsolete UI-only helper module:
  `src/ui/phase0_shell/draw/handles.rs`.
- Added/updated tests:
  - unit (`src/model/pen_tool_tests.rs`)
  - BDD (`tests/features/pen_tool.feature`, `tests/pen_tool_bdd.rs`)
  - GPUI parity/edge cases (`tests/gpui_close_path.rs`,
    `tests/gpui_draw_undo.rs`)
  - compatibility update for non-`Copy` `ToolInputEvent`
    (`tests/tool_fsm_bdd.rs`).

Gate evidence:

- `/tmp/check-fmt-gauss-0-5-2-refactor-draw-mode-to-use-tool-trait.out`
- `/tmp/lint-gauss-0-5-2-refactor-draw-mode-to-use-tool-trait.out`
- `/tmp/test-gauss-0-5-2-refactor-draw-mode-to-use-tool-trait.out`
- `/tmp/fmt-gauss-0-5-2-refactor-draw-mode-to-use-tool-trait.out`
- `/tmp/markdownlint-gauss-0-5-2-refactor-draw-mode-to-use-tool-trait.out`

## Context and orientation

Primary roadmap and architecture references:

- `docs/roadmap.md` (`0.5.2` scope and completion state)
- `docs/gauss-architecture-design.md` (§6 tool model, §20 immediate steps)
- `docs/using-gpui-and-gpui-component.md` (test split guidance)
- `docs/accesskit-based-accessibility-in-gpui.md` (UI interaction and
  accessibility context)
- `docs/rust-testing-with-rstest-fixtures.md`
- `docs/rstest-bdd-users-guide.md`
- `docs/rust-doctest-dry-guide.md`
- `docs/reliable-testing-in-rust-via-dependency-injection.md`

Primary code surfaces for extraction:

- `src/model/tool.rs` (Tool trait, mode FSM contracts)
- `src/ui/phase0_shell/draw/mod.rs` (current draw click/close/start flow)
- `src/model/pen_geometry.rs` (draw geometry helpers)
- `src/ui/phase0_shell/chrome.rs` (`handle_tool_input_event`,
  `apply_tool_commands`)
- `src/ui/phase0_shell/input.rs` and `src/ui/phase0_shell/view.rs`
  (event routing)

Primary test surfaces:

- Unit: `src/model/tool_tests.rs`, plus new Pen-focused unit test module(s)
- BDD: `tests/tool_fsm_bdd.rs`, `tests/features/tool_fsm.feature`, plus new
  Pen-focused BDD feature/steps
- GPUI: `tests/gpui_close_path.rs`, `tests/gpui_close_path_undo.rs`,
  `tests/gpui_draw_escape_commits_open_path.rs`,
  `tests/gpui_escape_returns_to_draw.rs`, `tests/gpui_draw_undo.rs`,
  `tests/gpui_draw_bezier_auto.rs`, `tests/gpui_keybinding_integration.rs`,
  `tests/gpui_tool_rail.rs`, `tests/gpui_mode_indicator.rs`

Documentation targets:

- `docs/gauss-architecture-design.md` (record PenTool extraction design)
- `docs/users-guide.md` (only if behaviour/wording changed)
- `docs/roadmap.md` (mark `0.5.2` and child bullets done at closure)
- `docs/execplans/0-5-2-refactor-draw-mode-to-use-tool-trait.md`
  (this living plan)

## Plan of work

Stage A: PenTool design and boundary definition (no behaviour change)

- Define `PenTool` FSM types in model code, including the minimum input context
  needed for click-path transitions.
- Keep deterministic output as ordered `ToolCommand` emission.
- Preserve existing mode FSM (`ToolModeFsm`) responsibilities for global
  mode-switch events, while introducing Pen-specific transitions for draw
  clicks.
- Capture the trait/input design decision in
  `docs/gauss-architecture-design.md`.

Validation gate:

- New/updated unit tests fail first for missing Pen transitions, then pass after
  implementation.
- No GPUI-layer edits yet beyond compile wiring shims.

Stage B: Extract draw click workflow to PenTool and wire adapter

- Move/encapsulate draw click decision logic from
  `src/ui/phase0_shell/draw/mod.rs` into PenTool transition handling.
- Keep GPUI coordinate conversion and event plumbing in `Phase0Shell`.
- Route Pen outputs through existing `apply_tool_commands` pathway.
- Preserve command ordering for:
  - start new shape,
  - append anchor,
  - close path,
  - stale active-path recovery.

Validation gate:

- Existing draw regression tests compile and maintain behavioural parity.
- Error propagation through `last_history_error` remains intact.

Stage C: Behavioural and GPUI regression expansion

- Add Pen-focused `rstest-bdd` scenarios for draw click workflow (happy,
  unhappy, and recovery paths).
- Extend GPUI tests for edge cases not fully locked today:
  - close attempts with fewer than 3 anchors,
  - close attempts outside snap radius,
  - stale active-path recovery behaviour,
  - no-regression on Escape/Tab and keybinding-driven mode activation.
- Keep scenario assertions observable (state and command outcomes), not internal
  implementation details.

Validation gate:

- New BDD and GPUI scenarios fail before extraction and pass after.
- Existing scenario suites remain green.

Stage D: Documentation sync, roadmap closure, and gates

- Update architecture design text with PenTool extraction decision and boundary
  notes.
- Update user's guide only if user-visible wording needs correction.
- Mark roadmap `0.5.2` and child bullets done after full gate success.
- Update this ExecPlan with final status, outcomes, and gate evidence.

Validation gate:

- Documentation describes true implemented behaviour.
- Roadmap state matches gate reality.

## Concrete steps

Working directory:

```plaintext
/data/leynos/Projects/gauss.worktrees/0-5-2-refactor-draw-mode-to-use-tool-trait
```

Discovery and symbol tracing (already completed for this plan draft):

```sh
grepai search --workspace Projects --project gauss \
  "draw mode tool trait pen tool fsm" --toon --compact --limit 10
leta grep "ToolModeFsm|ToolInputEvent|ToolCommand|ToolMode" \
  -k struct,enum,trait,function --head 200
```

Implementation-phase Spark coordination pattern:

```sh
# Use context pack id pk_gvavom4z to pin refs and collect agent findings.
# Agent tasks: scope/docs constraints, extraction seam risks, test matrix.
```

Required quality gates with durable logging:

```sh
set -o pipefail
make check-fmt | tee /tmp/check-fmt-$(get-project)-$(git branch --show).out
make lint | tee /tmp/lint-$(get-project)-$(git branch --show).out
make test | tee /tmp/test-$(get-project)-$(git branch --show).out
```

If lock contention occurs, record evidence and rerun once contention is cleared
before closure decisions.

## Validation and acceptance

Behavioural acceptance:

- Pen clicks still start, append, and close paths exactly as before.
- Escape still exits draw context as documented.
- Tab behaviour remains context-dependent (draw edge toggle vs manipulate
  segment toggle).
- Close-path and draw undo semantics remain unchanged.

Test acceptance:

- Unit tests (`rstest`) cover Pen transition happy/unhappy paths and
  deterministic command ordering.
- BDD tests (`rstest-bdd` v0.5.0) cover Pen workflow scenarios using
  Given/When/Then feature files and step definitions.
- GPUI tests verify wiring and end-to-end parity for key interactions.

Gate acceptance:

- `make check-fmt` succeeds.
- `make lint` succeeds.
- `make test` succeeds.
- Gate logs are present in `/tmp` with branch-safe filenames.

Documentation acceptance:

- Architecture design updated with extraction decisions.
- User's guide updated if and only if user-visible behaviour wording changed.
- Roadmap `0.5.2` and child bullets marked done after green gates.

## Idempotence and recovery

- All plan steps are intended to be re-runnable.
- If a refactor step partially lands, rerun targeted tests first, then proceed
  to full gates.
- If a gate fails due to environment contention, record process evidence, clear
  contention, and rerun the same command to regenerate authoritative logs.
- If behaviour parity is uncertain, do not mark roadmap done; keep ExecPlan
  status truthful and partial.

## Artifacts and notes

Current planning artifacts:

- Context pack id: `pk_gvavom4z`
- Spark team findings:
  - Reimu: scope boundaries and closure criteria
  - Marisa: extraction boundary and risk hotspots
  - Axel: validation matrix across unit/BDD/GPUI layers

Implementation closure artifacts to append later:

- Gate log paths under `/tmp`
- Key diffs and rationale snippets for architecture/doc updates
- Final acceptance checklist with timestamps

## Interfaces and dependencies

Planned interface additions/changes (tentative until implementation confirms):

- Introduce a model-layer `PenTool` implementing `Tool` or a closely aligned
  tool-transition contract.
- Introduce Pen input context structure(s) carrying draw-click decision inputs
  needed for deterministic transitions.
- Keep `ToolCommand` as the output channel for all state/document effects.
- Keep `Phase0Shell::apply_tool_commands` as the central command executor and
  error-surfacing path.

Dependency policy:

- No new runtime crates expected.
- Continue using existing test dependencies (`rstest`, `rstest-bdd`, GPUI test
  framework).

## Revision note

- 2026-02-27: Created initial ExecPlan draft for roadmap `0.5.2`.
  - Added scope, tolerances, risks, staged implementation plan, and validation
    criteria.
  - Incorporated discovery from `grepai`/`leta` and Spark team coordination via
    context pack `pk_gvavom4z`.
  - Set status to DRAFT pending implementation approval/execution.
- 2026-02-27: Completed roadmap `0.5.2` implementation and validation.
  - Extracted Pen draw-click transitions to `PenTool` (`Tool` implementation)
    with GPUI-independent geometry helpers in `src/model/pen_geometry.rs`.
  - Updated draw UI adapter to route click decisions through
    `ToolInputEvent::PenCanvasClicked`.
  - Added unit + BDD + GPUI coverage for happy, unhappy, and edge paths.
  - Updated architecture/user docs and marked roadmap `0.5.2` done.
  - Ran and passed `make check-fmt`, `make lint`, and `make test` with durable
    tee logs.
