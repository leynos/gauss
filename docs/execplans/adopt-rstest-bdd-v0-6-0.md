# Adopt `rstest-bdd` v0.6.0-beta1

## Big picture

Upgrade the workspace from `rstest-bdd` and `rstest-bdd-macros` `0.5.0` to
`0.6.0-beta1`, then introduce the first-party
`rstest-bdd-harness-gpui::GpuiHarness` as the canonical pattern for behavioural
tests that need a `gpui::TestAppContext`. As a pilot, migrate one existing
`#[gpui::test]` integration test (`gpui_shell_mode_indicator.rs`) into a
feature-driven BDD test and capture the resulting pattern in a new
`docs/developers-guide.md`. The remaining 38 GPUI tests stay on `#[gpui::test]`
for now; this plan deliberately avoids a workspace-wide rewrite. The pilot's
job is to prove the harness wiring, anchor the documented pattern, and give
later tests a copyable template.

This work is purely about test-infrastructure adoption. It must not change any
production behaviour, must keep all existing tests green, and must leave the
codebase in a state where future GPUI BDD migrations are cheap. Adopting the
beta release is acceptable because the `0.6.0` API surface is documented and
stable, and the migration guide enumerates every breaking change relevant to
this repository.

## Why now

- `0.5.0` does not expose a GPUI harness; behavioural coverage that needs a
  `TestAppContext` currently has to be written as a hand-rolled `#[gpui::test]`
  with an inline assertion narrative, duplicating the Gherkin-style structure
  the team prefers for behavioural tests.
- The recently merged i18n BDD work (commits `2217ded`, `68b4c94`) increased
  the share of UI-adjacent assertions running through `Phase0Shell`. That
  surface is the natural target for harness-backed BDD tests.
- The workspace already uses `rstest-bdd 0.5.0` for non-GPUI BDD tests, so the
  adoption cost is largely confined to dependency updates, the underscore
  fixture rule, and one new pilot test.

## Constraints

- Keep production code unchanged. Changes are confined to `Cargo.toml` files,
  `tests/`, and `docs/`.
- Maintain the existing `make check-fmt`, `make lint`, and `make test` gates.
  Every commit must pass them.
- Use British English (en-GB-oxendict) in documentation prose, with the
  exception of API names supplied by upstream crates.
- No code file may exceed 400 lines. New BDD test modules split steps into
  files under `tests/<test_name>_bdd/` mirroring the existing pattern in
  `tests/widget_capability_audit_bdd/` and `tests/undo_entry_count_bdd/`.
- Honour the workspace lint policy. Pay particular attention to
  `unwrap_used`, `expect_used`, and `missing_docs`; step modules need `//!`
  headers and tests should propagate failures via `TestSupportResult` rather
  than panicking inside fixture setup.
- Do not depend on `rstest-bdd-harness-tokio` in this plan. Tokio harness
  adoption is a separate decision and would entangle the runtime story with the
  harness story.

## Inputs reviewed

- `docs/rstest-bdd-users-guide.md`, sections "Using the GPUI harness"
  (lines 950–1027) and "Harness adapter core APIs" (lines 1514–1596).
- `docs/rstest-bdd-v0-6-0-migration-guide.md` in full, with particular
  attention to the breaking-change list and the migration checklist.
- `tests/CONSOLIDATION_MAP.md` for the GPUI test inventory.
- `tests/gpui_shell_mode_indicator.rs`, `tests/gpui_shell_quit_button.rs`,
  `tests/gpui_shell_canvas_layout.rs`, `tests/gpui_i18n_module.rs`,
  `tests/widget_capability_audit_bdd/main.rs`, and `tests/common/mod.rs` for
  the existing test patterns.
- Workspace `Cargo.toml`, `crates/test_support/Cargo.toml`,
  `rust-toolchain.toml` (channel `1.92.0`), and the existing pattern of
  `tests/<feature>_bdd/` modules already used in the repository.

## Pilot migration target

The pilot is `tests/gpui_shell_mode_indicator.rs`. Reasons:

- It exercises three observable transitions of the shell mode indicator
  (initial draw mode, Tab toggling the edge mode, switching to manipulate
  mode), which map cleanly onto `Given`/`When`/`Then` and a single feature file
  with three scenarios sharing a common `Background`.
- The test already calls `Phase0Shell::mode_status_line_for_tests`,
  `simulate_keystrokes`, and `enter_manipulate_mode_for_tests`, all of which
  are already exposed for the i18n module tests. No new test seams are required
  in production code.
- Behaviour is fully observable through a single string return value, so
  step assertions stay concise and avoid leaking GPUI types into step
  signatures beyond `&mut TestAppContext`.
- File size is small (42 lines), so the migration is a tight loop and the
  resulting BDD module stays well below the 400-line limit even after adding a
  feature file and steps.

A secondary candidate considered and **deferred**: `gpui_shell_quit_button.rs`.
It only has a single observable outcome (`did_request_quit`), which would yield
a one-scenario feature file. That offers little template value over the mode
indicator. The developer's guide names it as the obvious next candidate once
the pattern is in place.

## Workstreams

### A. Dependency and workspace setup

1. Add `rstest-bdd`, `rstest-bdd-macros`, and `rstest-bdd-harness-gpui` to
   `[workspace.dependencies]` in the root `Cargo.toml`. Move the existing
   pinned `rstest-bdd*` versions into the workspace table so member crates
   inherit them with `.workspace = true`. Set version requirements to
   `"0.6.0-beta1"`. Match `rstest` to whatever the new releases require
   (currently `0.26.x` already in the workspace).
2. Update `[dev-dependencies]` in the root `Cargo.toml` (and any member crate
   that uses `rstest-bdd*`) to inherit from the workspace table.
3. Confirm `gauss-core/Cargo.toml`, `gauss-svg/Cargo.toml`, and
   `test_support/Cargo.toml` do not pull in their own pinned `rstest-bdd*`
   versions. If they do, switch them to `.workspace = true`.
4. Run `cargo update -p rstest-bdd -p rstest-bdd-macros` (or a fresh resolve)
   and verify `Cargo.lock` updates only the targeted crates. Commit the
   manifest and lockfile change separately from the test migration so the two
   concerns review cleanly.
5. Run `make check-fmt`, `make lint`, and `make test` after the dependency
   bump. Address any new lint or compile errors driven solely by the bump
   before moving on.

### B. Sweep `0.6.0` breaking changes

1. **Implicit fixture-name normalization.** Search the test tree for
   `_world`, `__world`, or any other underscore-prefixed parameter that
   currently relies on a literal underscore in a fixture key. Use
   `rg -n '\b_+\w+\s*:' tests crates` and inspect matches. Where the underscore
   was only there to silence the unused-binding lint, no source change is
   needed because the new normalization rule still resolves to the same key.
   Where the literal underscore *was* the key, add an explicit `#[from(_name)]`
   attribute. Existing `gauss` BDD tests (`a11y_service_bdd*`,
   `widget_capability_audit_bdd`, `undo_entry_count_bdd`, `i18n_bdd`) bind
   `world: &mut Foo` without a leading underscore, so this is expected to be a
   low-friction sweep.
2. **`scenarios!(..., runtime = "tokio-current-thread")`.** Confirm the
   string `runtime = "tokio-current-thread"` does not appear anywhere in the
   gauss workspace. The legacy syntax is a deprecation candidate but not in
   active use here, so no change is expected.
3. **Custom `HarnessAdapter` implementations.** Confirm there are no
   bespoke `HarnessAdapter` implementations in the workspace. Adoption relies
   on first-party `GpuiHarness`, so no `HarnessResult<T>` migration is needed
   in tree.
4. **Result-returning fixtures.** No change required for adoption; flag in
   the developer's guide as an available pattern but do not introduce one
   speculatively.
5. Run the full gate after this sweep before starting the pilot migration.
   Commit any in-tree fix-ups separately from the pilot.

### C. Pilot migration: `gpui_shell_mode_indicator`

1. **Feature file.** Add `tests/features/shell_mode_indicator.feature` with
   three scenarios sharing a `Background` that opens the Phase 0 shell:
   - "Initial draw mode" → `Then the mode indicator reads "Mode: Draw (Line)"`.
   - "Pressing Tab toggles the draw edge mode" →
     `When I press the "tab" key`, then a Bezier-mode assertion.
   - "Switching to manipulate mode hides the edge suffix" →
     `When I enter manipulate mode`, then the manipulate assertion.
   Use British English in narrative lines, but quote the literal mode strings
   verbatim because they are produced by the production code.
2. **BDD module layout.** Create the module pair
   `tests/gpui_shell_mode_indicator_bdd.rs` (entry file with `#[scenario]`
   bindings and re-exports) and `tests/gpui_shell_mode_indicator_bdd/`
   directory containing `steps.rs` and a small `world.rs`. This mirrors the
   existing `widget_capability_audit_bdd` and `undo_entry_count_bdd` layouts
   and keeps each file under the 400-line limit.
3. **Harness wiring.** The scenario binding uses
   `#[scenario(path = "tests/features/shell_mode_indicator.feature", harness = rstest_bdd_harness_gpui::GpuiHarness)]`.
    Steps request the harness-injected context with
   `#[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext` and
   delegate to a small `ShellWorld` newtype that owns the `Phase0Shell` entity,
   the `VisualTestContext`, and any cached strings. The world is constructed in
   a Given step that wraps the existing `init_test_app` + `add_window_view` +
   `ensure_initial_draw` helpers from `tests/common/mod.rs`. This keeps the
   migration *additive* over the shared helpers rather than reimplementing them.
4. **Removal.** Once the BDD test runs green and asserts every original
   transition, delete `tests/gpui_shell_mode_indicator.rs`. The feature
   coverage is preserved by the BDD test and by the i18n module tests that
   already cover `mode_status_line_for_tests` from a different angle.
5. **Native-library gating.** The migration guide warns that `GpuiHarness`
   in non-shim environments may pull in X11 libraries. The gauss workspace does
   not vendor a GPUI shim and depends on the published `gpui` crate with
   `test-support`, so check whether `cargo test` on the existing Linux runner
   already links those libraries (it does, since the current `#[gpui::test]`
   tests already run there). If a new native dependency surfaces, document it
   in the developer's guide; do not silently add apt-package requirements.
6. **Run the gate.** `make check-fmt`, `make lint`, `make test`, in that
   order, with output captured via `tee` per repo guidance. Commit the pilot as
   a single atomic change once green.

### D. Documentation: new `docs/developers-guide.md`

Create a top-level developer's guide focused on testing patterns and the new
BDD harness adoption. The widget-audit-developer-guide.md remains as its own
narrowly scoped document. Sections:

1. **Purpose and audience.** One paragraph: this guide is for contributors
   adding tests or extending behavioural coverage in gauss. Pointers to
   `docs/rstest-bdd-users-guide.md` and
   `docs/rstest-bdd-v0-6-0-migration-guide.md` as upstream sources of truth.
2. **Test taxonomy.** Brief enumeration distinguishing
   `#[gpui::test]` integration tests (kept as the default for low-level GPUI
   integration coverage that does not benefit from Gherkin), pure
   `rstest`-based unit tests, and `rstest-bdd` behavioural tests.
   Cross-reference `tests/CONSOLIDATION_MAP.md`.
3. **When to choose the GPUI BDD harness.** A short decision list:
   choose the harness when (a) the behaviour can be expressed in three or more
   observable transitions sharing setup, (b) the assertions are
   narrative-friendly (status strings, accessibility output, mode flags), or
   (c) the steps will be reused across multiple scenarios. Otherwise prefer a
   single `#[gpui::test]`.
4. **The pattern, end to end.** Walk through the pilot:
   - Feature file location and the `tests/features/` convention.
   - BDD module split (`tests/<name>_bdd.rs` plus
     `tests/<name>_bdd/{steps.rs,world.rs}`).
   - Harness selection: `harness = rstest_bdd_harness_gpui::GpuiHarness`.
   - Accessing the injected `TestAppContext` via
     `#[from(rstest_bdd_harness_context)]`.
   - Reusing helpers from `tests/common/mod.rs` (preferred) instead of
     duplicating shell-construction code in the world module.
   - Returning `TestSupportResult<()>` from steps that can fail.
5. **Migrating an existing `#[gpui::test]` test.** Five-step recipe
   distilled from workstream C, with the next obvious candidate
   (`gpui_shell_quit_button`) named explicitly so a future contributor has a
   low-stakes place to apply the pattern.
6. **Breaking changes to remember.** A one-page summary of the v0.6.0
   migration guide focused on what gauss contributors must know: underscore
   fixture normalization, `Default` requirement on harness types selected by
   macros, and the `Result`/`StepResult` fixture rules.
7. **Running the gate.** Recap of the `make check-fmt`/`make lint`/`make
   test` triad with the `tee` log file convention from `AGENTS.md`.

The new guide stays under 300 lines, uses dashes for bullets, wraps prose at 80
columns and code blocks at 120, and runs through `make markdownlint` and
`make fmt` before commit.

### E. Cross-document updates

1. Add a one-line reference to the new developer's guide from
   `docs/users-guide.md` (or wherever a "Contributing/testing" pointer
   currently lives) so it is discoverable.
2. Update `docs/roadmap.md` only if it currently calls out the rstest-bdd
   version. Do not introduce new roadmap items in this plan.
3. Mention the harness adoption in the next relevant entry of
   `CHANGELOG.md` under an "Internal" or "Testing" heading; keep it factual and
   short.

## Verification

For every commit:

- `make check-fmt`
- `make lint`
- `make test`

Capture each invocation as `/tmp/$ACTION-gauss-adopt-rstest-bdd-v0-6-0.out` per
`AGENTS.md`. After the pilot lands, additionally:

- `cargo test --test gpui_shell_mode_indicator_bdd -- --nocapture` to confirm
  the harness emits the expected steps in order.
- `make markdownlint` and `make nixie` after touching `docs/`.

## Out of scope

- Migrating any `#[gpui::test]` test other than `gpui_shell_mode_indicator`.
- Adopting `rstest-bdd-harness-tokio` or any non-GPUI harness.
- Restructuring `tests/common/mod.rs` beyond minimal additions required by
  the pilot's `world.rs`.
- Introducing new behavioural coverage. Scenarios in the pilot must mirror
  the original `#[gpui::test]` assertions one-for-one.
- Production-code refactors. New `for_tests` seams are out of scope; if a
  scenario cannot be expressed with the existing seams, the migration is
  abandoned and recorded as a follow-up.

## Risks and open questions

- **Beta release.** `0.6.0-beta1` may receive further breaking churn before
  the final tag. Mitigate by pinning to the exact beta version and scheduling a
  follow-up to move to `0.6.0` once published. If the beta is withdrawn before
  this work lands, pause adoption and re-plan.
- **Native libraries.** Confirm during workstream C that the existing
  Linux test environment already provides any X11/xkbcommon libraries the GPUI
  harness needs. If not, document the apt-package requirement in the
  developer's guide and the runner setup; do not paper over a missing
  dependency.
- **`Default` requirement.** `GpuiHarness` is selected by path, so the
  macro instantiates it with `Default`. Confirm the upstream
  `rstest-bdd-harness-gpui` crate derives `Default` on `GpuiHarness` (the
  migration guide implies it does); if it does not, the pilot will fail to
  compile and the failure must be reported upstream rather than worked around
  with a wrapper type.
- **Lint friction.** The workspace's strict pedantic profile may flag
  generated code. Track any false positives in the pilot commit message rather
  than silencing them globally.

## Step-by-step execution checklist

- [x] Workstream A: dependency bump committed and gated.
- [x] Workstream B: underscore-fixture sweep complete; no behavioural
      changes; gate passes.
- [x] Workstream C: feature file added, BDD module created,
      `gpui_shell_mode_indicator.rs` removed, gate passes, `cargo test
      --test gpui_shell_mode_indicator_bdd` passes locally.
- [x] Workstream D: `docs/developers-guide.md` written; markdown lint and
      mermaid checks pass.
- [x] Workstream E: cross-references added; CHANGELOG updated.
- [x] Final pass: `make check-fmt && make lint && make test` clean on a
      fresh checkout of the branch.

## Implementation notes

### FixtureRefMut borrow conflict (discovered during workstream C)

`StepContext::borrow_mut` takes `&mut self` and returns a `FixtureRefMut` that
borrows `self` for its lifetime. A step that requests two mutable fixtures
(e.g. `&mut TestAppContext` from the harness and `&mut ShellWorld` from a world
fixture) triggers E0499 because the first `FixtureRefMut`'s borrow on
`StepContext` prevents the second `borrow_mut` call.

**Resolution:** Store the world in a thread-local `RefCell` rather than as a
`StepContext` fixture. Each step then borrows only one mutable fixture from
`StepContext` — the harness context. Thread-local storage is safe because each
GPUI integration test runs on a single thread.

**Reset protocol:** The `WORLD` thread-local persists for the lifetime of the
thread, so scenarios can inherit stale `Option` fields after an early exit.
Implement `pub(crate) fn reset_world()` in
`tests/gpui_shell_mode_indicator_bdd/world.rs` to write `ShellWorld::default()`
back into that `RefCell`, and invoke `world::reset_world()` as the first
statement in every `#[given]` step in
`tests/gpui_shell_mode_indicator_bdd/steps.rs`, before assigning new handles.
The reset targets the same `RefCell` the steps read through
`world::with_world`; it does not replace the harness context that `StepContext`
still borrows exclusively as `&mut TestAppContext`.

This pattern is documented in the `Thread-local world (and why)` section of
`docs/developers-guide.md`. The `scenario!` macro (auto-discovery) is not yet
validated with this pattern; individual `#[scenario]` bindings are used for now.
