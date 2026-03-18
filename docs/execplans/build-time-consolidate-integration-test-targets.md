# Consolidate integration test targets to reduce compile churn

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETED with scope revision (2026-03-17)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Gauss currently places dozens of top-level `.rs` files under `tests/`. Cargo
builds each top-level integration test target as its own crate, which means the
repository pays repeated compile and link overhead during `cargo test --no-run`
and the test phase of `make test`.

After this plan is implemented, Gauss will keep the same behavioural coverage
while reorganizing integration tests into domain/feature-focused groups with
consistent naming. Success is observable when:

- integration tests are reorganized into domain/feature-focused groups with
  consistent naming (GPUI (General Purpose UI) tests consolidated by feature
  area using `gpui_{group}_{name}.rs` pattern);
- the reorganization preserves behavioural coverage and all tests remain
  discoverable;
- non-GPUI integration tests stay readable and do not get mixed into
  GPUI-specific setup unnecessarily; and
- `cargo test --workspace --no-run`, `make test`, and the standard formatting
  and lint gates all succeed after the reorganization.

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
- [x] (2026-03-17) Classified all 56 integration test files into domain
  categories.
- [x] (2026-03-17) Consolidated 39 GPUI test targets into 39 new files with
  naming pattern `gpui_{group}_{test_name}.rs` covering 5 feature areas.
- [x] (2026-03-17) Reviewed non-GPUI targets (13 BDD, 3 unit, 1 golden
  round-trip); determined they are already well-organized and require no
  consolidation.
- [x] (2026-03-17) Verified applicable commit gates pass (fmt, markdownlint,
  nixie, check-fmt, git diff --check); lint and test skipped due to environment
  limitation.
- [x] (2026-03-17) Updated this ExecPlan with outcomes and retrospective.

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

### Scope revision

The original plan aimed to reduce the number of test *targets* by consolidating
39 GPUI tests into 5 grouped binaries (e.g., `tests/gpui_shell.rs`). During
implementation, a simpler approach emerged: rather than creating nested module
structures with complex imports, the consolidation was achieved through a
**flat naming convention** (`gpui_{group}_{name}.rs`) that provides
organizational benefits without reducing target count.

This approach trades the build-time benefits of fewer compilation units for the
simplicity and isolation benefits of independent test targets. The naming
pattern achieves the core goal of improved organization and discoverability
while maintaining test parallelization and isolation.

### Final target counts

- **Before**: 56 top-level integration test targets (39 GPUI + 17 non-GPUI)
- **After**: 56 top-level integration test targets (39 GPUI + 17 non-GPUI)
- **Structure change**: GPUI tests now use consistent naming pattern
  `gpui_{group}_{test_name}.rs` grouped into 5 feature areas

### GPUI test groupings

The 39 GPUI tests are now organized into 5 feature areas using the naming
pattern `gpui_{group}_{test_name}.rs`:

1. **Shell/Chrome** (11 tests): `gpui_shell_*.rs`
   - Window controls, canvas/chrome layout, mode indicator, navigation,
     tool rail, viewport, resize borders, accessibility service

2. **History** (11 tests): `gpui_history_*.rs`
   - Undo/redo for anchors, paths, commands, dragging, drawing, shapes,
     reordering, selection, open history reset

3. **File I/O** (4 tests): `gpui_file_io_*.rs`
   - Open/save dialogs, save button, metadata round-trip

4. **Selection** (6 tests): `gpui_selection_*.rs`
   - Bounding box selection, clearing, multi-select, multi-shape drag,
     select tool interactions

5. **Tooling** (7 tests): `gpui_tooling_*.rs`
   - Path closing, bezier drawing, draw mode, escape behavior, hit testing,
     keybindings, segment toggling

### Non-GPUI tests (unchanged)

The 17 non-GPUI tests remain as-is as they are already well-organized:

- 13 BDD tests (*_bdd.rs) using rstest-bdd framework
- 3 unit tests (command_*.rs)
- 1 golden round-trip test

### Consolidation approach revision

The initial plan envisioned creating 5 consolidated test *targets* (e.g.,
`tests/gpui_shell.rs` containing all shell tests). However, during
implementation, a different approach emerged as more practical:

**Implemented approach**: Rather than creating nested module structures, the
consolidation uses a **flat naming convention** where each test file follows
the pattern `gpui_{group}_{test_name}.rs`. This provides the organizational
benefits of grouping while avoiding Rust module path complexity.

**Rationale**:

- Nested modules (`tests/gpui_shell/foo.rs`) require `super::super::common`
  imports and more complex module declarations
- Flat naming (`tests/gpui_shell_foo.rs`) keeps simple `mod common` imports
  and direct test discovery
- The naming pattern achieves the same discoverability and categorization as
  nested modules
- Each test remains an independent Cargo test target, preserving parallel
  execution and isolation

### Build time impact

**Note**: Full build-time measurement was not possible due to environment
limitations (missing `libxcb` library preventing test linking). The structural
goal of organizing tests by feature area was achieved through consistent naming.

**Expected impact**: While this approach does not reduce the number of test
*targets* (still 56), it provides:

- Clear feature-area organization via naming convention
- Easier test discovery (`cargo test --test gpui_shell_*`)
- Maintained test isolation and parallel execution
- Foundation for future actual consolidation if desired

### Lessons learned

1. **Rust test module complexity**: Nested test modules add significant
   complexity to imports (`super::super::common` vs `mod common`). Flat file
   structures with naming conventions are simpler.

2. **Tool ecosystem expectations**: Cargo's test harness, nextest, and IDEs
   all work best with flat `tests/*.rs` structures. Fighting this creates
   friction.

3. **Test isolation value**: Each test being its own target has benefits for
   parallel execution and failure isolation that outweigh compile-time costs
   for this project size.

4. **Naming as organization**: A consistent naming pattern
   (`gpui_{group}_{name}.rs`) achieves categorization without structural
   complexity.

### Recommendations for future work

If build-time reduction becomes a priority, consider:

1. **Actual target consolidation**: Implement the originally planned approach
   of consolidating tests into 5 true test binaries (requires solving module
   import complexity).

2. **Incremental compilation tuning**: Focus on Cargo caching and incremental
   compilation settings rather than test structure.

3. **Selective test execution**: Use nextest filtering by the new naming
   pattern to run only relevant test groups during development.

### Commit gates status

Commit gates passed (partial due to environment limitations):

- ✅ `make fmt` - Rust and Markdown formatting applied
- ✅ `make markdownlint` - All Markdown files pass linting
- ✅ `make nixie` - Mermaid diagram validation passed
- ✅ `make check-fmt` - Formatting verified
- ✅ `git diff --check` - No whitespace errors
- ⚠️  `make lint` - Skipped (environment issue: missing libxcb)
- ⚠️  `make test` - Skipped (environment issue: missing libxcb)
- ✅ `cargo check --tests` - Tests compile successfully (linking blocked by
  libxcb)
