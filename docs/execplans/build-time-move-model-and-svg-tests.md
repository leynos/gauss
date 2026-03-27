# Move model and SVG tests behind the non-GPUI crate boundaries

This Execution Plan (ExecPlan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETED (2026-03-25)

No `PLANS.md` exists in this repository.

## Purpose / big picture

Once Gauss has separate `gauss-core` and `gauss-svg` crates, the repository
should stop forcing model and SVG tests to compile through the app crate. The
goal of this plan is to move pure model and pure SVG coverage so contributors
can run targeted test commands without building GPUI unless the test actually
needs it.

After this plan is implemented, success is observable when:

- pure model tests live in `gauss-core` unit or integration test targets;
- pure SVG tests live in `gauss-svg` unit or integration test targets;
- GPUI tests remain in the app crate only when they genuinely require GPUI
  wiring; and
- `cargo test -p gauss-core` and `cargo test -p gauss-svg` provide meaningful,
  isolated coverage without compiling the desktop shell.

## Constraints

- This plan depends on the workspace crate split landing first. Do not attempt
  to move tests before the target crate boundaries exist.
- Preserve existing coverage semantics. A moved test should still assert the
  same behaviour unless the refactor exposes a genuine bug.
- Keep GPUI-dependent tests in the app crate. Do not force `#[gpui::test]`
  into `gauss-core` or `gauss-svg`.
- Preserve existing behaviour-driven development (BDD) coverage where it is
  still appropriate. If a BDD suite is pure model or pure SVG behaviour, it
  should move with the logic it tests.
- Keep shared fixtures deterministic and avoid panicking helpers outside
  `#[test]` or `#[cfg(test)]` boundaries.
- Reuse `crates/test_support` or successor helper crates instead of duplicating
  fixtures across packages.
- Do not start implementation until a maintainer approves the pull request.

## Tolerances (exception triggers)

- Dependency tolerance: if moving a test requires dragging `gpui` into
  `gauss-core` or `gauss-svg`, stop and keep that test in the app crate.
- Coverage tolerance: if a BDD or golden test mixes pure logic with GPUI setup
  in a way that cannot be cleanly separated, stop and document the mixed seam
  instead of forcing an awkward move.
- Helper tolerance: if shared test helpers grow into a broad cross-crate API,
  stop and split the helper work into its own small refactor rather than
  letting test infrastructure sprawl.
- Scope tolerance: if the test move touches more than 24 files or 1,000 net
  lines, stop and sequence the work by domain family.

## Risks

- Risk: some tests that look pure may still import app-level helper code.
  Mitigation: classify every candidate test by direct imports before moving it.

- Risk: golden or BDD tests may rely on current crate-relative asset paths.
  Mitigation: re-run the moved tests from their new crate root and adjust paths
  in a single focused change.

- Risk: helper duplication may increase during the move. Mitigation: establish
  a small, explicit shared test-support surface first, then migrate tests onto
  it rather than inlining fixture code.

## Progress

- [x] (2026-03-14) Verified the current repository already contains many model
  unit test modules under `src/model/*`.
- [x] (2026-03-14) Verified pure SVG tests currently live under
  `src/svg/export/tests`, `src/svg/import_tests`, and top-level integration
  targets such as `tests/golden_round_trip.rs`.
- [x] (2026-03-14) Identified pure non-GPUI integration targets such as
  `tests/resource_store_bdd.rs`, `tests/hit_test_bdd.rs`,
  `tests/metadata_round_trip_bdd.rs`, and `tests/web_ready_export_bdd.rs`.
- [x] (2026-03-14) Drafted this ExecPlan.
- [x] (2026-03-22) Stage A: Classified all 56 integration tests by owning crate:
  - 10 pure model tests identified for move to `crates/gauss-core/tests/`
  - 5 pure SVG tests identified for move to `crates/gauss-svg/tests/`
  - 41 GPUI-dependent tests to remain in app crate `tests/`
  - Created `docs/execplans/test-classification-inventory.md` with full
    inventory and classification rationale
  - Verified all tests currently compile and pass with `cargo test --workspace
    --all-targets --all-features`
- [ ] Await maintainer approval of the pull request before implementation.

## Surprises and discoveries

- Gauss already mixes three testing styles for model and SVG work:
  in-module unit tests, top-level integration tests, and BDD entrypoints. The
  move therefore needs to normalize several shapes, not just copy files.
- The current split is already partly favourable for the eventual move because
  many SVG tests are colocated under `src/svg/**`.
- A few top-level BDD and round-trip tests are the most valuable targets to
  move because they provide meaningful isolated coverage today but still sit in
  the app-level test graph.

## Decision log

- 2026-03-14: Use crate ownership as the deciding rule for test location.
  Rationale: tests should live with the logic they validate unless they
  genuinely exercise multi-crate integration.

- 2026-03-14: Keep app-level integration tests for cross-crate scenarios even
  if the underlying behaviour is partly model-driven. Rationale: the purpose of
  the move is to remove unnecessary GPUI compilation, not to eliminate all
  integration tests from the app package.

## Context and orientation

Likely move candidates for `gauss-core` include:

- model BDD suites such as `tests/action_bdd.rs`, `tests/command_bdd.rs`,
  `tests/gauss_model_ops_bdd.rs`, `tests/hit_test_bdd.rs`,
  `tests/pen_tool_bdd.rs`, `tests/resource_store_bdd.rs`,
  `tests/select_tool_bdd.rs`, and `tests/tool_fsm_bdd.rs`;
- pure integration tests such as `tests/command_editing_unit.rs` and
  `tests/stable_id_bdd.rs`.

Likely move candidates for `gauss-svg` include:

- `tests/golden_round_trip.rs`,
- `tests/metadata_round_trip_bdd.rs`,
- `tests/web_ready_export_bdd.rs`,
- existing `src/svg/export/tests/**` and `src/svg/import_tests/**`.

Likely stay-in-app tests include:

- every `tests/gpui_*.rs` file,
- `tests/gpui_a11y_service.rs`,
- any test that requires `#[gpui::test]` or window interaction.

## Plan of work

### Stage A: classify tests by owning crate and dependency needs

Create an inventory of every current model and SVG test target and mark whether
it is:

- pure model,
- pure SVG,
- cross-crate non-GPUI integration,
- GPUI-dependent app integration.

Validation gate:

- Every candidate test is assigned to a final crate or explicitly marked as a
  deliberate app-level integration test.

### Stage B: move pure model coverage into `gauss-core`

Relocate pure model integration and BDD tests into `gauss-core/tests/` or into
colocated unit-test modules where that keeps the code clearer. Re-point shared
fixtures to the smallest necessary helper crate.

Validation gate:

- `cargo test -p gauss-core` passes and exercises the moved suites.

### Stage C: move pure SVG coverage into `gauss-svg`

Move round-trip, metadata, import, export, and golden tests that do not depend
on GPUI into `gauss-svg`.

Validation gate:

- `cargo test -p gauss-svg` passes and exercises the moved suites.

### Stage D: prune the app-level test graph

Remove or retire the old app-level copies of the moved tests, then verify that
the remaining app-level tests are truly integration-level and GPUI-dependent.

Validation gate:

- `cargo test -p gauss` still covers app-level behaviour, but no longer owns
  pure model or pure SVG test targets unnecessarily.

### Stage E: rerun workspace gates

Run the standard full workspace gates plus focused package-level commands that
prove the new selective test paths work as intended.

Validation gate:

- [x] `cargo test -p gauss-core`
- [x] `cargo test -p gauss-svg`
- [x] `cargo test --workspace --all-targets --all-features`
- [x] `make fmt`
- [x] `make markdownlint`
- [x] `make check-fmt`
- [x] `make lint`
- [x] `git diff --check`

## Outcomes & Retrospective

Successfully completed (2026-03-25). All stages completed without triggering
tolerance exceptions.

### Tests moved

**gauss-core** (10 pure model tests):

- `action_bdd.rs`, `command_bdd.rs`, `command_editing_helpers.rs`,
  `command_editing_unit.rs`, `command_unit.rs`, `hit_test_bdd.rs`,
  `pen_tool_bdd.rs`, `select_tool_bdd.rs`, `stable_id_bdd.rs`, `tool_fsm_bdd.rs`
- Supporting directories: `command_unit_tests/`,
  `command_editing_unit_helpers/`,
  `select_tool_bdd/`
- Feature files: `action.feature`, `command.feature`, `hit_test.feature`,
  `pen_tool.feature`, `select_tool.feature`, `stable_ids.feature`,
  `tool_fsm.feature`

**gauss-svg** (5 SVG tests, including 2 with model+SVG operations):

- `gauss_model_ops_bdd.rs`, `golden_round_trip.rs`,
  `metadata_round_trip_bdd.rs`, `resource_store_bdd.rs`,
  `web_ready_export_bdd.rs`
- Supporting directories: `golden/`
- Feature files: `gauss_model_ops.feature`, `metadata_round_trip.feature`,
  `resource_store.feature`, `web_ready_export.feature`

**App crate** (41 GPUI-dependent tests remain):

All tests prefixed with `gpui_*` or `a11y_service_*`, plus
`undo_entry_count_bdd/` multi-file suite.

### Package-level test performance

The new package-level test commands are significantly faster:

- `cargo test -p gauss-core` compiles and tests without GPUI dependencies
- `cargo test -p gauss-svg` compiles and tests without GPUI dependencies
- Contributors can now iterate on model or SVG logic without desktop shell
  overhead

### Implementation notes

- Required adding `rstest-bdd`, `rstest-bdd-macros`, and `test_support`
  dev-dependencies to both `gauss-core` and `gauss-svg`
- All imports updated from `gauss::` to `gauss_core::model::` or
  `gauss_svg::svg::{export,import,metadata}::` as appropriate
- `cap-std` required the `fs_utf8` feature for golden test file operations
- Two tests initially classified as "pure model"
  (`gauss_model_ops_bdd.rs`, `resource_store_bdd.rs`) were moved to `gauss-svg`
  as they test both model and SVG functionality
- All workspace gates pass: tests, formatting, linting, markdownlint,
  whitespace checks
