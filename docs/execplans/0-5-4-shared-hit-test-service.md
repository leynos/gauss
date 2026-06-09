# Implement shared hit-test service for deterministic selection and hover (0.5.4)

This Execution Plan (ExecPlan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE (2026-03-05 12:07Z)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item `0.5.4` in `docs/roadmap.md` requires a shared deterministic
hit-test service to support both selection and hover behaviour, with an
architecture that can evolve into spatial indexing (`R-tree`/`BVH`) later.

Success criteria for this milestone were:

- Shared model-layer hit-testing service with deterministic ordering.
- Selection and hover routed through one service in manipulate mode.
- Index abstraction boundary for future acceleration structures.
- Coverage at unit (`rstest`), behavioural (`rstest-bdd`), and GPUI layers.
- Architecture and user docs updated to reflect shipped behaviour.
- Roadmap item `0.5.4` marked done.
- Required gates passing with tee-logged evidence.

All criteria are now satisfied.

## Constraints

- Scope strictly to roadmap item `0.5.4` in `docs/roadmap.md`.
- Keep deterministic hit priority consistent with prior semantics:
  handle -> anchor -> segment -> shape -> none.
- Keep hit testing GPUI-independent in model-layer code.
- Keep `Phase0Shell` as adapter logic; do not move model transitions into UI.
- Provide an explicit index abstraction for future `R-tree`/`BVH` work.
- Use `grepai` and `leta` for discovery and symbol-level navigation.
- Coordinate planning and implementation using an agent team and context pack
  `pk_s233wctv`.
- Validate with unit (`rstest`), behavioural (`rstest-bdd`), and GPUI tests.
- Record design decisions in `docs/gauss-architecture-design.md`.
- Update `docs/users-guide.md` for user-visible behaviour notes.
- Mark roadmap `0.5.4` done only after docs + tests + gates converge.

## Tolerances (exception triggers)

- Scope: if implementation exceeds 18 touched files or 900 net lines changed,
  stop and split work into narrower milestones before continuing.
- Interface churn: if shared hit-test extraction requires redesigning `Tool`
  trait or `ToolCommand` contracts, stop and document options.
- Behaviour drift: if existing manipulate GPUI tests regress behaviourally,
  stop and perform parity analysis before continuing.
- Iteration: if gate failures persist across three repair cycles without lower
  failure count, stop and escalate with evidence.

No tolerance trigger required escalation for this delivery.

## Risks

- Risk: introducing mismatch between selection hit behaviour and prior shipped
  behaviour. Severity: high. Likelihood: medium. Mitigation: preserve hit order
  contract and add regression tests for priority and topmost selection.
- Risk: hover state updates may create redraw churn or stale state retention.
  Severity: medium. Likelihood: medium. Mitigation: update hover via shared
  service and clear when leaving manipulate mode.
- Risk: accidental coupling between service internals and UI details blocks
  future index optimizations. Severity: medium. Likelihood: medium. Mitigation:
  expose index/service API at model boundary and keep adapter usage narrow.

## Progress

- [x] (2026-03-05 11:31Z) Confirmed branch context:
  `0-5-4-shared-hit-test-service`.
- [x] (2026-03-05 11:34Z) Loaded required skills and discovery tooling:
  `execplans`, `grepai`, `leta`, `rust-router`.
- [x] (2026-03-05 11:35Z) Created shared context pack `pk_s233wctv` and used
  an agent team for parallel planning/exploration.
- [x] (2026-03-05 11:39Z) Authored initial ExecPlan for `0.5.4`.
- [x] (2026-03-05 11:59Z) Implemented shared model hit-test service and adapter
  wiring for manipulate selection + hover.
- [x] (2026-03-05 12:03Z) Added unit, behavioural, and GPUI tests for happy,
  unhappy, and edge cases.
- [x] (2026-03-05 12:05Z) Updated architecture and user docs; marked roadmap
  `0.5.4` done.
- [x] (2026-03-05 12:07Z) Replayed docs + code gates with tee logs and prepared
  commit.

## Surprises & Discoveries

- Clippy enforced `self-named-module-files` for `hit_test.rs`, so the shared
  service moved into `src/model/hit_test/mod.rs`.
- Project lint policy enforces a 400-line per-file limit, so geometry helpers
  were extracted to `src/model/hit_test/geometry.rs`.
- Float-coordinate assertions in GPUI tests were brittle until world positions
  were anchored to canvas coordinates used by existing fixture conventions.
- Existing manipulate adapter boundaries were stable enough that shared
  hit-testing could be introduced without `Tool` trait changes.

## Decision Log

- Decision: implement a model-layer `HitTestIndex` with a linear-scan backend
  first, while exposing backend and index seams for later `R-tree`/`BVH`
  replacement. Rationale: satisfies architecture §6.2 now without over-scoping.
  Date/Author: 2026-03-05 (assistant)
- Decision: route both selection hit resolution and hover updates through the
  same index in manipulate mode. Rationale: one deterministic source of truth
  avoids divergent hit semantics between interaction paths. Date/Author:
  2026-03-05 (assistant)

## Outcomes & Retrospective

Shipped implementation:

- Added shared hit-test module:
  - `src/model/hit_test/mod.rs`
  - `src/model/hit_test/geometry.rs`
- Removed UI-local duplicate hit-test implementation:
  - `src/ui/phase0_shell/manipulate/hit_test.rs`
- Routed manipulate selection and hover through shared index:
  - `src/ui/phase0_shell/manipulate/mod.rs`
  - `src/ui/phase0_shell/mod.rs`
  - `src/ui/phase0_shell/tool_commands.rs`
  - `src/ui/phase0_shell/test_helpers.rs`
- Exposed model module and unit tests:
  - `src/model/mod.rs`
  - `src/model/hit_test_tests.rs`

Shipped tests:

- Behavioural (`rstest-bdd`):
  - `tests/features/hit_test.feature`
  - `tests/hit_test_bdd.rs`
- GPUI integration:
  - `tests/gpui_hit_test_service.rs`

Shipped documentation updates:

- `docs/gauss-architecture-design.md`
- `docs/users-guide.md`
- `docs/roadmap.md`

Gate evidence:

- `/tmp/fmt-gauss-0-5-4-shared-hit-test-service.out`
- `/tmp/markdownlint-gauss-0-5-4-shared-hit-test-service.out`
- `/tmp/nixie-gauss-0-5-4-shared-hit-test-service.out`
- `/tmp/check-fmt-gauss-0-5-4-shared-hit-test-service.out`
- `/tmp/lint-gauss-0-5-4-shared-hit-test-service.out`
- `/tmp/test-gauss-0-5-4-shared-hit-test-service.out`

Retrospective:

- The shared index seam keeps adapter code small and deterministic while
  allowing future acceleration work to stay local to model internals.
- Cross-layer tests now lock behaviour at model, scenario, and GPUI event-flow
  levels, which lowers regression risk for upcoming tool-framework work.
