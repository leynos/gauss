# Define Resource Stores for EngineState (0.2.3)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

No `PLANS.md` exists in this repository.

## Purpose / Big Picture

Gauss Phase 0 needs concrete style and resource storage so the editor can carry
shared SVG resources (gradients, patterns, symbols) through open/save flows and
prepare for Phase 4 colour and effects work. Success is observable when:

- `EngineState` stores concrete `StyleStore` and `ResourceStore` data.
- Paint styles can reference gradients and patterns.
- SVG import/export round-trips `<defs>` resources and `url(#...)` paint
  references.
- Missing `url(#...)` references fail safely.
- Unit tests (`rstest`), behavioural tests (`rstest-bdd` 0.5.0), and GPUI
  tests pass for happy and unhappy paths.

## Constraints

- Follow roadmap item 0.2.3 in `docs/roadmap.md`.
- Follow architecture guidance in `docs/gauss-architecture-design.md` section
  20 and related model guidance in section 5.
- Keep model code independent of the Gauss Platform UI (GPUI) layer
  (`src/model`).
- Keep Rust module files under 400 lines.
- Use typed, explicit model identifiers for shared resources and styles.
- Preserve compatibility for existing callers where practical (for example,
  keep compatibility wrappers for SVG import/export and existing style
  constructors).
- Do not silence lint rules unless there is a narrow, documented reason.
- Validate through `make check-fmt`, `make lint`, and `make test`.
- Update user-facing documentation in `docs/users-guide.md` for behaviour
  changes.
- Record design decisions in `docs/gauss-architecture-design.md`.
- Mark roadmap item 0.2.3 as done once complete.

## Tolerances (Exception Triggers)

- Scope: if implementation requires more than 35 files or more than 1800 lines
  changed (insertions + deletions), stop and re-evaluate the approach.
- API surface: if existing public model APIs must be removed (rather than
  extended), stop and re-evaluate compatibility risk.
- Dependencies: if new crates beyond test-layer requirements are needed, stop
  and re-evaluate.
- Test iterations: if full quality gates fail more than three consecutive
  cycles without reducing failures, stop and escalate with findings.
- Ambiguity: if resource semantics conflict with existing SVG behaviour, prefer
  SVG fidelity and document the decision.

## Risks

- Risk: introducing non-solid paints could break existing colour controls.
  Mitigation: keep helper methods for solid paint access and update GPUI style
  controls/tests.
- Risk: import/export logic for `<defs>` parsing could regress simple SVG
  behaviour. Mitigation: preserve compatibility wrappers and add round-trip
  tests for both resource and non-resource documents.
- Risk: missing resource references could silently degrade rendering.
  Mitigation: fail import with explicit error and keep in-memory document
  unchanged on open failure.
- Risk: `rstest-bdd` version drift can break macros. Mitigation: bump to
  `rstest-bdd`/`rstest-bdd-macros` 0.5.0 and validate with behavioural tests.

## Progress

- [x] (2026-02-07) Locate model, SVG, and UI integration points using
  `grepai` semantic search and `leta` symbol navigation.
- [x] (2026-02-07) Implement concrete `ResourceStore` and `StyleStore` types
  with typed IDs and deterministic lookup semantics.
- [x] (2026-02-07) Introduce `Paint` enum and migrate rendering, style
  controls, and draw defaults from optional solid colours.
- [x] (2026-02-07) Implement SVG import/export resource-aware APIs with
  compatibility wrappers and explicit missing-reference errors.
- [x] (2026-02-07) Wire open/save flows to persist and hydrate resources in
  `Phase0Shell`.
- [x] (2026-02-07) Add/extend unit tests, behavioural tests (`rstest-bdd`),
  and GPUI tests for happy and unhappy paths.
- [x] (2026-02-07) Update architecture design docs, user guide, and roadmap.
- [x] (2026-02-07) Run full quality gates and confirm success.

## Surprises & Discoveries

- Clippy rule `self_named_module_files` rejected `src/svg/import.rs` when a
  sibling `src/svg/import/` directory was introduced. Import code was split
  into `src/svg/import/mod.rs`, `path_data.rs`, and `resource_tags.rs`.
- `make test` was initially long-running due to large test compilation; a fresh
  run completed successfully after stabilizing code shape.
- Existing paint code assumed solid colours; adding `Paint::as_solid()` and
  `Paint::from_solid()` kept non-resource UI behaviour stable while enabling
  typed references.

## Decision Log

- Decision: represent stroke/fill as `Paint` enum (`None`, `Solid`, `Gradient`,
  `Pattern`) instead of `Option<Rgba>`. Rationale: this keeps the model typed
  for shared resources while preserving simple solid colour paths. Date/Author:
  2026-02-07 (assistant)

- Decision: keep compatibility APIs (`import_svg`, `export_svg`,
  `PaintStyle::new`) while introducing resource-aware APIs
  (`import_svg_with_resources`, `export_svg_with_resources`,
  `PaintStyle::new_with_paint`). Rationale: reduce blast radius and permit
  incremental migration. Date/Author: 2026-02-07 (assistant)

- Decision: fail import on unresolved `url(#id)` via
  `SvgImportError::MissingReferencedResource`. Rationale: avoid silent data
  loss and present deterministic error behaviour. Date/Author: 2026-02-07
  (assistant)

- Decision: store resources with typed `slotmap` keys and explicit SVG ID
  lookup maps. Rationale: stable internal references plus deterministic SVG
  round-trip IDs. Date/Author: 2026-02-07 (assistant)

## Outcomes & Retrospective

- Implemented concrete style/resource stores for gradients, patterns, and
  symbols.
- Updated model paint representation and all key rendering/control call sites.
- Implemented resource-aware SVG import/export and open/save wiring.
- Added and updated tests across unit, BDD, and GPUI layers, including unhappy
  paths for missing resource references.
- Updated architecture and user documentation, and marked roadmap 0.2.3 done.

## Context and Orientation

Primary files touched:

- `src/model/resource_store.rs`
- `src/model/style_store.rs`
- `src/model/path.rs`
- `src/svg/export.rs`
- `src/svg/import/mod.rs`
- `src/svg/import/path_data.rs`
- `src/svg/import/resource_tags.rs`
- `src/ui/phase0_shell/file_dialogs.rs`
- `src/ui/phase0_support.rs`
- `tests/resource_store_bdd.rs`
- `tests/features/resource_store.feature`
- `tests/gpui_open_dialog.rs`

Documentation updates:

- `docs/gauss-architecture-design.md`
- `docs/users-guide.md`
- `docs/roadmap.md`
- `docs/execplans/0-2-3-define-resource-stores.md`

## Plan of Work (Executed)

Stage A: discovery and design alignment

- Locate all state, paint, and SVG read/write entry points.
- Confirm roadmap and architecture requirements for 0.2.3.

Stage B: model implementation

- Implement `ResourceStore` and `StyleStore` with typed IDs and lookup maps.
- Introduce `Paint` enum and maintain compatibility constructors/helpers.

Stage C: pipeline integration

- Add resource-aware SVG import/export APIs.
- Wire open/save paths to hydrate and persist resources.
- Update rendering and style controls for the new paint model.

Stage D: test and documentation completion

- Add `rstest` unit tests, `rstest-bdd` behavioural scenarios, and GPUI tests.
- Update architecture and user docs; mark roadmap item done.
- Run full quality gates.

## Validation and Acceptance

Run commands from repository root:

```shell
project="$(basename "$PWD")"
branch="$(git branch --show)"
make check-fmt | tee "/tmp/check-fmt-${project}-${branch}.out"
make lint | tee "/tmp/lint-${project}-${branch}.out"
make test | tee "/tmp/test-${project}-${branch}.out"
```

Expected outcomes:

- `make check-fmt` exits 0.
- `make lint` exits 0.
- `make test` exits 0 and includes passing resource coverage:
  - `model::resource_store_tests::*`
  - `svg::import_tests::imports_resource_defs_and_paint_refs`
  - `svg::import_tests::reports_missing_resource_refs`
  - `tests/resource_store_bdd.rs` scenarios
  - `tests/gpui_open_dialog.rs` resource happy/unhappy tests

## Idempotence and Recovery

All changes are source controlled and can be re-run safely. If any gate fails:

- inspect the corresponding `/tmp/*.out` log file,
- apply the minimal correction,
- rerun only the failed gate first,
- rerun the full gate sequence before finalizing.
