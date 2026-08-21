# Adopt rstest-bdd v0.6.0-beta3 and migrate a GPUI behavioural test to the GPUI harness

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

After this change the `gauss` workspace builds and tests against `rstest-bdd`
`0.6.0-beta3` instead of `0.5.0`, and one non-trivial GPUI behavioural test —
the draw/undo/redo flow currently written as a raw `#[gpui::test]` — is
expressed as a Gherkin scenario driven through the first-party
`rstest_bdd_harness_gpui::GpuiHarness`.

Its operation is observable in two ways:

1. `make test` (the project's full gate: format check, clippy, and the test
   suite) passes with every existing `*_bdd.rs` suite still green under the new
   dependency version.
2. A new integration test binary `tests/gpui_draw_undo_bdd.rs` runs a
   Gherkin scenario (`tests/features/draw_undo.feature`) end to end. Running
   `cargo test --test gpui_draw_undo_bdd` executes the GPUI harness, opens a
   real `TestAppContext` window, drives draw clicks and undo/redo through
   registered steps, and passes. Deleting a `Then` assertion or the
   `Drop`-based reset makes it fail, proving the steps and the two-sided reset
   protocol are load-bearing.

The wider goal is to act as a beta proof-point for `rstest-bdd 0.6.0-beta3`'s
GPUI harness against a downstream project that consumes the **published**
`gpui 0.2.2` crate (not the vendored fork the upstream regression suite uses),
and to record friction in a beta tester's log at
`~/docs/rstest-bdd-v0-6-0-beta3-gauss-testers-log.md`.

## Constraints

Hard invariants that must hold throughout implementation. Violation requires
escalation, not a workaround.

- The workspace targets Rust 1.85+ / edition 2024 and must continue to. Do not
  raise the toolchain floor to satisfy the new dependency.
- The project consumes **published `gpui = "0.2.2"`** from crates.io. Do not
  vendor gpui or switch to the upstream `rstest-bdd` vendored fork. All GPUI
  harness step code must use the *published* `gpui 0.2.2` API shapes (see
  "Published vs vendored gpui" under Context and orientation), not the vendored
  shapes shown verbatim in the upstream user's guide.
- The workspace clippy posture in the root `Cargo.toml` (`pedantic` plus the
  denied restriction lints such as `unwrap_used`, `expect_used`,
  `float_arithmetic`) must remain unchanged. New test code either satisfies
  these lints or carries a file-scoped `#![expect(...)]` with a `reason`,
  matching how the existing `tests/common/mod.rs` and `tests/gpui_*.rs` files
  handle the same lints.
- Do not weaken, delete, or `#[ignore]` any existing test to make the upgrade
  pass. Existing behavioural coverage must remain intact except for the single
  `#[gpui::test]` function being intentionally replaced.
- Only the `draw_click_adds_points_and_undo_removes` function in
  `tests/gpui_history_draw_undo.rs` is replaced. The other two tests in that
  file (`activate_pen_tool_from_manipulate_allows_drawing`,
  `stale_active_path_is_recovered_when_pen_draws_new_shape`) must remain and
  keep passing.

## Tolerances (exception triggers)

Thresholds that trigger escalation (stop and record in the Decision Log) when
breached.

- Dependency compatibility: if `rstest-bdd-harness-gpui = "0.6.0-beta3"`
  cannot resolve against `gpui 0.2.2` (for example it pins an incompatible
  `gpui` version, producing two `gpui` crates in the tree so that
  `TestAppContext` types do not unify), stop at the Stage B spike and escalate.
  This is the single most likely blocker.
- Publication: if any of `rstest-bdd`, `rstest-bdd-macros`,
  `rstest-bdd-harness`, or `rstest-bdd-harness-gpui` is not available at
  `0.6.0-beta3` on crates.io, stop and escalate.
- New dependencies: adopting the harness adds `rstest-bdd-harness-gpui` and
  `serial_test` as dev-dependencies. These two are anticipated and approved by
  this plan. Any *further* new dependency beyond those two requires escalation.
- Breaking-change surface: if the underscore-normalization breaking change (see
  Risks) turns out to affect more than a handful of existing step/scenario
  signatures, or requires changing production code (not just test signatures),
  stop and escalate before mass-editing.
- Scope: if adopting the version bump requires changing production source files
  under `src/` or `crates/*/src/` (as opposed to `Cargo.toml` files and test
  code), stop and escalate.
- Iterations: if the migrated scenario still fails after 5 focused
  fix-and-rerun attempts, stop and escalate with the transcript.

## Risks

- Risk: `rstest-bdd-harness-gpui 0.6.0-beta3` depends on a `gpui` version that
  does not unify with the workspace's published `gpui 0.2.2`, so a step's
  `#[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext` refers to
  a different `TestAppContext` type than the one `gauss` uses. Severity: high.
  Likelihood: medium. Mitigation: verify first, in a minimal spike (Stage B),
  before touching the real test. This is the go/no-go gate. If it fails, the
  fallback is to land the core version bump only and defer the GPUI migration,
  recording the incompatibility in the tester's log and escalating.

- Risk: published `gpui 0.2.2` diverges from the vendored fork the upstream
  playbook is written against (closure arity of `add_window_view`,
  `VisualTestContext::from_window` returning a value rather than `Option`,
  `read_entity`/`update_entity` return shapes, and `window_handle()` living on
  the `VisualContext` trait). Copying the guide snippets verbatim will not
  compile. Severity: medium. Likelihood: high (certain if snippets are copied
  blindly). Mitigation: the plan specifies the published-crate shapes
  explicitly and the existing `tests/common/mod.rs` already demonstrates them
  (`add_window_view(|_window, view_cx| ...)`).

- Risk: the underscore-normalization breaking change silently rebinds a
  `_name` step/scenario parameter to a different fixture key. Severity: medium.
  Likelihood: low (an initial grep found no leading-underscore step/scenario
  parameters in `gauss`). Mitigation: run the audit grep in Stage A and add
  explicit `#[from(_name)]` wherever a literal underscore key was intended.

- Risk: editing only a `.feature` file does not trigger a Cargo rebuild
  (upstream caveat until roadmap item 11.3.1 lands), producing misleading
  "passing" runs. Severity: low. Likelihood: medium. Mitigation: always edit the
  `.rs` step file alongside the feature, or run `cargo clean -p gauss` before
  re-running when only the feature changed.

- Risk: workspace restriction lints (`expect_used`, `unwrap_used`,
  `float_arithmetic`) reject idiomatic test code. Severity: low. Likelihood:
  medium. Mitigation: mirror the file-scoped `#![expect(...)]` pattern already
  used in `tests/common/mod.rs`.

## Progress

- [x] (2026-07-08) Stage A: audit and dependency inventory (no functional
  change). Findings: three manifests pin `rstest-bdd = "0.5.0"` (root,
  `gauss-core`, `gauss-svg`); no `tokio-current-thread` alias; no custom
  `HarnessAdapter`; no leading-underscore fixture params and no `#[from(_...)]`
  (only a `_p3` tuple-destructure in `gpui_tooling_close_path.rs`, not a
  fixture). `0.6.0-beta3` confirmed published for `rstest-bdd`,
  `rstest-bdd-macros`, and `rstest-bdd-harness-gpui`. No production-source or
  underscore edits required.
- [x] (2026-07-08) Stage B (prototyping/spike): **GO**.
  `cargo tree -i gpui` shows a single `gpui v0.2.2` shared by `gauss`,
  `gpui-component`, and `rstest-bdd-harness-gpui v0.6.0-beta3`. The throwaway
  `cargo test --test harness_gpui_spike` passed (`1 passed`), proving the
  harness injects a `gpui::TestAppContext` that unifies with the workspace's
  published `gpui 0.2.2`. Spike files deleted; dependency edits retained.
- [x] (2026-07-08) Stage C: bumped `rstest-bdd`/`rstest-bdd-macros` to
  `0.6.0-beta3` in all three manifests. `cargo test -p gauss-core -p gauss-svg`
  green (0 failed). Full-workspace `make all` gate pending before the
  milestone-1 CodeRabbit review.
- [x] (2026-07-08) Milestone 1 gate: `make all` green (909 tests passed, fmt +
  clippy clean). CodeRabbit `review --agent`: **0 findings**.
- [x] (2026-07-08) Stage D/E: added `tests/features/draw_undo.feature` and
  implemented `tests/gpui_draw_undo_bdd.rs` (steps + thread-local state).
  `cargo test --test gpui_draw_undo_bdd --features test-support` → `1 passed`.
  Falsification: temporarily changing the anchor-count assertion to `count + 1`
  produced a red at step index 2 (`Then the draw shape anchor count is 1`) with
  the harness prepending feature path/line/scenario name to the panic; reverted
  to green.
- [x] (2026-07-08) Stage F (refactor/cleanup): removed
  `draw_click_adds_points_and_undo_removes` and its nine now-dead file-local
  helpers from `tests/gpui_history_draw_undo.rs`, trimmed the imports, and kept
  the two remaining `#[gpui::test]` functions. Full-workspace `make all` gate
  pending.
- [x] (2026-07-08) Milestone 2 gate: `make all` green (909 passed).
  CodeRabbit `review --agent`: **0 findings**.
- [x] (2026-07-08) Stage G: updated the beta tester's log
  (`~/docs/rstest-bdd-v0-6-0-beta3-gauss-testers-log.md`) and this plan's
  retrospective.
- [x] Stage H (post-review follow-up, on user request): refactor steps/helpers
  to be fallible (no panics; `Result` + `?`), and validate the "`#[then]`
  should be an actual test" concern. Done: steps now return spelled-out
  `Result<(), TestSupportError>`; validation uncovered the alias-swallow
  false-green and the fallible-scenario `unused_must_use` defect (Surprises &
  Decision Log); concern flagged in the tester's log. `make all` green (909
  passed); CodeRabbit `review --agent`: **0 findings**. Plan COMPLETE.

## Surprises & discoveries

- Observation: all 76 `add_window_view` call sites in `tests/` use the
  two-argument closure `|_window, view_cx| ...`. Evidence:
  `grep -rhoE "add_window_view\(\|[^)]*\|" tests` → 76 ×
  `add_window_view(|_window, view_cx|`. Impact: confirms `gauss` is on the
  published `gpui 0.2.2` API surface, so the vendored-fork snippets in the
  upstream guide must be adapted, not copied.

- Observation: a step whose return type is a `Result` **type alias** silently
  swallows `Err` — the `#[then]` passes green even when its assertion fails.
  Evidence: with steps declared `-> TestSupportResult<()>` (an alias for
  `Result<(), TestSupportError>`) and the anchor-count assertion deliberately
  broken, `cargo test` (after recompiling `gauss`) reported `1 passed`.
  Spelling the type out as `Result<(), TestSupportError>` made the identical
  broken assertion fail (`Step failed at index 2 ... expectation failed`).
  Impact: the `rstest-bdd` step macro classifies return types syntactically and
  does not resolve aliases, so an alias step is treated as value-returning and
  its `Err` is discarded. This is a silent false-green — the worst failure
  class for a test framework. All step signatures now spell out `Result<..>`.
  Filed as the FLAGGED CONCERN in the beta tester's log with a full validation
  matrix and upstream suggestions.

- Observation: a *fallible* scenario (`-> Result<(), E>`) trips
  `unused_must_use` in the generated `#[gpui::test]` body (a hard error under
  `-D warnings`). Evidence:
  `warning: unused Result that must be used --> ...#[scenario(...`, escalated
  to an error by `cargo clippy -- -D warnings`. Impact: a unit (`()`) scenario
  still propagates step `Err`s correctly, so the scenario is kept
  unit-returning; the fallible-scenario shape is avoided.

## Decision log

- Decision: migrate `draw_click_adds_points_and_undo_removes` (the
  draw→undo→redo flow) rather than a simpler selection test. Rationale: it is
  the richest behavioural narrative with durable `Entity<Phase0Shell>` + window
  handles shared across many steps, so it is the strongest exercise of the
  stateful GPUI harness playbook. Confirmed with the user. Date/Author:
  2026-07-08, planning session.

- Decision: replace the original `#[gpui::test]` function once the BDD scenario
  is green, rather than keeping both. Rationale: one source of truth; avoids
  duplicated-coverage drift. Confirmed with the user. Date/Author: 2026-07-08,
  planning session.

- Decision: add `rstest-bdd-harness-gpui` and `serial_test` as dev-dependencies
  of the root crate only (not `gauss-core`/`gauss-svg`, whose BDD suites are
  non-GPUI). Rationale: the GPUI integration tests live in the root crate's
  `tests/` directory; the harness and serial gate are only needed there.
  Date/Author: 2026-07-08, planning session.

- Decision: verified the published `gpui 0.2.2` test API against the crate
  source rather than trusting the guide's vendored snippets:
  `VisualTestContext::from_window(window, cx: &TestAppContext) -> Self` (owned,
  takes a shared `&TestAppContext`), `window_handle()` is a
  `gpui::VisualContext` trait method, and `add_window_view` returns
  `(Entity<V>, &mut VisualTestContext)`. Rationale: avoid compile churn and
  match the gate-passing house style. Date/Author: 2026-07-08, implementation.

- Decision: step and helper bodies use `let … else { panic!(…) }` for
  Option/Result unwrapping, not `.expect(...)`. Rationale: the plan assumed the
  `clippy.toml` `allow-expect-in-tests = true` allowance would cover the step
  code. It does not — that allowance only exempts `#[test]`/`#[gpui::test]`
  function bodies, and `#[given]`/`#[when]`/`#[then]` steps (and their helpers)
  are plain functions, so `expect_used`/`unwrap_used` fire there under
  `--all-targets` clippy. `shadow_reuse` also had to be avoided by binding the
  handle tuple to a separate name before the `let … else`. The playbook's own
  `let … else { panic!(…) }` shape passes the pedantic profile. Date/Author:
  2026-07-08, implementation (surfaced by the milestone-2 gate).

- Decision: on user request, make steps and helpers fully fallible (return
  `Result`, propagate with `?` / explicit `Err`) instead of panicking, and keep
  the scenario unit-returning. Step signatures spell out
  `Result<(), TestSupportError>` (never the `TestSupportResult` alias).
  Rationale: the user asked to avoid panics in fixtures/steps and prefer
  fallible functions, and to validate that a `#[then]` is a real test.
  Validation uncovered that the `Result` type alias silently swallows step
  `Err`s (false green) and that a fallible scenario trips `unused_must_use`
  under `-D warnings`; the spelled-out-step + unit-scenario shape is the only
  combination that is both a genuine assertion and lint-clean. `with_visual_cx`
  returns `Err` for the missing-handle invariant so the last panic is gone.
  Date/Author: 2026-07-08, implementation (post-milestone-2 follow-up).

- Decision: substitute a falsification check for a literal red-skeleton stage.
  Rationale: the deliverable *is* a test, so "fails before implementation" is
  best evidenced by implementing it, observing green, then temporarily breaking
  one `Then` expectation to observe a red for the intended reason and
  reverting. This is the execplans-sanctioned nearest observable substitute.
  Date/Author: 2026-07-08, implementation.

## Outcomes & retrospective

Both objectives were met. The workspace now builds and tests against
`rstest-bdd 0.6.0-beta3`, and the draw/undo/redo GPUI behaviour runs as a
Gherkin scenario driven through `rstest_bdd_harness_gpui::GpuiHarness` against
the **published** `gpui 0.2.2`. `make all` is green (909 tests; net-zero count
after swapping one `#[gpui::test]` for the BDD scenario), and a
broken-assertion falsification proved the harness attributes failures to the
feature path, line, scenario name, and failing step.

What went well:

- The headline risk (harness/`gpui 0.2.2` version incompatibility) did not
  materialize: `rstest-bdd-harness-gpui 0.6.0-beta3` shares the single
  `gpui 0.2.2` node, so `TestAppContext` unified with no `[patch]` gymnastics.
- Verifying the published `gpui` API against the crate source up front avoided
  the vendored-vs-published compile churn the plan anticipated.
- `harness = GpuiHarness` inferred `#[gpui::test]` with no `attributes = ...`.

What would be done differently:

- The plan assumed `.expect(...)` would be fine in step bodies under
  `allow-expect-in-tests`. It was not, because steps are plain functions.
  Future BDD-on-strict-clippy work should reach for `let … else { panic!(…) }`
  from the start. Captured as beta feedback in the tester's log.
- Two avoidable stop-hook fmt failures came from hand-wrapping `use` lists and
  closures; running `cargo fmt` immediately after each new file would have
  prevented them.

Post-review follow-up (fallible steps + `#[then]` concern): on request, the
steps and helpers were made fallible (no panics; `Result` + `?`). Validating
that a `#[then]` is a genuine assertion uncovered **two real `rstest-bdd`
defects** — the headline outcomes of the whole trial:

1. A step declared with a `Result` **type alias** silently swallows its `Err`,
   turning a failing `#[then]` into a false green. Fix: spell out `Result<..>`
   in step signatures.
2. A fallible scenario return trips `unused_must_use` in the generated
   `#[gpui::test]` body (hard error under `-D warnings`). Mitigation: keep the
   scenario unit-returning; it still propagates step `Err`s.

Both are documented with a validation matrix and upstream suggestions in the
beta tester's log, and filed upstream as leynos/rstest-bdd#573 (alias-swallow
false-green) and leynos/rstest-bdd#574 (fallible-scenario `unused_must_use`).
The affected sites in `tests/gpui_draw_undo_bdd.rs` carry `TODO(...#573)` /
`TODO(...#574)` comments. The first is severe (a silent false-green in a test
framework) and is the most important thing this trial found.

## Context and orientation

`gauss` is a Rust vector-drawing application built on GPUI. The workspace root
crate is `gauss`; member crates are `crates/gauss-core`, `crates/gauss-svg`, and
`crates/test_support`.

Behaviour tests already use `rstest-bdd 0.5.0` in three manifests:

- `Cargo.toml` (root) — dev-dependencies `rstest`, `rstest-bdd`,
  `rstest-bdd-macros`.
- `crates/gauss-core/Cargo.toml` — same three.
- `crates/gauss-svg/Cargo.toml` — same three.

There are two distinct test styles in the root crate's `tests/` directory:

- Non-GPUI BDD suites (`tests/i18n_bdd.rs`, `tests/a11y_service_bdd.rs`,
  `tests/widget_capability_audit_bdd/`, `tests/undo_entry_count_bdd/`,
  `tests/a11y_service_routing_bdd.rs`) using `#[scenario]`/`#[given]`/
  `#[when]` /`#[then]` with plain `rstest` fixtures. `tests/i18n_bdd.rs` is a
  clean reference for the current binding style.
- Raw GPUI integration tests (`tests/gpui_*.rs`, 40+ files) using
  `#[gpui::test]` directly with a shared helper module `tests/common/mod.rs`.
  These are **not** currently BDD; they are ordinary GPUI tests.

The migration target, `tests/gpui_history_draw_undo.rs`, contains three
`#[gpui::test]` functions. Only `draw_click_adds_points_and_undo_removes`
(lines ~163–243) is being migrated. It:

1. Initializes the app (`common::init_test_app`).
2. Opens a window with
   `cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx))`,
   obtaining a durable `Entity<Phase0Shell>` and a `VisualTestContext`.
3. Draws the initial frame (`common::ensure_initial_draw`).
4. Computes two canvas points (`common::canvas_points`).
5. Clicks the first point → draw shape has 1 anchor; clicks the second → 2
   anchors; undoes → 1 anchor; undoes → shape absent; redoes → 1 anchor; redoes
   → 2 anchors, asserting document state after each.

The reusable helpers it needs already live in `tests/common/mod.rs`:
`init_test_app`, `ensure_initial_draw`, `canvas_points`, `read_document`,
`read_history_len`, `find_draw_shape`, `require_draw_shape`,
`click_canvas_and_wait`, `simulate_document_undo`, `simulate_document_redo`.
The file-local helpers (`ExpectedDrawShapeState`, `assert_draw_shape_state`,
`assert_draw_shape_absent`, `ClickVerification`, `click_and_verify_state`,
`apply_action_and_verify`, `undo_and_verify_state`, `undo_and_verify_absent`,
`redo_and_verify_state`) are used **only** by the function being migrated and
become dead code once it is removed.

Key terms:

- **Harness / `GpuiHarness`**: an `rstest-bdd` adapter that wraps scenario
  execution so steps run inside a GPUI `TestAppContext`. Selected with
  `#[scenario(..., harness = rstest_bdd_harness_gpui::GpuiHarness)]`. When
  `attributes = ...` is omitted the macro infers `GpuiAttributePolicy`, which
  emits `#[gpui::test]` for the generated test.
- **Harness context / reserved fixture key `rstest_bdd_harness_context`**: the
  channel through which a step requests the harness-provided value. For
  `GpuiHarness` that value is `gpui::TestAppContext`. A step receives it with
  `#[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext`.
- **Stateful GPUI playbook**: the v0.6 interim pattern for scenarios whose steps
  must share durable handles *and* borrow `&mut TestAppContext`. Because the
  v0.6 `StepContext` cannot lend two mutable borrows in one step, durable
  handles live in a `thread_local!` cell instead of a second fixture. Each
  scenario is `#[serial]` and observes a two-sided reset protocol
  (reset-before-assignment and a `Drop`-based reset-after-scenario).

### Published vs vendored gpui (must-read before writing steps)

The upstream user's guide and its regression suite are written against the
*vendored* gpui under `vendor/gpui`. `gauss` uses the *published* `gpui 0.2.2`.
Use the published shapes:

- `add_window_view` closure takes **two** arguments:
  `cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx))` and returns
  `(Entity<V>, &mut VisualTestContext)`.
- Obtain the window handle via `visual_cx.window_handle()`; `window_handle` is a
  `gpui::VisualContext` trait method, so add `use gpui::VisualContext;`.
- `gpui::VisualTestContext::from_window(window, cx)` returns a
  `VisualTestContext` **by value** (no `Option`).
- `read_entity`/`update_entity` return `R` directly (no `Option`/`Result`
  wrappers). The migrated scenario does not need these; it reads document state
  through the existing `common::read_document` helper.

The harness itself only ever deals in `TestAppContext`, so it is unaffected by
this divergence; only the step call-sites differ.

## Plan of work

### Stage A — audit and dependency inventory (no functional change)

Establish the exact edit set and confirm the breaking-change surface.

1. Confirm the three manifests pinning `rstest-bdd = "0.5.0"`:
   run `grep -rn "rstest-bdd" --include=Cargo.toml .`.
2. Audit the underscore-normalization breaking change. Run the following on
   one line:
   `grep -rnE "fn [a-z0-9_]+\s*\(" --include="*.rs" tests crates/*/tests`
   `| grep -E "\b_[a-z]"` and inspect any hit whose parameter both starts with
   `_` and is a fixture (not a placeholder). Where a literal `_name` fixture
   key was intended, plan a `#[from(_name)]` edit. Expectation from the initial
   survey: no changes required.
3. Confirm no `scenarios!(..., runtime = "tokio-current-thread")` and no custom
   `HarnessAdapter` implementations exist:
   `grep -rn "tokio-current-thread\|HarnessAdapter" --include="*.rs" .`.
   Expectation: none (gauss has no tokio integration).

Validation: the audit produces a concrete, small edit list (expected: three
version bumps plus two new dev-dependencies, zero production-source edits).

### Stage B — prototyping spike: prove harness/gpui compatibility (go/no-go)

This is the highest-risk unknown and is deliberately isolated so it can be
deleted or promoted cleanly.

1. In the root `Cargo.toml` `[dev-dependencies]`, temporarily add
   `rstest-bdd-harness-gpui = "0.6.0-beta3"` and `serial_test = "3"`, and bump
   `rstest-bdd`/`rstest-bdd-macros` to `0.6.0-beta3`.
2. Add a throwaway `tests/harness_gpui_spike.rs` with the smallest possible
   harness-backed scenario and a single step that only proves the injected type
   is the workspace's `gpui::TestAppContext`:

   ```rust
   // tests/harness_gpui_spike.rs (throwaway; deleted at end of Stage B)
   use rstest_bdd_macros::{given, scenario};
   use serial_test::serial;

   #[given("a gpui test context")]
   fn a_gpui_test_context(
       #[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext,
   ) {
       // Touch a TestAppContext-only method to force type unification with
       // the workspace's published gpui 0.2.2.
       let _ = cx.windows();
   }

   #[scenario(
       path = "tests/features/harness_gpui_spike.feature",
       harness = rstest_bdd_harness_gpui::GpuiHarness,
   )]
   #[serial]
   fn harness_gpui_spike() {}
   ```

   with `tests/features/harness_gpui_spike.feature`:

   ```gherkin
   Feature: GPUI harness spike

     Scenario: The harness injects a gpui test context
       Given a gpui test context
   ```

3. Run `cargo test --test harness_gpui_spike`.

Go/no-go: if it compiles and passes, the harness unifies with published
`gpui 0.2.2` — proceed. If it fails to compile because of a duplicate/mismatched
`gpui` (inspect with `cargo tree -i gpui`), stop, record the versions in the
tester's log and this Decision Log, and escalate per the compatibility
tolerance. Delete `tests/harness_gpui_spike.rs` and its feature file once the
result is recorded (keep the dependency edits — they are needed for Stage C
onward).

### Stage C — bump the dependency across the workspace

1. In `Cargo.toml`, `crates/gauss-core/Cargo.toml`, and
   `crates/gauss-svg/Cargo.toml`, change `rstest-bdd` and `rstest-bdd-macros`
   from `"0.5.0"` to `"0.6.0-beta3"`. Leave `rstest = "0.26.1"` unchanged.
2. Apply any `#[from(_name)]` edits identified in Stage A (expected: none).
3. Run the existing BDD suites and the wider gate.

Validation: `make test` passes; specifically the existing behavioural suites
(`cargo test --test i18n_bdd`, `--test a11y_service_bdd`, and the gauss-core /
gauss-svg BDD tests via `cargo test -p gauss-core -p gauss-svg`) remain green.

### Stage D — red: feature file and failing skeleton

1. Add `tests/features/draw_undo.feature` (content under "Interfaces and
   dependencies").
2. Add `tests/gpui_draw_undo_bdd.rs` with the scenario binding and step
   signatures but with a deliberately unimplemented body (e.g. steps that
   `todo!()` or assert a wrong count) so the scenario fails.

Validation: `cargo test --test gpui_draw_undo_bdd` fails, and it fails for the
intended reason (a step assertion/`todo!`, not a compile error or a
missing-step registry panic). Record the failure transcript.

### Stage E — green: implement steps and thread-local state

Implement, in `tests/gpui_draw_undo_bdd.rs`:

1. The `ScenarioState` container, `thread_local!` cell, `reset_*` helpers,
   `ScenarioStateCleanup` `Drop` guard, and `scenario_state_cleanup` fixture
   (shapes under "Interfaces and dependencies").
2. The `#[given]` that resets state, initializes the app, opens the window,
   draws the initial frame, computes the two canvas points, and stores durable
   handles + points in the cell.
3. The `#[when]`/`#[then]` steps that rebuild a `VisualTestContext` from the
   stored window handle and the fresh `&mut TestAppContext` each time, then
   drive clicks/undo/redo and assert document state via `common::read_document`
   / `common::require_draw_shape` / `common::find_draw_shape`.

Reuse `tests/common/mod.rs` by declaring `mod common;` in the new file, exactly
as the other `gpui_*.rs` tests do.

Validation: `cargo test --test gpui_draw_undo_bdd` passes. Then a falsification
check: temporarily change one `Then` expected count and confirm the scenario
fails; revert. Temporarily remove the `Drop` reset and confirm behaviour is
observably affected on a repeated run (document in the log), then revert.

### Stage F — refactor/cleanup: remove the original

1. Delete the `draw_click_adds_points_and_undo_removes` function from
   `tests/gpui_history_draw_undo.rs` and every file-local helper that becomes
   unused as a result (listed in Context and orientation). Keep the other two
   `#[gpui::test]` functions and any helpers they still use (`canvas_bounds`,
   `read_document`, `require_draw_shape`, `find_draw_shape`).
2. Run `make lint` to surface any newly-dead code; remove leftovers until clean.

Validation: `make test` passes. `cargo test --test gpui_history_draw_undo`
still runs the two remaining tests and passes.

### Stage G — documentation

1. Append final entries to
   `~/docs/rstest-bdd-v0-6-0-beta3-gauss-testers-log.md`.
2. Complete this plan's `Outcomes & retrospective`, `Surprises & discoveries`,
   and `Progress`.

## Concrete steps

Run all commands from the worktree root
`/data/leynos/Projects/gauss.worktrees/adopt-rstest-bdd-v0-6-0-beta3`.

Stage A:

```bash
grep -rn "rstest-bdd" --include=Cargo.toml .
grep -rn 'runtime = "tokio-current-thread"' --include="*.rs" .
grep -rn "HarnessAdapter" --include="*.rs" .
```

Expected: three `rstest-bdd*` lines per manifest (nine total); no tokio-runtime
or `HarnessAdapter` hits.

Stage B (spike), after editing the root `Cargo.toml` dev-dependencies:

```bash
cargo tree -i gpui            # expect a single gpui 0.2.2 in the tree
cargo test --test harness_gpui_spike
```

Expected transcript (success):

```plaintext
test harness_gpui_spike ... ok
test result: ok. 1 passed; 0 failed; ...
```

Stage C:

```bash
make test
cargo test -p gauss-core -p gauss-svg
```

Stage D/E:

```bash
cargo test --test gpui_draw_undo_bdd   # fails at Stage D, passes at Stage E
```

Stage F:

```bash
cargo test --test gpui_history_draw_undo
make test
```

Feature-file caveat: if only `tests/features/draw_undo.feature` changed since
the last build, run `cargo clean -p gauss` before re-testing so the macro
re-reads it.

## Validation and acceptance

Acceptance is behavioural:

- Red: `cargo test --test gpui_draw_undo_bdd` fails before the steps are
  implemented, for a step-level reason (assertion or `todo!`), not a compile or
  missing-step error.
- Green: after Stage E, `cargo test --test gpui_draw_undo_bdd` reports the
  single scenario passing.
- Regression: `make test` passes; the two retained tests in
  `tests/gpui_history_draw_undo.rs` still pass; all pre-existing `*_bdd.rs`
  suites still pass under `0.6.0-beta3`.
- Falsification evidence: altering one `Then` count makes the scenario fail;
  reverting restores green.

Quality criteria for "done":

- Tests: `make test` green, including the new scenario and all retained suites.
- Lint/typecheck: `make lint` and `make check-fmt` pass with no new
  `#[expect]` beyond file-scoped lint accommodations that mirror existing test
  files.
- Dependencies: exactly two new dev-dependencies added
  (`rstest-bdd-harness-gpui`, `serial_test`); zero production-source edits.

Quality method: run `make all` (format check, lint, test) as the final gate.

## Idempotence and recovery

- The version-bump edits (Stage C) are pure `Cargo.toml` string changes and are
  trivially reversible with `git checkout -- Cargo.toml crates/*/Cargo.toml`.
- The Stage B spike files are throwaway; deleting them leaves no residue.
- If Stage B fails the go/no-go gate, revert the harness/`serial_test`
  dev-dependency additions but keep (or also revert) the version bump per the
  escalation decision, and stop.
- Re-running any `cargo test --test ...` command is safe and side-effect-free
  (GPUI tests use headless `TestAppContext`; temp files are guarded by
  `TempFileGuard`).

## Interfaces and dependencies

### Manifests

Root `Cargo.toml` `[dev-dependencies]` after the change:

```toml
rstest = "0.26.1"
rstest-bdd = "0.6.0-beta3"
rstest-bdd-macros = "0.6.0-beta3"
rstest-bdd-harness-gpui = "0.6.0-beta3"
serial_test = "3"
```

`crates/gauss-core/Cargo.toml` and `crates/gauss-svg/Cargo.toml`: bump
`rstest-bdd` and `rstest-bdd-macros` to `"0.6.0-beta3"` only (no harness, no
`serial_test`).

### Feature file — `tests/features/draw_undo.feature`

```gherkin
Feature: Draw-mode undo and redo

  Scenario: Draw clicks add anchors and undo removes them
    Given a fresh Phase 0 shell window
    When the first anchor is placed
    Then the draw shape anchor count is 1
    When the second anchor is placed
    Then the draw shape anchor count is 2
    When the last change is undone
    Then the draw shape anchor count is 1
    When the last change is undone
    Then the draw shape is absent
    When the last change is redone
    Then the draw shape anchor count is 1
    When the last change is redone
    Then the draw shape anchor count is 2
```

### Step module — `tests/gpui_draw_undo_bdd.rs`

Shapes that must exist at the end of the milestone (published `gpui 0.2.2` API):

```rust
mod common;

use common::{
    canvas_points, ensure_initial_draw, find_draw_shape, init_test_app,
    read_document, require_draw_shape,
    click_canvas_and_wait, simulate_document_redo, simulate_document_undo,
};
use gauss::ui::Phase0Shell;
use gpui::{AnyWindowHandle, Entity, Pixels, Point, TestAppContext, VisualTestContext};
use gpui::VisualContext; // window_handle() lives on this trait for published gpui
use rstest::fixture;
use rstest_bdd_macros::{given, scenario, then, when};
use serial_test::serial;
use std::cell::RefCell;

#[derive(Default)]
struct ScenarioState {
    entity: Option<Entity<Phase0Shell>>,
    window: Option<AnyWindowHandle>,
    first: Option<Point<Pixels>>,
    second: Option<Point<Pixels>>,
}

thread_local! {
    static SCENARIO_STATE: RefCell<ScenarioState> =
        RefCell::new(ScenarioState::default());
}

fn with_state<R>(f: impl FnOnce(&mut ScenarioState) -> R) -> R { /* SCENARIO_STATE.with(...) */ }
fn reset_state_after_scenario() { /* *cell.borrow_mut() = ScenarioState::default() */ }
fn reset_state_before_assignment() { reset_state_after_scenario() }

struct ScenarioStateCleanup;
impl Drop for ScenarioStateCleanup {
    fn drop(&mut self) { reset_state_after_scenario() }
}

#[fixture]
fn scenario_state_cleanup() -> ScenarioStateCleanup {
    reset_state_before_assignment();
    ScenarioStateCleanup
}

// Rebuild a fresh VisualTestContext from the stored window handle and the
// harness-provided TestAppContext, then run `f` with the durable view entity.
fn with_visual_cx<R>(
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut VisualTestContext, &Entity<Phase0Shell>) -> R,
) -> R {
    let (entity, window) = with_state(|s| (
        s.entity.clone().expect("entity handle set by the given step"),
        s.window.expect("window handle set by the given step"),
    ));
    // Published gpui 0.2.2: from_window returns VisualTestContext by value.
    let mut visual_cx = VisualTestContext::from_window(window, cx);
    f(&mut visual_cx, &entity)
}

#[given("a fresh Phase 0 shell window")]
fn fresh_phase0_shell_window(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
) {
    reset_state_before_assignment();
    init_test_app(cx);
    let (entity, visual_cx) =
        cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx));
    ensure_initial_draw(visual_cx);
    let window = visual_cx.window_handle();
    let (first, second) =
        canvas_points(visual_cx).expect("canvas points should be available");
    with_state(|s| {
        s.entity = Some(entity);
        s.window = Some(window);
        s.first = Some(first);
        s.second = Some(second);
    });
}

#[when("the first anchor is placed")]
fn place_first_anchor(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    let first = with_state(|s| s.first.expect("first point set"));
    with_visual_cx(cx, |vcx, _view| click_canvas_and_wait(vcx, first));
}

#[when("the second anchor is placed")]
fn place_second_anchor(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    let second = with_state(|s| s.second.expect("second point set"));
    with_visual_cx(cx, |vcx, _view| click_canvas_and_wait(vcx, second));
}

#[when("the last change is undone")]
fn undo_last_change(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    with_visual_cx(cx, |vcx, _view| simulate_document_undo(vcx));
}

#[when("the last change is redone")]
fn redo_last_change(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    with_visual_cx(cx, |vcx, _view| simulate_document_redo(vcx));
}

#[then("the draw shape anchor count is {count:usize}")]
fn draw_shape_anchor_count(
    #[from(rstest_bdd_harness_context)] cx: &mut TestAppContext,
    count: usize,
) {
    with_visual_cx(cx, |vcx, view| {
        let doc = read_document(vcx, view);
        let shape = require_draw_shape(&doc, "draw_undo scenario")
            .expect("draw shape should be present");
        assert_eq!(shape.path.anchors.len(), count);
    });
}

#[then("the draw shape is absent")]
fn draw_shape_absent(#[from(rstest_bdd_harness_context)] cx: &mut TestAppContext) {
    with_visual_cx(cx, |vcx, view| {
        let doc = read_document(vcx, view);
        assert!(find_draw_shape(&doc).is_none(), "draw shape should be absent");
    });
}

#[scenario(
    path = "tests/features/draw_undo.feature",
    name = "Draw clicks add anchors and undo removes them",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial]
fn draw_undo_scenario(
    #[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup,
) {
}
```

Notes on the shapes above:

- The `.expect(...)` calls will trip the workspace `expect_used` lint. Mirror
  `tests/common/mod.rs`: add a file-scoped
  `#![expect(clippy::expect_used, reason = "integration test invariants")]` (and
  `clippy::float_arithmetic` if any geometry arithmetic is added) with a
  `reason`. Confirm the exact set against `make lint` output rather than
  guessing.
- With `harness = GpuiHarness` and `attributes` omitted, the macro infers
  `GpuiAttributePolicy`, emitting `#[gpui::test]` for `draw_undo_scenario`.
- Every step borrows only one mutable fixture from `StepContext` (the harness
  context). Durable handles live in the thread-local cell, which is the whole
  point of the interim playbook and avoids the `E0499` two-mutable-borrow error.

## Revision note

Initial draft. Establishes a two-part delivery: a workspace-wide
`rstest-bdd 0.5.0 → 0.6.0-beta3` bump (Stages A, C) gated by a harness/
`gpui 0.2.2` compatibility spike (Stage B), followed by a red→green→refactor
migration of the `draw_click_adds_points_and_undo_removes` GPUI test to the
`GpuiHarness` stateful playbook (Stages D–F), adapted to the published
`gpui 0.2.2` API rather than the vendored fork. Awaiting approval before
implementation.

2026-07-08 — implementation complete. What changed since the draft: the Stage B
spike confirmed the harness resolves against the single published `gpui 0.2.2`
(go/no-go passed, no incompatibility). The `.expect(...)`-in-steps assumption
was wrong — `allow-expect-in-tests` does not cover `#[given]`/`#[when]`/
`#[then]` step functions, so step and helper bodies use
`let … else { panic!(…) }` (Decision Log). Two stop-hook fmt failures from
hand-wrapped `use` lists and closures were resolved with `cargo fmt`. Both
milestones passed `make all` (909 tests) and CodeRabbit (`review --agent`, 0
findings each). No remaining work; Status is COMPLETE.
