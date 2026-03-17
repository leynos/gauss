# Consolidate integration test targets to reduce compile churn

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`,
`Surprises & Discoveries`, `Decision Log`, and
`Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: DRAFT (2026-03-14)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Gauss currently places dozens of top-level `.rs` files under `tests/`. Cargo
builds each top-level integration test target as its own crate, which means the
repository pays repeated compile and link overhead during `cargo test --no-run`
and the test phase of `make test`.

After this plan is implemented, Gauss will keep the same behavioural coverage
while reducing the number of top-level integration test crates by grouping
related scenarios into a small set of domain-focused binaries. Success is
observable when:

- the top-level `tests/*.rs` target count is reduced from the current `56` to a
  materially smaller grouped set;
- the `39` current `gpui_*.rs` integration targets are consolidated into a few
  larger GPU-accelerated UI (GPUI)-focused binaries by feature area;
- non-GPUI integration tests stay readable and do not get mixed into
  GPUI-specific setup unnecessarily; and
- `cargo test --workspace --no-run`, `make test`, and the standard formatting
  and lint gates all succeed after the consolidation.

## Constraints

- Preserve existing behavioural coverage. This task changes test packaging, not
  what the tests assert.
- Keep GPUI-specific tests clearly separated from pure model/SVG tests.
- Keep files under the repository 400-line cap; if a merged test target grows
  too large, split by domain instead of creating a new monolith.
- Reuse existing helpers in `tests/common`, `tests/select_tool_bdd/`, and other
  support modules instead of duplicating setup code.
- Preserve current test harness choices:
  - `#[gpui::test]` for windowing and platform wiring,
  - `rstest-bdd` for behaviour-driven development (BDD) scenarios,
  - ordinary Rust tests for integration checks that do not need GPUI.
- Avoid changing application code unless a narrow testing seam is genuinely
  required to keep the merged tests readable.
- Do not start implementation until a maintainer approves the pull request.

## Tolerances (exception triggers)

- Scope tolerance: if consolidation requires changing more than 20 top-level
  test files plus their supporting helper modules, stop and sequence the work
  into multiple commits or follow-up plans.
- Readability tolerance: if any merged test target would exceed 400 lines
  without a clean module split, stop and group the domain differently.
- Coverage tolerance: if a proposed merge would force GPUI-only setup into a
  pure model/SVG test path, stop and keep those tests separate.
- Tooling tolerance: if `nextest` or existing continuous integration (CI)
  filtering depends on the exact current test target names, stop and record the
  required CI updates before renaming targets.
- Failure tolerance: if merged crates expose order-dependence or shared-state
  leaks that current isolated crates mask, stop and fix the test isolation
  problem before proceeding further.

## Risks

- Risk: merged crates may create symbol or helper name collisions.
  Mitigation: convert repeated `mod`-local helpers into explicit nested modules
  with clear domain names.

- Risk: a few very large test crates may become hard to navigate.
  Mitigation: use domain-focused entry files such as `gpui_history.rs` or
  `gpui_selection.rs` that re-export smaller submodules.

- Risk: some tests may only appear independent because they currently compile as
  separate crates. Mitigation: run merged targets repeatedly and under nextest
  after each grouping step to expose hidden shared-state problems early.

- Risk: contributor workflows may rely on old test target names. Mitigation:
  document the new grouped targets in the final change and update any helper
  notes or CI filters in the same series.

## Progress

- [x] (2026-03-14) Counted `56` top-level integration test targets in
  `tests/*.rs`.
- [x] (2026-03-14) Counted `39` top-level `gpui_*.rs` targets and `17`
  top-level non-GPUI targets.
- [x] (2026-03-14) Confirmed additional nested helper modules live under
  `tests/common`, `tests/select_tool_bdd`, `tests/command_unit_tests`, and
  other support directories.
- [x] (2026-03-14) Drafted this ExecPlan.
- [ ] Await maintainer approval of the pull request before implementation.

## Surprises and discoveries

- The current integration suite is already partway modularized: many folders
  under `tests/` are helper-only modules, which means the consolidation work is
  largely about reusing existing support code instead of inventing a new test
  architecture.
- The biggest consolidation opportunity is the GPUI suite. Those `39` separate
  crates all compile the desktop app and its test-support dependencies.
- The non-GPUI integration targets are already fairly well grouped by feature
  family, so they may only need modest consolidation or may even stay mostly as
  they are if the GPUI reduction delivers enough win.

## Decision log

- 2026-03-14: Treat GPUI target consolidation as the primary goal and keep
  non-GPUI consolidation secondary. Rationale: the GPUI targets repeatedly pull
  the heaviest dependency tree and therefore dominate compile churn.

- 2026-03-14: Prefer a few feature-area entry crates over one giant test crate.
  Rationale: this preserves discoverability, keeps files under the line cap,
  and avoids reintroducing long-form monoliths in the test tree.

## Context and orientation

The current top-level integration targets include:

- GPUI shell and UI tests such as `tests/gpui_open_dialog.rs`,
  `tests/gpui_save_dialog.rs`, `tests/gpui_selection_history.rs`, and
  `tests/gpui_window_controls.rs`.
- Non-GPUI behaviour tests such as `tests/command_bdd.rs`,
  `tests/pen_tool_bdd.rs`, `tests/resource_store_bdd.rs`, and
  `tests/golden_round_trip.rs`.

Likely grouped target families are:

- `tests/gpui_shell.rs` for window and chrome behaviour;
- `tests/gpui_selection.rs` for selection, drag, resize, and reorder flows;
- `tests/gpui_history.rs` for undo/redo and history grouping flows;
- `tests/gpui_file_io.rs` for open/save and metadata round-trip flows;
- `tests/gpui_tooling.rs` for tool activation, edge toggles, and keybindings.

The exact grouping may change during implementation, but each final crate
should have a clear domain and a bounded size.

## Plan of work

### Stage A: classify every current integration target

Build a mapping from the current `tests/*.rs` files to feature areas. Mark each
target as one of:

- GPUI shell/windowing,
- GPUI editing interaction,
- non-GPUI model behaviour,
- SVG round-trip/import/export,
- BDD scenario entrypoint.

Validation gate:

- A checked-in mapping note or the `Progress` section names the final grouping
  for every current top-level target.

### Stage B: merge GPUI targets by feature area

Create grouped GPUI test entry files and move the existing test bodies behind
`mod` declarations or nested modules. Keep shared setup in helper modules
instead of repeating it across the new grouped crates.

Validation gate:

- Each merged crate stays under the 400-line cap at the top level by delegating
  to submodules.
- `cargo test --test <group-name>` passes for every new grouped GPUI target.

### Stage C: clean up non-GPUI targets where it helps

Only consolidate non-GPUI targets where repeated setup or domain overlap makes
the result clearer and cheaper. Leave already-coherent standalone targets alone
if merging would blur domain boundaries.

Validation gate:

- Non-GPUI targets still reflect clear feature seams.
- Shared helpers replace duplication rather than inline copy/paste.

### Stage D: update developer and CI-facing references

If CI, scripts, or docs refer to retired test target names, update them in the
same change. Ensure contributors can still run focused suites with clear
commands.

Validation gate:

- Any target-name changes are reflected in docs or scripts in the same commit.

### Stage E: rerun the full gate stack

Run the standard repository gates plus a build-only test pass that proves the
target count reduction changed the compiled test graph rather than only moving
code around.

Validation gate:

- `cargo test --workspace --no-run`
- `make fmt`
- `make markdownlint`
- `make nixie`
- `make check-fmt`
- `make lint`
- `make test`
- `git diff --check`

## Outcomes & retrospective

Pending. Record the before/after top-level integration target counts, the final
group names, any CI or nextest filter changes, and whether the consolidation
reduced the observed test-compilation cost.
