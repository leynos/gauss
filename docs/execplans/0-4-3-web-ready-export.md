# Implement web-ready SVG export (0.4.3)

This Execution Plan (ExecPlan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

No `PLANS.md` exists in this repository.

## Purpose / big picture

Roadmap item 0.4.3 requires a dedicated web-ready export path that removes
editor-specific Gauss metadata while still producing valid SVG output. After
this work, users can export a minimal SVG for web publishing without `gauss:*`
attributes, without the Gauss namespace declaration, and without persisted
Gauss metadata payloads.

Success is observable when:

- The normal Save flow still preserves metadata round-trip behaviour.
- The web-ready export flow strips Gauss metadata artefacts deterministically.
- The exported web-ready SVG remains valid and importable as plain SVG.
- Unit (`rstest`), behaviour-driven development (BDD) tests via
  `rstest-bdd` v0.5.0, and GPUI (GPU-accelerated user interface) integration
  tests cover happy paths, unhappy paths, and edge cases.
- `make check-fmt`, `make lint`, and `make test` pass.
- `docs/users-guide.md` and `docs/gauss-architecture-design.md` reflect the new
  behaviour.
- `docs/roadmap.md` marks 0.4.3 as done.

## Constraints

- Scope is limited to roadmap task 0.4.3 and its direct UI command wiring.
- Preserve existing metadata-preserving save/open behaviour from 0.4.1/0.4.2.
- Preserve valid SVG semantics for visible artwork (`<defs>`, `<path>`, paint,
  opacity, and viewBox output stay standards-compliant).
- Keep model-layer code GPUI-independent.
- Keep module files below the repository 400-line guideline by extracting helper
  modules where needed.
- Do not add dependencies unless explicitly approved.
- Use `rstest` for unit tests, `rstest-bdd` for behavioural tests, and
  `#[gpui::test]` for UI integration.
- Keep docs in en-GB-oxendict spelling and update architecture plus user-guide
  documentation when behaviour changes.
- Mark roadmap item 0.4.3 done only after implementation, docs updates, and all
  required gates pass.

## Tolerances (exception triggers)

- Scope: if implementation exceeds 18 files or 900 net changed lines, stop and
  re-evaluate before proceeding.
- API surface: if public APIs outside SVG export and Phase0 file-dialog action
  wiring must change, stop and re-evaluate.
- Ambiguity: if "minimal SVG" requires transformations beyond metadata removal
  (for example geometry rewriting or structural optimization), stop and seek
  direction.
- Data retention: if web-ready export requirements conflict with preserving
  third-party non-Gauss `<metadata>` payloads, stop and confirm policy.
- Stability: if full-gate failures persist for more than three repair cycles,
  stop and escalate with logs and options.

## Risks

- Risk: accidentally regressing normal Save metadata round-trip.
  Mitigation: keep preserve-mode as the default export behaviour and add
  explicit regression coverage in unit, BDD, and GPUI layers.
- Risk: malformed XML output if namespace/metadata stripping is partial.
  Mitigation: centralize metadata policy checks in export writer helpers and
  assert final output shape via deterministic string tests and re-import tests.
- Risk: UI command drift (Save and web-ready export sharing incorrect path).
  Mitigation: split save mode explicitly in file-dialog workflow and test both
  command routes end-to-end.
- Risk: test brittleness due to path prompts in GPUI headless tests.
  Mitigation: reuse existing save-dialog helper patterns and temp-file guards.

## Progress

- [x] (2026-02-25) Verified branch context and repository instructions.
- [x] (2026-02-25) Loaded `execplans`, `grepai`, and `leta` skills.
- [x] (2026-02-25) Used a Spark-style explorer team to gather roadmap,
  architecture, export-code, and testing-context findings in parallel.
- [x] (2026-02-25) Ran `grepai search` and `grepai trace` to locate current
  export wiring and callers.
- [x] (2026-02-25) Ran `leta` symbol navigation to map exporter and UI action
  entry points.
- [x] (2026-02-25) Drafted this ExecPlan at
  `docs/execplans/0-4-3-web-ready-export.md`.
- [x] (2026-02-25) Verified Lane A outcomes now present in branch:
  `export_svg_with_resources_web_ready(_checked)` plus unit/golden coverage for
  metadata stripping and validation paths.
- [x] (2026-02-25) Added `rstest-bdd` web-ready behavioural coverage with
  happy/unhappy/edge scenarios:
  - `crates/gauss-svg/tests/features/web_ready_export.feature`
  - `crates/gauss-svg/tests/web_ready_export_bdd.rs`
- [x] (2026-02-25) Updated documentation for web-ready behaviour and policy:
  - `docs/users-guide.md`
  - `docs/gauss-architecture-design.md`
- [x] (2026-02-25) Marked roadmap item `0.4.3` complete in `docs/roadmap.md`.
- [x] (2026-02-25) Wired and exposed the web-ready export command in Phase0
  shell actions (new `ExportSvgWebReady` action, chrome button, and file-dialog
  export intent split).
- [x] (2026-02-25) Added GPUI coverage for web-ready export happy/unhappy
  command flows in `tests/gpui_file_io_save_dialog.rs`.
- [x] (2026-02-25) Ran docs and lint gates successfully with logs:
  - `/tmp/fmt-gauss-0-4-3-web-ready-export.out`
  - `/tmp/markdownlint-gauss-0-4-3-web-ready-export.out`
  - `/tmp/nixie-gauss-0-4-3-web-ready-export.out`
  - `/tmp/check-fmt-gauss-0-4-3-web-ready-export.out`
  - `/tmp/lint-gauss-0-4-3-web-ready-export.out`
- [x] (2026-02-26) Full gate replay completed successfully:
  - `make check-fmt` (`/tmp/check-fmt-gauss-0-4-3-web-ready-export.out`)
  - `make lint` (`/tmp/lint-gauss-0-4-3-web-ready-export.out`)
  - `make test` (`/tmp/test-gauss-0-4-3-web-ready-export.out`)
- [x] (2026-02-26) Documentation gates replay completed successfully after the
  final roadmap consistency update:
  - `make fmt` (`/tmp/fmt-gauss-0-4-3-web-ready-export.out`)
  - `make markdownlint`
    (`/tmp/markdownlint-gauss-0-4-3-web-ready-export.out`)
  - `make nixie` (`/tmp/nixie-gauss-0-4-3-web-ready-export.out`)
- [x] (2026-02-26) Marked the duplicate roadmap tracking entry `1.7.3` as done
  so it now matches completed phase-0 metadata and web-ready export work.
- [x] (2026-02-26) Applied post-review refactors and coverage follow-ups:
  `ExportMode` now drives metadata policy, GPUI/BDD pattern-reference failure
  coverage matches gradient coverage, and wording consistency updates landed in
  this ExecPlan.
- [x] (2026-02-26) Closed a web-ready defs leak: `gauss:*` attributes on
  pattern/symbol resources are now filtered during web-ready export while
  metadata-preserving export continues to emit them.
- [x] (2026-02-26) Reconciled post-review follow-ups against current code:
  Figure 5.1 now uses `CanvasSize` signatures, file-dialog save/export paths
  share one canvas-size binding with a tracked FIXME, metadata-mode override
  behaviour is explicitly documented, and duplicated BDD `line_shape` fixtures
  were centralized into `crates/test_support`.

## Surprises & discoveries

- Export now includes explicit web-ready helper APIs:
  `export_svg_with_resources_web_ready()` and
  `export_svg_with_resources_web_ready_checked()`.
- Phase0 shell now has a dedicated web-ready command path:
  `ExportSvgWebReady` action, top-bar trigger, and `SaveIntent::WebReady`.
- Existing metadata-preserving test coverage remained reusable, and web-ready
  behavioural coverage landed as a separate BDD harness:
  - `crates/gauss-svg/src/svg/export/metadata_tests.rs`
  - `crates/gauss-svg/tests/metadata_round_trip_bdd.rs`
  - `crates/gauss-svg/tests/web_ready_export_bdd.rs`
  - `crates/gauss-svg/tests/golden_round_trip.rs`
  - `tests/gpui_file_io_save_dialog.rs`

## Decision log

- Decision: implement web-ready behaviour as an explicit export policy mode,
  not a separate exporter implementation. Rationale: keeps one code path for
  SVG emission and reduces divergence risk. Date/Author: 2026-02-25 (assistant,
  draft)

- Decision: preserve current `SaveSvg` semantics as metadata-preserving export,
  and add a dedicated web-ready action. Rationale: avoids breaking existing
  workflows and tests while satisfying roadmap command requirements.
  Date/Author: 2026-02-25 (assistant, draft)

- Decision: define web-ready output as removing all Gauss-specific metadata
  artefacts (Gauss namespace declaration, `gauss:*` attributes, metadata block
  carried by `EngineState::gauss_metadata_block`). Rationale: aligns with
  roadmap 0.4.3 and architecture terminology for minimal web-ready SVG.
  Date/Author: 2026-02-25 (assistant, draft)

- Decision: if selective retention of non-Gauss third-party `<metadata>` is
  required, treat that as a follow-on task unless explicitly requested during
  implementation. Rationale: current stored metadata block is opaque and not
  partitioned by namespace; selective filtering would require additional parser
  work. Date/Author: 2026-02-25 (assistant, draft)

- Decision: keep web-ready behavioural coverage in a dedicated feature/test
  pair (`web_ready_export.feature`, `web_ready_export_bdd.rs`) instead of
  extending `metadata_round_trip` files. Rationale: avoids exceeding repository
  file-size guidance while keeping metadata-preserving and metadata-stripping
  behaviours clearly separated. Date/Author: 2026-02-25 (assistant, lane C)

- Decision: mark both roadmap surfaces (`0.4.3` and duplicate `1.7.3`
  metadata/export item) as done for consistency. Rationale: these entries refer
  to the same now-implemented capability; leaving one unchecked creates false
  residual scope. Date/Author: 2026-02-26 (assistant, final closure)

## Context and orientation

Primary implementation points discovered with `grepai` and `leta`:

- Export seam and metadata writing:
  - `src/svg/export/mod.rs`
    - `ExportOptions`
    - `export_svg_with_metadata(_checked)`
    - `write_svg_header`
    - `write_metadata_block`
    - `write_shape_gauss_metadata`
- Save/open wiring:
  - `src/ui/phase0_shell/mod.rs` (`SaveSvg` action)
  - `src/ui/phase0_shell/view.rs` (`.on_action` bindings)
  - `src/ui/phase0_shell/chrome.rs` (Save button wiring)
  - `src/ui/phase0_shell/file_dialogs.rs` (`request_save`, `apply_save_path`)
- Existing test surfaces to extend:
  - `crates/gauss-svg/src/svg/export/metadata_tests.rs`
  - `crates/gauss-svg/tests/metadata_round_trip_bdd.rs`
  - `crates/gauss-svg/tests/features/metadata_round_trip.feature`
  - `crates/gauss-svg/tests/golden_round_trip.rs`
  - `tests/gpui_file_io_save_dialog.rs`
  - `tests/gpui_file_io_metadata_round_trip.rs`
- Documentation/bookkeeping:
  - `docs/gauss-architecture-design.md`
  - `docs/users-guide.md`
  - `docs/roadmap.md`

## Spark team workstreams

Use a three-lane Spark team during implementation:

- Lane A: Export policy and serializer refactor.
  Ownership: `src/svg/export/mod.rs` and export unit tests.
- Lane B: Command wiring and GPUI integration.
  Ownership: Phase0 shell actions, file dialogs, and GPUI tests.
- Lane C: Behavioural/golden/docs closure.
  Ownership: BDD scenarios, golden artefacts, architecture/user docs, roadmap.

All lanes must converge before final gate replay.

## Plan of work

Stage A: Add export metadata policy mode

- Extend `ExportOptions` with an explicit metadata policy enum (for example,
  `PreserveGaussMetadata` and `StripGaussMetadataForWeb`).
- Keep existing constructors defaulting to preserve mode to avoid breaking
  current callers.
- Gate namespace declaration, metadata block emission, and shape-level
  `gauss:*` attribute emission on policy.
- Keep paint/reference validation identical across both modes.

Validation checkpoint:

- Unit tests verify preserve mode still emits Gauss metadata and web-ready mode
  strips it.

Stage B: Add command-level web-ready export path

- Introduce a new action in Phase0 shell (for example, `ExportWebReadySvg`).
- Add action wiring in `view.rs` and a trigger in top-bar file actions (or
  equivalent command surface if existing UI guidance prefers action-only).
- Split file-dialog save execution path by export mode while keeping shared
  prompt/result handling.
- Ensure save result reporting (`last_saved_path`, `last_save_error`) remains
  consistent for both modes.

Validation checkpoint:

- Existing Save tests remain green.
- New GPUI tests prove web-ready command writes a file with stripped metadata.

Stage C: Expand unit and golden coverage

- Add `rstest` unit cases for:
  - happy: web-ready export strips `xmlns:gauss`, `gauss:id`, `gauss:name`,
    `gauss:locked`, `gauss:hidden`, and opaque `gauss:*` attrs.
  - unhappy: checked web-ready export still errors on missing gradient/pattern
    references.
  - edge: empty docs and shapes with null IDs still produce valid minimal SVG.
- Extend `crates/gauss-svg/tests/golden_round_trip.rs` with web-ready
  normalization assertions.
- Add one or more golden fixtures in `crates/gauss-svg/tests/golden/` for
  deterministic web-ready output.

Validation checkpoint:

- Golden tests prove deterministic output and idempotent web-ready
  export/import/export behaviour.

Stage D: Expand behavioural (BDD) coverage

- Extend `crates/gauss-svg/tests/features/metadata_round_trip.feature` and
  `crates/gauss-svg/tests/metadata_round_trip_bdd.rs` with web-ready
  scenarios:
  - happy: web-ready export strips Gauss metadata while preserving renderable
    path/style data.
  - unhappy: web-ready checked export reports missing referenced resources.
  - edge: web-ready output imports as plain SVG with default metadata state.

Validation checkpoint:

- `rstest-bdd` scenarios pass with new and existing metadata scenarios.

Stage E: Documentation and roadmap closure

- Update `docs/users-guide.md` with:
  - distinction between Save (metadata-preserving) and web-ready export,
  - what data is stripped,
  - user-visible command location.
- Update `docs/gauss-architecture-design.md` to record implementation outcome
  for section 10.1 and immediate next steps section 20.
- Mark roadmap entry `0.4.3` done in `docs/roadmap.md` (and update related
  checklist references if applicable).

Validation checkpoint:

- Documentation reflects implemented behaviour and command naming exactly.

Stage F: Gates, evidence, and commit hygiene

- Run required Rust gates and collect logs with branch-safe names.
- Because docs are updated, run docs quality gates as well.
- Commit in atomic slices, gating each commit before it is created.

Validation checkpoint:

- All required gates pass and evidence logs are retained in `/tmp`.

## Concrete steps and commands

Run from repository root:

```sh
project="$(basename "$PWD")"
branch="$(git branch --show)"
```

Discovery and tracing (implementation kickoff):

```sh
grepai search "web-ready SVG export metadata stripping" --json --compact

grepai trace callers "export_svg_with_metadata_checked" --json

leta workspace add "$PWD"
leta show src/svg/export/mod.rs:export_svg_with_metadata
leta show src/ui/phase0_shell/file_dialogs.rs:apply_save_path
```

Targeted development checks during implementation:

```sh
cargo test --workspace svg::export::metadata_tests | tee \
  "/tmp/test-unit-${project}-${branch}.out"

cargo test --workspace --test metadata_round_trip_bdd | tee \
  "/tmp/test-bdd-${project}-${branch}.out"

cargo test --workspace --test gpui_file_io_save_dialog | tee \
  "/tmp/test-gpui-save-dialog-${project}-${branch}.out"

cargo test --workspace --test gpui_file_io_metadata_round_trip | tee \
  "/tmp/test-gpui-metadata-round-trip-${project}-${branch}.out"
```

Required full gates before completion:

```sh
make check-fmt | tee "/tmp/check-fmt-${project}-${branch}.out"
make lint | tee "/tmp/lint-${project}-${branch}.out"
make test | tee "/tmp/test-${project}-${branch}.out"
```

Documentation gates (required when docs are modified):

```sh
make fmt | tee "/tmp/fmt-${project}-${branch}.out"
make markdownlint | tee "/tmp/markdownlint-${project}-${branch}.out"
make nixie | tee "/tmp/nixie-${project}-${branch}.out"
```

## Validation and acceptance

Implementation is accepted when all are true:

- Web-ready command exports valid SVG without Gauss metadata artefacts:
  - no `xmlns:gauss="https://gauss.dev/ns/metadata/1"`
  - no `gauss:*` attributes on shapes
  - no preserved Gauss metadata block from editor state
- Normal Save behaviour remains metadata-preserving and existing round-trip
  tests still pass.
- Unit tests (`rstest`) cover happy, unhappy, and edge export policy cases.
- Behaviour tests (`rstest-bdd` v0.5.0) cover happy, unhappy, and edge
  web-ready scenarios.
- GPUI tests cover both command routes and failure propagation.
- Golden tests include deterministic web-ready output normalization.
- `make check-fmt`, `make lint`, and `make test` succeed.
- `docs/users-guide.md` and `docs/gauss-architecture-design.md` are updated.
- `docs/roadmap.md` marks 0.4.3 as done.

## Idempotence and recovery

- Export-policy changes are deterministic and re-runnable.
- If a gate fails, inspect the corresponding `/tmp/*.out` log, apply minimal
  fix, rerun the failed gate, then rerun the full gate stack.
- If implementation scope breaches tolerances, stop and record options in the
  `Decision Log` before continuing.

## Outcomes & retrospective

Lane C completion update (2026-02-25):

- Added behavioural test coverage for web-ready export happy, unhappy, and edge
  paths via `rstest-bdd` feature scenarios.
- Updated user and architecture documentation with implemented web-ready policy,
  metadata stripping rules, and command-surface notes.
- Marked roadmap architecture-foundation task `0.4.3` complete.

Gate closure update (2026-02-26):

- Required Rust gates now pass in this branch, including a full workspace test
  replay.
- Documentation gates also pass after the final roadmap consistency edit.
- Roadmap tracking now consistently marks the completed web-ready export work
  in both phase-0 and phase-1 tracking sections.
