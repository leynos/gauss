# Implement Stable ID Generation (0.2.2)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

No `PLANS.md` exists in this repository.

## Purpose / Big Picture

Gauss needs stable, generational IDs for document objects so that the same
objects keep the same identifiers across frames, which is required for the
future AccessKit (the accessibility toolkit used by GPUI, the GPU-accelerated
UI framework) accessibility tree. Success is observable when document objects
are created with generational IDs, those IDs remain stable across UI frames and
reorder operations, and tests (unit, behaviour-driven development (BDD), and
GPUI integration) prove the stability and regeneration behaviour.

## Constraints

- Follow the architecture guidance in `docs/gauss-architecture-design.md`
  section 5.1 and the roadmap item 0.2.2.
- Use generational IDs (`slotmap` or equivalent) and wrap them in explicit
  newtypes to prevent mixing.
- Follow ADR 003 ("slotmap for ShapeId and AccessKit ID mapping"). See
  `docs/adr-003-slotmap-shapeid-accesskit-id-mapping.md`.
- Maintain GPUI independence in `src/model`; any GPUI code lives in `src/ui`.
- Keep every Rust module under 400 lines; split files if required.
- Use en-GB-oxendict spelling in documentation and comments.
- Do not silence clippy lints unless absolutely necessary.
- Avoid `unsafe` unless there is no reasonable alternative; document any
  usage with a `SAFETY` comment if it becomes necessary.
- Commit after each logical change and gate each commit with:

  - `make check-fmt`
  - `make lint`
  - `make test`

- Use Makefile targets and `tee` log files as described in `AGENTS.md`.
- Update `docs/users-guide.md` if user-visible behaviour or UI changes.
- Record design decisions in the design document.
- Mark roadmap item 0.2.2 as done when the feature is complete.

## Tolerances (Exception Triggers)

- Scope: if the change requires modifying more than 20 files or more than 800
  net lines of code, stop and escalate.
- Interface: if a public API in `gauss::model` or `gauss::ui` must change in a
  way that breaks downstream callers, stop and escalate.
- Dependencies: adding one new dependency is allowed only if it is
  `slotmap` or `generational-arena` (the requirement). Any additional new
  dependency requires escalation.
- Tests: if `make test` fails more than two consecutive times after
  adjustments, stop and escalate with failure details.
- Ambiguity: if stable ID scope is unclear (document IDs vs UI IDs), stop and
  present options with trade-offs.

## Risks

- Risk: Converting `Document` storage to generational IDs changes many call
  sites and tests. Severity: medium. Likelihood: high. Mitigation: introduce
  small, explicit `Document` APIs (`insert`/`remove`/`lookup`) and update call
  sites in one focused commit.

- Risk: GPUI tests assume deterministic IDs via `Uuid`; refactoring may break
  test fixtures. Severity: medium. Likelihood: high. Mitigation: create
  deterministic test helpers for stable IDs and update fixtures to use them.

- Risk: AccessKit ID stability requires a clear mapping from document IDs to
  `u64` node IDs. Severity: medium. Likelihood: medium. Mitigation: add explicit
  conversion helpers and tests that lock the mapping.

- Risk: Code Graph Model Context Protocol (MCP) is unavailable in this
  environment. Severity: low. Likelihood: high. Mitigation: fall back to `rg`
  and manual code inspection; record this in `Surprises & Discoveries` if it
  happens.

## Progress

- [x] (2026-01-18 18:30Z) Inspect docs and code paths related to IDs and
  document storage; confirm scope.
- [x] (2026-01-18 19:05Z) Decide on generational ID approach and record the
  decision in the design document.
- [x] (2026-01-18 20:25Z) Implement stable ID generation and update document
  storage and call sites.
- [x] (2026-01-18 20:45Z) Add unit, BDD, and GPUI tests for ID stability.
- [x] (2026-01-18 22:15Z) Update documentation and roadmap; run full quality
  gates; commit changes.

## Surprises & Discoveries

- Observation: Code Graph MCP resources are not available via
  `list_mcp_resources`/`list_mcp_resource_templates` in this environment.
  Evidence: both commands returned empty lists. Impact: proceed with `rg`-based
  codebase discovery instead.
- Observation: Stable ID changes required updates across more than 20 files,
  exceeding the plan's tolerance threshold; proceeding was necessary to keep
  the model and GPUI layers consistent.

## Decision Log

- Decision: implement `ShapeId` as a `slotmap` key and map AccessKit IDs via
  `KeyData::as_ffi`/`from_ffi`, with document-owned ID allocation. Rationale:
  `slotmap` provides generational keys without custom plumbing, preserves
  stable IDs across reorder/undo, and offers a stable `u64` mapping for
  AccessKit node IDs. See ADR 003
  (`docs/adr-003-slotmap-shapeid-accesskit-id-mapping.md`). Date/Author:
  2026-01-18 (assistant)

## Outcomes & Retrospective

- Implemented generational `ShapeId` values backed by `slotmap`, with stable
  AccessKit mappings, and updated document/command/UI call sites.
- Added unit, BDD, and GPUI coverage for stable IDs and AccessKit
  round-tripping.
- Updated architecture and roadmap documentation, and validated formatting,
  lint, and test gates.

## Context and Orientation

`ShapeId` is currently a `Uuid` newtype defined in `src/model/path.rs`. The
`Document` structure in `src/model/document.rs` stores shapes in a
`Vec<Shape>`, which provides ordering but does not provide generational ID
semantics. Several GPUI tests and test helpers construct specific `ShapeId`
values using `Uuid` to keep IDs deterministic. AccessKit integration is not yet
wired, but stable node IDs are required and documented in
`docs/accesskit-based-accessibility-in-gpui.md` and
`docs/gauss-architecture-design.md` section 5.1.

Relevant files and modules:

- `src/model/path.rs` (defines `ShapeId` and geometry types)
- `src/model/document.rs` (document storage and access)
- `src/model/ops.rs` and `src/model/command/*` (mutations using IDs)
- `src/ui/phase0_shell/*` (model integration and GPUI behaviours)
- `tests/` and `crates/test_support/` (fixtures and GPUI tests)
- `docs/gauss-architecture-design.md` (design requirements)
- `docs/roadmap.md` (roadmap entry to mark done)
- `docs/users-guide.md` (update if behaviour or UI changes)

## Plan of Work

Stage A: confirm scope and choose approach (no code changes)

- Read `docs/gauss-architecture-design.md` section 5.1 and confirm the stable
  ID expectations for document objects and AccessKit node IDs.
- Use Code Graph Model Context Protocol (MCP) if available; if not, document
  the absence and use `rg -n "ShapeId"` and `rg -n "Document"` to list all
  impacted files.
- Decide whether to use `slotmap` or `generational-arena` based on the
  requirement and ergonomics for ordering. Record the decision in
  `docs/gauss-architecture-design.md` (and optionally an Architecture Decision
  Record (ADR) entry if created).
- Define how a `ShapeId` (generational) maps to an AccessKit `u64` node ID.

Go/no-go: if the scope demands a full node tree redesign beyond shape IDs, stop
and escalate.

Stage B: scaffolding and tests (small, verifiable diffs)

- Add a stable ID helper module (e.g., `src/model/stable_id.rs`) that defines
  the generational ID newtypes and conversion helpers.
- Update test support to create deterministic IDs without `Uuid` (for example,
  a `TestShapeIdGenerator` fixture or a `ShapeId::from_raw` helper limited to
  tests).
- Add unit test scaffolding using `rstest` that exercises ID creation,
  reuse-after-delete, and conversion to AccessKit node IDs.
- Add a new BDD feature file under `tests/features/` describing stable ID
  behaviour and scaffold `#[scenario]` bindings.

Go/no-go: if scaffolding requires more than a new helper module and small test
adjustments, stop and re-evaluate tolerances.

Stage C: implementation (minimal change to satisfy tests)

- Replace `ShapeId`’s `Uuid` backing with a generational key type and update
  its API (`new`, `into_inner`, etc.) accordingly.
- Update `Document` to use a generational store (for example,
  `slotmap::SlotMap<ShapeId, Shape>` plus a `Vec<ShapeId>` for draw order), and
  add explicit APIs:
  - `insert_shape` (allocates ID, returns `ShapeId`)
  - `remove_shape` (frees ID, updates ordering)
  - `shape`/`shape_mut` (lookup by ID)
  - `iter_in_draw_order` (ordered iteration)
- Update command and ops modules to use the new `Document` APIs rather than
  direct `Vec` access.
- Update GPUI code paths that create shapes (e.g. `src/ui/phase0_shell/draw/*`)
  to obtain IDs via the document insert method instead of direct construction.
- Ensure any AccessKit-related conversions use the stable mapping helper.

Go/no-go: if this requires a full node tree refactor or more than the
explicitly listed files, stop and escalate.

Stage D: hardening, documentation, cleanup

- Add unit tests (rstest) for:
  - ID stability across reorder operations.
  - Generation increments when an ID is removed and a slot is reused.
  - AccessKit node ID conversion stability.
- Add BDD tests using `rstest-bdd` to cover happy and unhappy paths, such as
  attempting to access a removed shape by ID.
- Add a GPUI test that creates a shape, records its `ShapeId`, triggers a
  re-render or mode change, and asserts the ID remains unchanged.
- Update `docs/gauss-architecture-design.md` with the chosen ID strategy and
  rationale.
- Update `docs/users-guide.md` if any user-visible behaviour changes (likely
  “no change”; explicitly state if there is none).
- Mark roadmap item 0.2.2 as done in `docs/roadmap.md`.

## Concrete Steps

All commands should run from the repository root.

1. Discover scope (Code Graph MCP if available; otherwise `rg`):

   ```bash
   rg -n "ShapeId|Document" -S src tests crates
   ```

2. Create or update stable ID module and adjust model/document code.

3. Update tests and fixtures; add new rstest and rstest-bdd tests.

4. Update documentation and roadmap.

5. Run format and lint gates with `tee` logging, using the recommended log
   naming scheme from `AGENTS.md`:

   ```bash
   make check-fmt | tee /tmp/check-fmt-$(get-project)-$(git branch --show).out
   make lint | tee /tmp/lint-$(get-project)-$(git branch --show).out
   make test | tee /tmp/test-$(get-project)-$(git branch --show).out
   ```

If documentation changed, also run:

```bash
make fmt | tee /tmp/fmt-$(get-project)-$(git branch --show).out
make markdownlint | tee /tmp/markdownlint-$(get-project)-$(git branch --show).out
make nixie | tee /tmp/nixie-$(get-project)-$(git branch --show).out
```

## Validation and Acceptance

- Unit tests: `cargo test` (via `make test`) passes, and the new unit tests
  specifically verify generational ID behaviour.
- Behaviour tests: `rstest-bdd` scenarios run under `make test` and cover
  stable ID behaviour, including error paths.
- GPUI tests: new `#[gpui::test]` coverage confirms IDs remain stable across
  UI interaction cycles.
- Lint/format: `make check-fmt` and `make lint` pass without warnings.
- Documentation: design decisions recorded, roadmap updated, and the user
  guide updated if behaviour changes.

## Idempotence and Recovery

All edits are source-controlled and can be reapplied safely. If a change breaks
tests, revert the last commit and re-run the test suite to isolate the failure.
Avoid destructive commands such as `git reset --hard`.

## Artifacts and Notes

Expected new or modified artifacts:

- `src/model/stable_id.rs` (or the chosen equivalent) for generational ID
  definitions.
- Updated `src/model/path.rs` and `src/model/document.rs`.
- New tests under `tests/` or `crates/test_support/` (as appropriate).
- Updated `docs/gauss-architecture-design.md` decision entry.
- Updated `docs/roadmap.md` entry for 0.2.2.
- Updated `docs/users-guide.md` if applicable.

## Interfaces and Dependencies

Planned interfaces (names may be adjusted to fit conventions):

- `crate::model::ShapeId` as a newtype wrapper around a generational key.
- `crate::model::Document` methods:
  - `pub fn insert_shape(&mut self, shape: Shape) -> ShapeId`
  - `pub fn remove_shape(&mut self, id: ShapeId) -> Option<Shape>`
  - `pub fn shape(&self, id: ShapeId) -> Option<&Shape>`
  - `pub fn shape_mut(&mut self, id: ShapeId) -> Option<&mut Shape>`
  - `pub fn iter_in_draw_order(&self) -> impl Iterator<Item = &Shape>`
- `crate::model::A11yNodeId` (if introduced) with conversion from `ShapeId`
  that yields a stable `u64` for AccessKit.

Dependencies:

- Add `slotmap = "<version>"` (caret requirement) to `Cargo.toml` if not
  already present. If `generational-arena` is chosen instead, document why in
  the design document and update this plan accordingly.

## Revision note (required when editing an ExecPlan)

Initial draft created on 2026-01-18. No revisions yet.
