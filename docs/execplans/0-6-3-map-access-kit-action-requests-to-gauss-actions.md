# Map AccessKit accessibility toolkit action requests to Gauss Actions (0.6.3)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE (2026-03-06)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item `0.6.3` in [docs/roadmap.md](../roadmap.md) requires Gauss to stop
treating the AccessKit (Accessibility Toolkit) action requests as metadata-only
semantics and instead route AccessKit action requests through the same editor
action pipeline used by keyboard shortcuts and UI controls.

After this milestone is implemented, assistive technology should be able to
invoke supported chrome and editor actions without special-case behaviour, and
keyboard-only behaviour should remain the parity baseline rather than a
separate implementation path.

Success is observable when:

- AccessKit action requests for supported nodes are translated into existing
  Gauss `Action` values or existing shell/window actions instead of bypassing
  the command path.
- The same state transitions happen whether a user triggers an operation from a
  GPUI (GPU-accelerated UI) keybinding, a GPUI control, or an accessibility
  action request.
- Unsupported or stale accessibility requests fail safely without mutating
  state or panicking.
- Unit tests (`rstest`), behaviour-driven development (BDD) tests
  (`rstest-bdd` v0.5.0), and GPUI tests cover happy paths, unhappy paths, and
  parity edge cases.
- `docs/gauss-architecture-design.md` records the final action-routing
  decision, `docs/users-guide.md` stops describing accessibility action
  invocation as unimplemented, and `docs/roadmap.md` marks `0.6.3` done only
  after implementation plus green gates.
- `make check-fmt`, `make lint`, and `make test` succeed with tee-logged
  evidence.

## Constraints

- Scope is strictly roadmap item `0.6.3`:
  - map AccessKit action requests to Gauss Actions;
  - ensure keyboard-only operation parity.
- Preserve the current `0.6.1` and `0.6.2` behaviour:
  deterministic tree projection, incremental updates, stable node IDs, and
  chrome semantics must remain intact.
- Reuse the existing action pipeline rather than inventing a second dispatch
  system:
  - model actions are defined in `src/model/action.rs`;
  - GPUI action bindings live in `src/ui/action_bridge/mod.rs`;
  - shell action bindings and handlers live in
    `src/ui/phase0_shell/action_dispatch.rs`.
- Keep stable node metadata centralized in
  `src/ui/phase0_shell/accessibility.rs`.
- Keep accessibility projection logic and accessibility request routing inside
  `src/ui/phase0_shell/a11y_service/` or a tightly adjacent seam; do not smear
  AccessKit-specific conditionals across unrelated model modules.
- Prefer mapping to existing actions and handlers. If a requested action has no
  matching Gauss action yet, document the gap and either:
  - route it through an existing shell/window action if that already provides
    the same behaviour, or
  - explicitly reject it as unsupported for this milestone.
- Test coverage must include:
  - unit tests with `rstest`,
  - behaviour tests with `rstest-bdd` v0.5.0,
  - GPUI integration tests using `#[gpui::test]`.
- Update:
  - `docs/gauss-architecture-design.md`,
  - `docs/users-guide.md`,
  - `docs/roadmap.md`.
- Do not start implementation until a maintainer approves the pull request.

## Tolerances (exception triggers)

- Scope tolerance: if correct implementation requires introducing new
  user-visible actions that are not already represented in
  `src/model/action.rs`, pause and decide whether they belong in `0.6.3` or a
  follow-up roadmap item.
- Architecture tolerance: if AccessKit request handling cannot be routed
  through existing shell/action seams without broad restructuring across more
  than two subsystems, stop and re-scope before continuing.
- Blast-radius tolerance: if delivery grows beyond 14 files or 650 net LOC,
  pause and decompose before proceeding.
- Platform tolerance: if this milestone requires wiring OS-specific AccessKit
  adapter callbacks that do not already exist in the codebase, stop and record
  the missing platform seam instead of improvising runtime adapter work.
- Test tolerance: if parity coverage requires replacing existing test harnesses
  rather than extending them, pause and split the harness refactor from the
  feature work.
- Gate tolerance: if `make check-fmt`, `make lint`, or `make test` fail for
  unrelated pre-existing reasons, capture tee-log evidence and keep completion
  partial instead of marking roadmap work done.

## Risks

- Risk: accessibility requests could bypass the existing action pipeline and
  drift from keyboard behaviour. Mitigation: define one translation layer from
  `(node id, AccessKit action)` to existing Gauss actions or existing shell
  window handlers, then reuse existing execution code paths.

- Risk: some AccessKit node/actions may not have a one-to-one match with the
  current `Action` enum. Mitigation: enumerate supported node/action pairs up
  front and treat unsupported requests as explicit no-ops or typed errors.

- Risk: chrome controls currently use shell/window action handlers rather than
  model `Action` values. Mitigation: document the distinction in the
  architecture update and keep the execution path unified at the shell layer
  even if the model-layer action enum remains unchanged for this milestone.

- Risk: stale node IDs or wrong action kinds could mutate the wrong target.
  Mitigation: add unhappy-path tests for stale shape IDs, unsupported actions,
  and mismatched node/action combinations before wiring the happy path.

- Risk: docs drift could leave `docs/users-guide.md` and `docs/roadmap.md`
  claiming accessibility action invocation is still deferred. Mitigation:
  update design doc, user's guide, and roadmap in the same change sequence
  before the final gate replay.

## Progress

- [x] (2026-03-06) Confirmed branch context:
  `0-6-3-map-access-kit-action-requests-to-gauss-actions`.
- [x] (2026-03-06) Loaded required skills: `execplans`, `grepai`, and `leta`.
- [x] (2026-03-06) Verified roadmap and architecture anchors for `0.6.3`.
- [x] (2026-03-06) Used `grepai` and `leta`-assisted discovery plus direct
  code inspection to map the current action bridge, shell action handlers, and
  accessibility service seams.
- [x] (2026-03-06) Used an agent team for planning activity and prepared
  context-pack anchor `pk_w27ptmrl` (`gauss-0-6-3-a11y-action-mapping`) for
  implementation coordination. The two parallel explorers were interrupted
  after timing out, so the final draft below uses locally verified evidence
  rather than unverified agent output.
- [x] (2026-03-06) Drafted this ExecPlan document.
- [x] (2026-03-06) User approved the ExecPlan and authorized implementation.
- [x] (2026-03-06) Confirmed the current tree only exposes AccessKit actions on
  chrome button nodes, so `0.6.3` will route those requests through the
  existing shell/window action path and fail closed for unsupported node/action
  pairs.
- [x] (2026-03-06) Implemented the initial request-routing seam plus unit, BDD,
  GPUI, and documentation updates for `0.6.3`; gate replay remains in progress
  before roadmap closure.
- [x] (2026-03-06) `make check-fmt` passed after applying the one rustfmt
  reflow required in `src/ui/phase0_shell/mod.rs`.
- [x] (2026-03-06) Completed focused verification for the new unit, BDD, and
  GPUI coverage before the full gate replay.
- [x] (2026-03-06) Updated architecture, user's guide, and roadmap documents to
  match shipped behaviour.
- [x] (2026-03-06) Ran required gates with tee logs:
  - `/tmp/check-fmt-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out`
  - `/tmp/lint-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out`
  - `/tmp/test-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out`
- [x] (2026-03-06) Marked roadmap item `0.6.3` done after implementation,
  documentation sync, and green gates.
- [x] (2026-03-14) Verified the follow-up routing review finding against the
  current branch, kept the local `UnknownNode` versus `UnsupportedAction`
  split, and reran the full gate stack with fresh tee logs:
  - `/tmp/check-fmt-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out`
  - `/tmp/lint-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out`
  - `/tmp/test-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out`
- [x] (2026-03-14) Removed duplicated BDD routing-error assertion scaffolding
  by extracting a shared helper in `tests/a11y_service_routing_bdd.rs`, then
  reran `make check-fmt`, `make lint`, and `make test` on the refactor.
- [x] (2026-03-14) Collapsed duplicate unit tests in
  `src/ui/phase0_shell/a11y_service/tests.rs` into one parameterized `rstest`,
  then reran `make check-fmt`, `make lint`, and `make test`.
- [x] (2026-03-14) Replaced the duplicated unknown-node literal in
  `tests/a11y_service_routing_bdd.rs` with a shared constant, then reran
  `make check-fmt`, `make lint`, and `make test` after the crash-restarted gate
  stack completed successfully.

## Surprises & Discoveries

- `docs/gauss-architecture-design.md` §11.1 already names the missing contract
  precisely: `A11yService` must map AccessKit action requests back into Gauss
  Actions/Commands, and `0.6.3` is the deferred milestone that closes that gap.
- The current accessibility tree already exposes clickable chrome button nodes
  via `supports_action(Action::Click)` in unit, BDD, and GPUI tests, but no
  request-handling seam exists yet in `src/ui/phase0_shell/a11y_service/mod.rs`.
- The GPUI path is already well-factored:
  - model actions are declared in `src/model/action.rs`;
  - default keybindings live in `src/model/keybinding/mod.rs`;
  - GPUI bridge structs and registrations live in `src/ui/action_bridge/mod.rs`;
  - `Phase0Shell::bind_model_actions()`,
    `Phase0Shell::bind_window_actions()`, and
    `Phase0Shell::execute_model_action()` in
    `src/ui/phase0_shell/action_dispatch.rs` perform the actual dispatch.
- Window chrome actions (`MinimizeWindow`, `ToggleMaximize`,
  `ToggleFullscreen`, `CloseWindow`, `ShowWindowMenu`) are separate GPUI action
  handlers, not model `Action` enum variants. The implementation needs one
  honest decision about whether `0.6.3` keeps chrome routing at the shell layer
  or adds model-level actions.
- Existing test coverage is already split the right way for this milestone:
  unit tests for `A11yService`, BDD tests around accessibility behaviour, and
  GPUI tests for keybinding/action parity. `0.6.3` can extend those suites
  instead of inventing new harnesses.
- The shared workspace build directory can be locked by unrelated or abandoned
  `cargo test` processes in other worktrees. For this implementation, the first
  focused verification run blocked on that lock until a stale test process from
  an earlier local attempt was cleared.
- Repository lint policy also enforces a 400-line module cap via Whitaker.
  Completing `0.6.3` required moving action-routing and shell-dispatch logic
  into dedicated submodules rather than leaving the new code in already-large
  `a11y_service/mod.rs` and `action_dispatch.rs`.

## Decision Log

- 2026-03-06: Planning will treat keyboard parity as the reference execution
  path, not merely an additional validation surface. Rationale: the roadmap and
  architecture both say accessibility actions must trigger the same pipeline as
  UI and keyboard use.

- 2026-03-06: The first implementation target should be a translation seam from
  AccessKit requests to existing shell/model actions, not direct mutation from
  `A11yService`. Rationale: preserves the "Everything is an Action" invariant
  and minimizes behavioural drift.

- 2026-03-06: Chrome-node routing and document/tool routing may legitimately
  use different action types as long as they converge on the same shell command
  path. Rationale: today the codebase already separates model `Action` values
  from window-control GPUI actions.

- 2026-03-06: No implementation work begins until the user approves this
  ExecPlan. Rationale: follow the required approval gate for plan-first work.

- 2026-03-06: `0.6.3` stays open in `docs/roadmap.md` until `make lint` and
  `make test` pass, even though the code and docs changes are already in place.
  Rationale: roadmap closure follows the repository gate policy rather than
  implementation-complete intuition.

- 2026-03-06: Keep BDD coverage focused on the routed-request contract exposed
  by `A11yService`, and use GPUI tests to prove the shell execution path and
  quit-state side effects. Rationale: the existing BDD harness is snapshot-
  centric, while GPUI tests already provide the end-to-end shell surface needed
  for parity assertions.

- 2026-03-06: Extracted `src/ui/phase0_shell/a11y_service/action_routing.rs`
  and `src/ui/phase0_shell/action_dispatch.rs` to satisfy repository size gates
  without weakening the shared action-path design. Rationale: the
  implementation needs to pass Whitaker's module-length policy, not just
  compile and test.

## Outcomes & Retrospective

Current outcome: complete.

Delivered behaviour:

- supported AccessKit click requests for Phase 0 chrome controls route through
  the same `Phase0Shell` execution helpers used by GPUI actions and keyboard
  shortcuts;
- unsupported tree IDs or unsupported node/action pairs fail closed with typed
  routing errors and do not mutate shell state;
- stale or missing accessibility node IDs now fail with
  `A11yActionRequestError::UnknownNode`, keeping them distinct from unsupported
  action kinds on otherwise valid nodes;
- unit tests, `rstest-bdd` scenarios, and GPUI tests cover supported routing,
  unsupported requests, and shell-side quit behaviour parity;
- `docs/gauss-architecture-design.md`, `docs/users-guide.md`, and
  `docs/roadmap.md` all reflect the shipped behaviour.

Gate evidence:

- `make check-fmt` passed.
- `make lint` passed, including Whitaker's module-size and style checks.
- `make test` passed with the full workspace target set.

Status: COMPLETE

This milestone left the repository with:

- a documented and tested route from supported AccessKit action requests into
  existing Gauss action handlers;
- explicit unsupported-action handling for out-of-scope requests;
- parity coverage showing keyboard and accessibility requests exercise the same
  behaviour;
- synchronized roadmap, architecture, and user's guide status.

## Context and orientation

Primary requirement anchors:

- `docs/roadmap.md` lines 172-182 define `0.6.3`.
- `docs/gauss-architecture-design.md` lines 1266-1300 define the missing
  `A11yService` action-routing contract.
- `docs/gauss-architecture-design.md` lines 1583-1601 mark `0.6.3` as the
  remaining accessibility milestone in the architecture-foundation phase.
- `docs/users-guide.md` accessibility status now states that
  assistive-technology click requests for the shipped chrome controls are
  mapped through the same shell/window action handlers used by keyboard
  shortcuts and UI controls.

Current code surfaces:

- `src/ui/phase0_shell/a11y_service/mod.rs`
  - current tree projection and incremental update bookkeeping.
- `src/ui/phase0_shell/a11y_service/action_routing.rs`
  - current request-to-action translation entrypoint and typed routing errors.
- `src/ui/phase0_shell/accessibility.rs`
  - stable node IDs and chrome semantics;
  - authoritative source for which node IDs represent actionable chrome
    controls.
- `src/model/action.rs`
  - current document/editor action inventory to reuse for shape, tool,
    selection, and history requests.
- `src/model/keybinding/mod.rs`
  - current keyboard bindings, which define the parity baseline.
- `src/ui/action_bridge/mod.rs`
  - GPUI action bridge types and keybinding registration.
- `src/ui/phase0_shell/action_dispatch.rs`
  - `bind_window_actions()`, `bind_model_actions()`, and
    `execute_model_action()` are the current shell action-dispatch points.

Current validation surfaces:

- Unit:
  - `src/ui/phase0_shell/a11y_service/tests.rs`
    - current `rstest` coverage for tree projection, incremental updates, and
      routed request handling.
- Behaviour:
  - `tests/a11y_service_bdd.rs`
  - `tests/a11y_service_routing_bdd.rs`
- GPUI:
  - `tests/gpui_a11y_service.rs`
  - `tests/gpui_keybinding_integration.rs`

Documentation surfaces to update during implementation:

- `docs/gauss-architecture-design.md`
- `docs/users-guide.md`
- `docs/roadmap.md`

## Agent-team execution plan

Use a small team during implementation, coordinated around context pack
`pk_w27ptmrl`.

1. Explorer A: verify the exact supported AccessKit action/node matrix by
   reading `src/ui/phase0_shell/accessibility.rs`,
   `src/ui/phase0_shell/a11y_service/mod.rs`, and related tests.
2. Explorer B: verify the action execution path from model action definitions
   through `src/ui/action_bridge/mod.rs` into
   `src/ui/phase0_shell/action_dispatch.rs`, and identify which behaviours
   still use shell-only window actions.
3. Worker: implement the request-routing seam plus test/doc updates, then run
   the required gates.

If agent findings disagree with local evidence, local verified code wins and
the context pack should be updated before continuing.

## Supported-behaviour target for 0.6.3

The implementation should produce an explicit support matrix rather than
implicitly routing every AccessKit request.

Planned supported requests:

1. Chrome button `Click` requests for:
   - window menu,
   - minimize,
   - maximize/restore,
   - fullscreen,
   - close.
2. Shape-list or canvas-adjacent requests that clearly map to existing editor
   actions already represented in `Action`, if those nodes currently expose the
   corresponding AccessKit actions.
3. Focus-related requests only if the current shell already has a deterministic
   focus target transition for the same behaviour.

Planned unsupported requests for this milestone unless code inspection proves
they are already modelled:

1. Text navigation or rich-text actions.
2. Value-setting actions for controls that do not yet exist in Phase 0.
3. Arbitrary shape-edit commands that have no current `Action` equivalent.

## Plan of work

### Stage A: Define the request-routing seam

Add a small translation layer near `A11yService` that accepts:

- the target `NodeId`,
- the AccessKit action request kind,
- current shell/application state required to validate the request.

This layer should return one of:

- an existing model `Action`,
- an existing shell/window control action,
- a typed unsupported/invalid request result.

Acceptance for Stage A:

- supported node/action pairs are enumerated in code and tests;
- stale or unsupported requests are explicit and side-effect-free;
- the translation layer does not mutate state directly.

### Stage B: Reuse the existing execution path

Wire the routing seam into the existing shell action handlers so that
successful accessibility requests execute the same code paths used by keyboard
and GPUI controls.

Expected code touchpoints:

- `src/ui/phase0_shell/a11y_service/mod.rs`
- `src/ui/phase0_shell/action_dispatch.rs`
- possibly `src/ui/phase0_shell/mod.rs` or `test_helpers.rs`
- only extend `src/model/action.rs` if a genuinely missing action is required
  and approved.

Acceptance for Stage B:

- no direct document mutation occurs from the accessibility handler;
- keyboard and accessibility requests converge on the same handler logic;
- no-op or unsupported requests leave state unchanged.

### Stage C: Extend unit coverage first

Before the final wiring is considered complete, extend
`src/ui/phase0_shell/a11y_service/tests.rs` with `rstest` coverage for:

1. supported chrome-node click routing;
2. stale node IDs rejected without mutation;
3. unsupported actions on supported nodes rejected without mutation;
4. supported actions on wrong node kinds rejected without mutation;
5. any shape/editor action routing that is in scope for the milestone.

Acceptance for Stage C:

- tests prove both happy and unhappy path request translation;
- tests assert no state-change side effects on rejected requests.

### Stage D: Extend behaviour-driven coverage

Add behaviour-driven development (BDD) scenarios to:

- `tests/features/a11y_service.feature`
- `tests/a11y_service_bdd.rs`

Cover observable behaviour such as:

1. clicking a chrome accessibility node requests the matching editor/window
   behaviour;
2. unsupported accessibility requests are ignored with explicit failure
   reporting;
3. keyboard-triggered and accessibility-triggered paths lead to the same
   outcome for at least one shared action.

Acceptance for Stage D:

- the feature file reads as user-observable accessibility behaviour rather than
  internal implementation detail;
- step definitions remain small and focused.

### Stage E: Extend GPUI parity coverage

Add or extend GPUI tests in:

- `tests/gpui_a11y_service.rs`
- `tests/gpui_keybinding_integration.rs`

The GPUI layer should prove that:

1. a supported accessibility action changes state in the same way as the
   existing GPUI-dispatched action;
2. unsupported or stale requests do not crash and do not mutate state;
3. keyboard-only parity remains true for any actions covered by the new
   accessibility route.

Acceptance for Stage E:

- at least one parity assertion compares equivalent before/after states across
  keyboard and accessibility triggers;
- test helpers remain reusable rather than embedding large bespoke fixtures.

### Stage F: Update docs and roadmap

Update:

1. `docs/gauss-architecture-design.md`
   - record the final request-routing decision and any intentional
     unsupported-action boundaries.
2. `docs/users-guide.md`
   - replace the current "not yet mapped" note with shipped behaviour and any
     limitations the user should know.
3. `docs/roadmap.md`
   - mark `0.6.3` done only after implementation and all required gates pass.

Acceptance for Stage F:

- docs describe shipped behaviour, not planned behaviour;
- roadmap checklist and surrounding narrative stay consistent.

### Stage G: Run the required gates

Run the required gates with tee logs and `pipefail`:

```bash
set -o pipefail && make check-fmt | tee /tmp/check-fmt-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out
```

```bash
set -o pipefail && make lint | tee /tmp/lint-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out
```

```bash
set -o pipefail && make test | tee /tmp/test-gauss-0-6-3-map-access-kit-action-requests-to-gauss-actions.out
```

Acceptance for Stage G:

- all three commands exit successfully;
- the roadmap checkbox is updated only after these passes are confirmed.

## Approval gate

Implementation must not begin until the user explicitly approves this document
or requests revisions to it.
