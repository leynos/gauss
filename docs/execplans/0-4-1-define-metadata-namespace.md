# Define Gauss metadata namespace (0.4.1)

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

No `PLANS.md` exists in this repository.

## Purpose / Big Picture

Roadmap item 0.4.1 establishes the namespace contract that all later metadata
work depends on. After this work, Gauss will have a documented and
implementation-backed metadata namespace policy for SVG load/save, with tests
that prove happy and unhappy paths at unit, behavioural (`rstest-bdd`), and
Gauss UI framework (GPUI) integration layers.

This creates the architectural boundary required by:

- `docs/roadmap.md` item 0.4.1.
- `docs/gauss-architecture-design.md` section 10.1.
- `docs/gauss-architecture-design.md` section 20.

## Constraints

- Keep scope to roadmap task 0.4.1 (namespace definition and policy), not full
  metadata round-trip implementation from 0.4.2.
- Preserve valid standard SVG output for visible artwork.
- Keep model-layer code GPUI-independent.
- Keep files under 400 lines; split modules where needed.
- Use British English (en-GB-oxendict) in docs.
- Add or update an architectural decision record (ADR) documenting namespace
  policy according to `docs/documentation-style-guide.md`.
- Validate with `make check-fmt`, `make lint`, and `make test`.
- Use `rstest` for unit tests, `rstest-bdd` (v0.5.0) for behavioural tests,
  and `#[gpui::test]` for integration tests.
- Update `docs/users-guide.md` for user-visible load/save behaviour changes.
- Record design implications in `docs/gauss-architecture-design.md` if policy
  details become architecture-significant.
- Mark roadmap entry 0.4.1 as done only after implementation and all quality
  gates pass.

## Tolerances (Exception Triggers)

- Scope: if work exceeds 20 files or 900 net lines, stop and re-evaluate.
- API surface: if public API signatures must change outside SVG/import/export
  and file-dialog boundaries, stop and re-evaluate.
- Dependencies: if a new crate is required, stop and re-evaluate.
- Ambiguity: if namespace policy cannot satisfy SVG compatibility and roadmap
  goals together, pause and present options.
- Test instability: if full gate failures persist for more than three repair
  cycles without trend improvement, stop and escalate with findings.

## Risks

- Risk: locking into a namespace form that conflicts with phase 0.4.2
  round-trip work. Mitigation: define namespace constants and parsing/writing
  seams now, then route all metadata handling through those seams.
- Risk: accidental rendering impact in other SVG viewers.
  Mitigation: restrict metadata to namespaced attributes or `<metadata>`
  content and assert that path/style rendering remains unchanged in tests.
- Risk: mismatched interpretation between docs and implementation.
  Mitigation: encode policy in ADR, users guide, architecture notes, and test
  assertions referencing the same namespace URI/prefix.
- Risk: GPUI open/save tests become flaky with filesystem prompts.
  Mitigation: follow existing `tests/gpui_save_dialog.rs` and
  `tests/gpui_open_dialog.rs` harness patterns and temp-file guards.

## Progress

- [x] (2026-02-10) Reviewed roadmap and architecture requirements for 0.4.1.
- [x] (2026-02-10) Located current SVG import/export and open/save integration
  points with `grepai` and `leta`.
- [x] (2026-02-10) Drafted this ExecPlan with staged implementation and test
  strategy.
- [x] (2026-02-10) Implemented namespace constants/policy in SVG load/save
  path via `src/svg/metadata.rs`, `src/svg/export/mod.rs`, and
  `src/svg/import/mod.rs`.
- [x] (2026-02-10) Added unit tests for namespace helpers and SVG import/export
  namespace behaviour.
- [x] (2026-02-10) Added `rstest-bdd` behavioural scenarios for namespace
  declaration and rejection policy in `tests/resource_store_bdd.rs`.
- [x] (2026-02-10) Added GPUI open/save tests for namespace happy and unhappy
  paths in `tests/gpui_open_dialog.rs` and `tests/gpui_save_dialog.rs`.
- [x] (2026-02-10) Added ADR 005 and updated architecture/user documentation.
- [x] (2026-02-10) Marked roadmap item 0.4.1 done.
- [x] (2026-02-10) Ran full quality gates and captured logs.

## Surprises & Discoveries

- Existing resource round-trip work (`0.2.3`) already provides a useful test
  template for 0.4.x metadata work in:
  - `src/svg/export/tests.rs`
  - `src/svg/import_tests.rs`
  - `tests/resource_store_bdd.rs`
  - `tests/gpui_open_dialog.rs`
  - `tests/gpui_save_dialog.rs`
- Export root generation currently emits only the SVG namespace in
  `src/svg/export/mod.rs` via `write_svg_header`, which is the natural point to
  add explicit Gauss namespace policy output.
- Rust raw string delimiters in test fixtures must use `r##"..."##` when
  embedded XML includes colour literals such as `"#000000"`; otherwise
  compilation fails before runtime assertions.

## Decision Log

- Decision: include both representation forms in policy, with a primary
  canonical form for this phase. Rationale: roadmap language allows either
  namespaced attributes or `<metadata>`; defining both avoids dead-end design
  while keeping one canonical path for deterministic tests and docs.
  Date/Author: 2026-02-10 (assistant, draft)

- Decision: keep 0.4.1 implementation minimal and seam-oriented, deferring full
  metadata payload round-trip to 0.4.2. Rationale: preserves roadmap sequencing
  and avoids scope bleed while still creating an enforceable namespace
  contract. Date/Author: 2026-02-10 (assistant, finalized)

- Decision: enforce canonical `gauss` prefix declaration during import even if
  Gauss namespace URI appears under a different prefix. Rationale: avoids alias
  drift and keeps deterministic metadata identity across load/save.
  Date/Author: 2026-02-10 (assistant, finalized)

## Outcomes & Retrospective

Implemented outcomes:

- Added namespace policy module at `src/svg/metadata.rs` with canonical prefix
  and URI constants, plus import validation helpers.
- Export now always emits canonical
  `xmlns:gauss="https://gauss.dev/ns/metadata/1"` declaration.
- Import now rejects invalid `gauss` namespace binding and rejects Gauss
  namespace usage without canonical `xmlns:gauss` declaration.
- Added unit coverage across metadata helper tests and SVG import/export tests.
- Added behavioural coverage in `tests/resource_store_bdd.rs` and
  `tests/features/resource_store.feature`.
- Added GPUI integration coverage in `tests/gpui_open_dialog.rs` and
  `tests/gpui_save_dialog.rs`.
- Added ADR 005 and updated architecture (`docs/gauss-architecture-design.md`),
  user guide (`docs/users-guide.md`), and roadmap (`docs/roadmap.md`).

Retrospective:

- The namespace policy seam is now in place for roadmap 0.4.2 metadata payload
  round-trip work with minimal refactoring pressure.
- Strict validation catches malformed namespace usage early, but this may
  reject previously tolerated SVG aliases intentionally.

## Context and Orientation

The relevant code and documentation surfaces are:

- SVG export root writing:
  - `src/svg/export/mod.rs` (`write_svg_header`, export entry points)
- SVG import pipeline:
  - `src/svg/import/mod.rs` (`import_svg_with_resources`)
  - `src/svg/import/resource_tags.rs`
  - `src/svg/import/resource_tag_attributes.rs`
- Existing SVG tests:
  - `src/svg/export/tests.rs`
  - `src/svg/import_tests.rs`
- Behavioural tests:
  - `tests/resource_store_bdd.rs`
  - `tests/features/resource_store.feature`
- GPUI open/save integration tests:
  - `tests/gpui_open_dialog.rs`
  - `tests/gpui_save_dialog.rs`
- Documentation targets:
  - `docs/adr-00x-*.md` (new ADR file required)
  - `docs/gauss-architecture-design.md`
  - `docs/users-guide.md`
  - `docs/roadmap.md`

## Plan of Work

Stage A: finalize metadata namespace policy and names

- Define the canonical Gauss metadata namespace URI and prefix in one place
  used by import and export code.
- Specify allowed representations for this phase:
  - namespaced attributes (`gauss:*`) on SVG elements, and/or
  - `<metadata>` block with Gauss namespaced payload.
- Add parser/writer helper boundaries so 0.4.2 can extend payload handling
  without reworking call sites.

Validation gate:

- Unit tests prove namespace constants, accepted forms, and rejection/handling
  of invalid namespace usage.

Stage B: wire namespace policy into export/import seams

- Update SVG header/export generation to emit namespace declaration according to
  policy.
- Update import path to recognize declared Gauss namespace form and route
  metadata handling through helper functions.
- Ensure non-Gauss and malformed namespace data follows explicit unhappy-path
  behaviour (error or ignore, as defined by policy).

Validation gate:

- `src/svg/export/tests.rs` and `src/svg/import_tests.rs` include happy and
  unhappy coverage for namespace declaration handling.

Stage C: behavioural and GPUI integration proof

- Add or extend feature scenarios in `tests/features/*.feature` and step
  definitions to prove namespace behaviour from user-observable workflows.
- Add GPUI save/open tests proving namespace declaration survives save/open and
  that invalid namespace conditions surface expected errors without corrupting
  in-memory state.

Validation gate:

- Behavioural tests pass under `rstest-bdd` 0.5.0 conventions.
- GPUI tests pass in headless test harness.

Stage D: documentation and completion bookkeeping

- Add ADR describing:
  - namespace URI and prefix,
  - why this choice was made,
  - alternatives considered (`gauss:*` attributes vs `<metadata>` block).
- Update `docs/users-guide.md` with user-visible save/open metadata behaviour.
- Update architecture doc if needed for persistent policy references.
- Mark roadmap 0.4.1 checkbox done.

Validation gate:

- Documentation linting/formatting checks pass and references are internally
  consistent.

## Concrete Steps

Run from repository root.

1. Discover and confirm implementation points (already done in draft phase):

    `grepai search "SVG import export metadata namespace policy" --json --compact`

    `leta grep "resource|metadata|svg" "src/svg" -k function,struct,module`

2. Implement namespace policy seams and tests in the files listed above.

3. Execute quality gates with persistent logs:

    `project="$(basename "$PWD")"`

    `branch="$(git branch --show)"`

    `make check-fmt | tee "/tmp/check-fmt-${project}-${branch}.out"`

    `make lint | tee "/tmp/lint-${project}-${branch}.out"`

    `make test | tee "/tmp/test-${project}-${branch}.out"`

Expected gate outcomes:

- `make check-fmt` exits 0.
- `make lint` exits 0 (no Clippy warnings).
- `make test` exits 0 with passing namespace-focused unit, BDD, and GPUI tests.

## Validation and Acceptance

Implementation is accepted when all of the following are true:

- Exported SVGs include the documented Gauss metadata namespace declaration in
  the canonical form.
- Import path recognizes canonical namespace usage and handles invalid/missing
  namespace conditions according to the defined unhappy-path policy.
- Unit tests (`rstest`) cover:
  - canonical namespace emission and parsing,
  - invalid namespace forms,
  - edge cases such as duplicate/unknown prefixes.
- Behaviour tests (`rstest-bdd`) cover:
  - happy path save/open namespace observability,
  - unhappy path behaviour with malformed or conflicting namespace input.
- GPUI tests cover:
  - save output containing namespace policy artefacts,
  - open error/reporting behaviour when policy is violated.
- ADR and user documentation are updated and consistent with code behaviour.
- `make check-fmt`, `make lint`, and `make test` all pass.
- Roadmap task `0.4.1` is marked done.

## Idempotence and Recovery

- File edits are idempotent and safe to reapply.
- If a gate fails, inspect the corresponding `/tmp/*-${branch}.out` log,
  apply a targeted fix, rerun the failed gate, then rerun all gates.
- If namespace policy decisions change during implementation, update this
  ExecPlan and ADR in the same commit that changes behaviour.

## Interfaces and Dependencies

Planned internal interfaces (exact names may be finalized during Stage A):

- `src/svg/metadata/mod.rs` or equivalent helper module for:
  - namespace constants (`GAUSS_METADATA_PREFIX`, `GAUSS_METADATA_NAMESPACE`)
  - validation helpers for accepted metadata forms
- Export path integration from `src/svg/export/mod.rs`.
- Import path integration from `src/svg/import/mod.rs` and
  `src/svg/import/resource_tags.rs`.

No new external dependencies are expected.

## Artefacts and Notes

Planned output artefacts:

- New/updated ADR file (`docs/adr-005-gauss-metadata-namespace.md` expected).
- Updated users guide section describing metadata namespace behaviour.
- Updated roadmap checkbox for 0.4.1.
- Test additions across unit, BDD, and GPUI layers.

## Revision Note

Initial draft created on 2026-02-10 to guide implementation of roadmap item
0.4.1 with explicit testing and documentation deliverables.

Updated on 2026-02-10 after implementation: status set to COMPLETE, progress
items closed, and outcomes documented with final file-level results.
