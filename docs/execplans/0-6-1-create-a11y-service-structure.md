# Create A11yService skeleton with incremental updates (0.6.1)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: DRAFT (2026-03-01)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item `0.6.1` in `docs/roadmap.md` requires creating an `A11yService`
structure that:

- builds an AccessKit tree from UI and document state;
- pushes incremental updates as state changes.

The user-visible outcome is the first real accessibility subsystem backbone for
Gauss. After implementation, accessibility tree updates should be deterministic
and stable, and the architecture should be ready for follow-on items `0.6.2`
(stable ID wiring in chrome) and `0.6.3` (action request mapping).

Success is observable when:

- `A11yService` exists with a clear boundary between tree computation and
  platform adapter integration;
- tree snapshots are built from current `Phase0Shell` + document state;
- incremental updates are emitted only for relevant state changes;
- unit tests (`rstest`), behaviour tests (`rstest-bdd` v0.5.0), and GPUI tests
  cover happy, unhappy, and edge paths;
- `docs/gauss-architecture-design.md` records decisions made during delivery;
- `docs/users-guide.md` documents user-visible accessibility behaviour/status;
- `make check-fmt`, `make lint`, and `make test` pass with tee-logged evidence;
- roadmap entry `0.6.1` (and its child bullets) is marked done only after full
  implementation and green gates.

## Constraints

- Scope strictly to roadmap item `0.6.1` in `docs/roadmap.md`.
- Keep this plan self-contained and implementation-ready for a future execution
  pass.
- Preserve existing user-visible editing behaviour; this is an architecture
  foundation milestone.
- Align with architecture §11.1 in `docs/gauss-architecture-design.md`.
- Reuse existing stable ID foundations:
  - `src/ui/phase0_shell/accessibility.rs` constants for chrome nodes.
  - `src/model/path.rs` `ShapeId <-> AccessKit node id` mapping.
- Use `grepai` and `leta` for codebase understanding.
- Use agent-team synthesis through `context_pack` (`pk_htrwgi43`) for planning
  and implementation coordination.
- Keep `A11yService` platform-agnostic in core logic; isolate OS integration
  concerns behind adapter seams.
- Validate via:
  - unit tests with `rstest`,
  - behaviour tests with `rstest-bdd` v0.5.0,
  - targeted GPUI tests using `TestAppContext` and `VisualTestContext`.
- Update `docs/gauss-architecture-design.md` with design decisions made during
  implementation.
- Update `docs/users-guide.md` for any user-visible behaviour/status changes.
- Keep files under 400 lines by splitting scenario bindings and step
  definitions where needed.
- Do not mark roadmap `0.6.1` done until implementation plus full gates are
  complete.

## Tolerances (exception triggers)

- Scope: if implementation needs more than 24 files or 1,300 net LOC, stop and
  re-evaluate decomposition.
- Interface: if this milestone requires changing public behaviour of existing
  edit commands/tools, stop and escalate before proceeding.
- Dependency: if a new runtime dependency beyond planned AccessKit crates is
  required, stop and escalate.
- Architecture: if platform-specific adapter code must leak into model-layer
  logic, stop and restructure before continuing.
- Iteration: if test/lint gates fail across 3 repair cycles without reducing
  failures, stop and escalate with evidence.
- Environment: if gate commands fail due to contention/timeouts, record tee
  logs and keep status partial rather than inferring success.

## Risks

- Risk: ID-domain collisions between chrome constants and shape-derived IDs.
  Severity: high Likelihood: medium Mitigation: reserve explicit ID ranges and
  add collision tests.

- Risk: update chattiness in immediate-mode rendering causes noisy or expensive
  tree updates. Severity: medium Likelihood: medium Mitigation: dirty-bit
  tracking plus one coalesced update per frame/state cycle.

- Risk: adapter lifecycle timing differs by platform and may not align with
  current GPUI window readiness/focus events. Severity: high Likelihood: medium
  Mitigation: define explicit lifecycle hooks in the service contract and keep
  platform attach logic isolated.

- Risk: action parity gaps between model actions and direct window-control
  handlers delay `0.6.3` integration. Severity: medium Likelihood: medium
  Mitigation: capture explicit unsupported-action behaviour and keep mapping
  table extensible.

- Risk: roadmap closure drift (marking done before all gates/docs are complete).
  Severity: medium Likelihood: medium Mitigation: enforce closure checklist and
  tee-log evidence in this plan.

## Progress

- [x] (2026-03-01) Confirmed branch context:
  `0-6-1-create-a11y-service-structure`.
- [x] (2026-03-01) Loaded required skills and guidance:
  `leta`, `grepai`, `execplans`.
- [x] (2026-03-01) Gathered roadmap + architecture anchors for `0.6.1`.
- [x] (2026-03-01) Mapped existing accessibility seams with `grepai` and
  `leta` (`Phase0Shell`, stable ID mapping, existing constants).
- [x] (2026-03-01) Ran agent-team analysis and shared references through
  context pack `pk_htrwgi43`:
  - Reimu Hakurei (service seams and milestones),
  - Marisa Kirisame (test strategy matrix),
  - Axel Stone (docs + roadmap closure policy).
- [x] (2026-03-01) Drafted this ExecPlan document.
- [ ] Implement `A11yService` skeleton code and wiring.
- [ ] Add/extend unit, BDD, and GPUI tests for 0.6.1 behaviour.
- [ ] Update architecture and user documentation.
- [ ] Run full gates and capture tee logs.
- [ ] Mark roadmap `0.6.1` (and child bullets) done only after full closure.

## Surprises & discoveries

- This worktree already contains foundational accessibility constants in
  `src/ui/phase0_shell/accessibility.rs`, but they are intentionally unused and
  marked as prepared for future integration.
- Stable node identity foundations already exist for document shapes via
  `ShapeId::to_accesskit_node_id` and `ShapeId::from_accesskit_node_id`.
- `leta` has a broken-pipe failure mode when piped output is truncated; retry
  or avoid piped listing commands for stable discovery.
- Current accessibility references are concentrated in design/docs and ID
  helpers; there is no existing runtime AccessKit adapter wiring yet.

## Decision log

- Decision: this deliverable is plan-only for now; implementation is deferred
  to a subsequent execution pass. Rationale: user request is to formulate an
  execplan artifact for roadmap `0.6.1`. Date/Author: 2026-03-01 (assistant)

- Decision: keep roadmap `0.6.1` unchecked during plan-only delivery.
  Rationale: roadmap completion requires implemented behaviour, updated docs,
  and full green gates. Date/Author: 2026-03-01 (assistant)

- Decision: propose a dedicated `src/ui/a11y/` module boundary for service,
  tree building, diffing, and mapping concerns. Rationale: maintains separation
  of concerns and avoids crowding existing `phase0_shell` modules. Date/Author:
  2026-03-01 (assistant)

- Decision: baseline-first incremental policy (`full tree first`, then diffs).
  Rationale: deterministic bootstrapping and simpler correctness proofs for
  tests. Date/Author: 2026-03-01 (assistant)

- Decision: preserve adapter-agnostic service core and isolate platform adapter
  lifecycle hooks. Rationale: architecture boundary consistency and reduced
  cross-platform risk. Date/Author: 2026-03-01 (assistant)

## Outcomes & retrospective

Current outcome: planning complete, implementation pending.

What this plan now provides:

- clear architecture-aligned milestones for implementing `A11yService` 0.6.1;
- explicit unit/BDD/GPUI validation expectations including unhappy paths;
- documentation update requirements for architecture and user guide;
- deterministic gate and roadmap closure criteria.

Lessons for follow-on execution:

- start with ID policy and service contract before touching adapters;
- keep diff logic testable without platform dependencies;
- enforce tee-log evidence and honest partial status when gates are blocked.

## Context and orientation

Primary requirement references:

- `docs/roadmap.md` (`0.6.1` scope and child bullets)
- `docs/gauss-architecture-design.md` (§11.1, §20)
- `docs/using-gpui-and-gpui-component.md` (GPUI testing guidance)
- `docs/accesskit-based-accessibility-in-gpui.md` (AccessKit model and update
  considerations)
- `docs/rust-testing-with-rstest-fixtures.md`
- `docs/rstest-bdd-users-guide.md`
- `docs/rust-doctest-dry-guide.md`
- `docs/reliable-testing-in-rust-via-dependency-injection.md`

Key current code surfaces:

- `src/ui/phase0_shell/mod.rs` (`Phase0Shell` state/lifecycle seams)
- `src/ui/phase0_shell/view.rs` (render/update seam)
- `src/ui/phase0_shell/chrome.rs` and `window_controls.rs` (chrome controls)
- `src/ui/phase0_shell/accessibility.rs` (predefined node IDs and labels)
- `src/model/path.rs` (`ShapeId` AccessKit node id mapping)
- `src/model/action.rs` (action routing target surface)

Proposed new module surfaces:

- `src/ui/a11y/mod.rs`
- `src/ui/a11y/service.rs`
- `src/ui/a11y/tree_builder.rs`
- `src/ui/a11y/diff.rs`
- `src/ui/a11y/action_map.rs`
- `src/ui/a11y/node_ids.rs`

## Plan of work

Stage A: Define service contracts and ID policy

1. Introduce `A11yService` structure and lifecycle hooks.
2. Define snapshot/diff models and typed error surfaces.
3. Define ID-range policy to prevent chrome/document collisions.
4. Wire existing stable ID sources (`accessibility.rs` constants and
   `ShapeId` mapping).

Acceptance for Stage A:

- service APIs compile and are documented;
- ID policy is explicit and unit-testable;
- no behaviour regressions in unrelated UI features.

Stage B: Build full-tree baseline from shell + document state

1. Build root/chrome/canvas/shape nodes deterministically.
2. Ensure shape nodes use stable IDs derived from `ShapeId`.
3. Capture focus/selection representation in the tree state model.

Acceptance for Stage B:

- deterministic full tree output from identical input snapshots;
- first publish path exists for later incremental updates.

Stage C: Add incremental update pipeline

1. Implement diff computation from old/new snapshots.
2. Add dirty trigger mapping from command, selection, and tool-state changes.
3. Emit no-op updates when there is no relevant state change.

Acceptance for Stage C:

- insert/update/remove and no-op paths are covered by unit tests;
- unchanged frames do not rebuild/push full trees.

Stage D: Define action request mapping seam (0.6.1-level skeleton)

1. Add mapping table from accessibility action requests to existing Gauss
   actions/handlers.
2. Return explicit unsupported-action outcomes where mapping is deferred to
   `0.6.3`.
3. Keep routing through existing action/command pathways.

Acceptance for Stage D:

- mapping seam exists and is test-covered for supported/unsupported cases;
- no direct model mutation bypass is introduced.

Stage E: Validation matrix (unit + BDD + GPUI)

1. Unit tests (`rstest`) for tree build, diff, and error paths.
2. Behaviour tests (`rstest-bdd` v0.5.0) with split `main.rs`/`steps.rs`:
   - baseline snapshot creation,
   - incremental insert/update/remove,
   - stable IDs under reorder,
   - no-op update on unchanged state,
   - stale ID recovery,
   - collision or invalid-sequence error paths.
3. GPUI tests for service wiring and update triggering from user interactions.

Acceptance for Stage E:

- test coverage includes happy, unhappy, and edge cases;
- test layout respects module size constraints.

Stage F: Documentation, gates, and roadmap closure

1. Update `docs/gauss-architecture-design.md`:
   - §11.1 explicit service contract,
   - adapter boundary clarifications,
   - incremental update testing expectations.
2. Update `docs/users-guide.md` for user-visible accessibility status/notes.
3. Run gates with tee logs and record evidence.
4. Mark roadmap `0.6.1` and child bullets done only after all closure criteria
   are met.

Acceptance for Stage F:

- docs match implemented behaviour and architecture decisions;
- all required gates are green with durable logs;
- roadmap status reflects actual completion state.

## Validation and command runbook

Run commands with durable tee logs and `pipefail`.

```sh
set -o pipefail

# Targeted test loops during implementation
cargo test --lib a11y_service \
  | tee "/tmp/test-unit-$(get-project)-$(git branch --show).out"
cargo test --test a11y_service_bdd \
  | tee "/tmp/test-bdd-$(get-project)-$(git branch --show).out"
cargo test --test gpui_a11y_service \
  | tee "/tmp/test-gpui-$(get-project)-$(git branch --show).out"

# Required closure gates
make check-fmt \
  | tee "/tmp/check-fmt-$(get-project)-$(git branch --show).out"
make lint \
  | tee "/tmp/lint-$(get-project)-$(git branch --show).out"
make test \
  | tee "/tmp/test-$(get-project)-$(git branch --show).out"
```

If documentation changes are part of closure, also run:

```sh
set -o pipefail
make fmt \
  | tee "/tmp/fmt-$(get-project)-$(git branch --show).out"
make markdownlint \
  | tee "/tmp/markdownlint-$(get-project)-$(git branch --show).out"
make nixie \
  | tee "/tmp/nixie-$(get-project)-$(git branch --show).out"
```

## Closure checklist for roadmap 0.6.1

Only mark roadmap entry `0.6.1` done when all conditions are true:

1. `A11yService` structure exists and builds tree from UI + document state.
2. Incremental update path exists and is validated.
3. Unit, BDD, and GPUI tests covering happy/unhappy/edge cases are green.
4. `docs/gauss-architecture-design.md` captures final design decisions.
5. `docs/users-guide.md` reflects user-visible accessibility behaviour/status.
6. `make check-fmt`, `make lint`, and `make test` all succeed with tee logs.
7. Child bullets under `0.6.1` are checked before checking the parent item.
