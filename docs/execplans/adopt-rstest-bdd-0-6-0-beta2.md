# Adopt rstest-bdd v0.6.0-beta2 and migrate a GPUI behavioural test to the GPUI harness

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Today the `gauss` crate pins `rstest-bdd` and `rstest-bdd-macros` to `0.5.0`
(see `Cargo.toml` lines 99–100). The GPUI behavioural tests under `tests/` (for
example `tests/gpui_shell_mode_indicator.rs`) are written directly against
GPUI's `#[gpui::test]` attribute and a hand-rolled shared helper module
(`tests/common/mod.rs`); they are *not* Gherkin/BDD scenarios. The rstest-bdd
project has since shipped a v0.6.0 line that adds a first-party **GPUI harness**
(`rstest-bdd-harness-gpui`) which lets Gherkin scenarios run inside GPUI's
headless test app and inject `gpui::TestAppContext` into steps through the
reserved fixture key `rstest_bdd_harness_context`.

After this change a developer can:

1. Build and test the workspace against `rstest-bdd` 0.6.0-beta2 with the new
   harness crates available, with `make check-fmt`, `make lint`, and
   `make test` all green.
2. Run one GPUI behavioural scenario expressed as a Gherkin `.feature` file
   driven by the `rstest_bdd_harness_gpui::GpuiHarness`, observable as a
   passing test (`scenario_mode_indicator_reflects_tool_and_edge_mode`) that
   fails if the mode-indicator behaviour regresses.
3. Read a written record (this plan plus a migration note) of how the adoption
   was performed, what friction was hit, and what gaps remain in the upstream
   migration guide and user's guide.

The deliverable also includes three forward-looking proposals captured in this
plan: the order to migrate the remaining GPUI behavioural tests, gaps in the
rstest-bdd v0.6.0 roadmap that should be resolved, and improvements to the
harness, its interface, or its documentation that should be rolled into the
v0.6.0 or v1.0.0 roadmap.

### Signposted documentation and skills

Read these before and during implementation. They are the source of truth.

- Migration guide (in this repo):
  `docs/rstest-bdd-v0-6-0-migration-guide.md`. Authoritative for the breaking
  changes, the harness dependency matrix, the GPUI harness configuration, the
  stateful-GPUI playbook, and the `E0499`/`E0502` troubleshooting entry.
- User's guide (in this repo): `docs/rstest-bdd-users-guide.md`. This is the
  latest rstest-bdd user's guide; the migration guide links its "Stateful GPUI
  scenarios with durable handles" section (from line 1088), which carries a
  complete copy-pasteable playbook (feature file, steps, scenario binding,
  reset protocol) that this plan mirrors.
- Upstream repo (sibling checkout): `/data/leynos/Projects/rstest-bdd`.
  - Roadmap: `docs/roadmap.md` (phases 9, 10, 11, 12 cover harness adapters,
    beta2 quick wins, v0.6.1, and the v0.7.0 redesign).
  - GPUI harness crate: `crates/rstest-bdd-harness-gpui/` (public API in
    `src/lib.rs`, `src/gpui_harness.rs`, `src/policy.rs`).
  - Worked minimal example: `examples/gpui-counter/` (Cargo.toml, feature file
    `tests/features/counter.feature`, test `tests/counter.rs`).
  - Stateful regression suite:
    `crates/rstest-bdd-harness-gpui/tests/stateful_window.rs` (the canonical
    thread-local durable-handle pattern).
  - Base harness crate: `crates/rstest-bdd-harness/src/lib.rs` (the
    `HarnessAdapter` trait, `ScenarioRunRequest`, `HarnessResult`).
  - ADR-007: `docs/adr-007-harness-context-injection.md` (why the reserved
    fixture key exists).
  - Design doc borrow-constraint sections:
    `docs/rstest-bdd-design.md` §§2.7.6.1–2.7.6.5.
- Existing in-repo BDD examples to copy the house style from:
  `tests/i18n_bdd.rs` (scenario + step + fixture shape) and
  `tests/widget_capability_audit_bdd/` (multi-file BDD layout).
- Repo conventions: `AGENTS.md` (Rust style, lint policy, en-GB-oxendict
  spelling, 400-line file limit, `make` gates) and
  `docs/documentation-style-guide.md`.

Claude skills to load when touching the relevant area:

- `execplans` — maintaining this living plan.
- `leta` — symbol navigation and references across `gauss` and the harness
  crate (load first; create a workspace for the worktree).
- `rust-router` — entry point for Rust language skills; route to
  `rust-types-and-apis` for the harness trait/fixture shapes,
  `rust-memory-and-state` for the `StepContext::borrow_mut` borrow constraint
  and the thread-local interim pattern, and `rust-errors` for fallible fixtures
  and scenario return types.
- `domain-cli-and-daemons` is *not* relevant here; the work is test harness
  adoption inside a GPUI desktop app.

## Constraints

Hard invariants that must hold throughout implementation. Violation requires
escalation, not a workaround.

- Do not regress any existing test. Every current `tests/gpui_*.rs`,
  `tests/*_bdd*.rs`, and unit test must still pass. The migrated test replaces
  the behaviour coverage of `tests/gpui_shell_mode_indicator.rs`; the original
  file is removed only once the BDD replacement passes and asserts the same
  three behaviours.
- `make check-fmt`, `make lint`, and `make test` must all succeed at the end of
  every committed milestone. `make lint` runs Clippy with `-D warnings`,
  `cargo doc`, and Whitaker; `make test` runs nextest with
  `--workspace --all-targets --all-features`.
- The workspace lint policy in `Cargo.toml` (`unwrap_used`, `expect_used`,
  `indexing_slicing`, `float_arithmetic`, `missing_docs`, etc., all `deny`)
  must be honoured. New test code uses `.expect(...)` over `.unwrap()` and adds
  tightly scoped `#[expect(..., reason = "...")]` only as a last resort, never
  blanket `#[allow]`.
- All prose and comments use en-GB-oxendict spelling ("-ize"/"-yse"/"-our").
- No code file exceeds 400 lines (`AGENTS.md`). The migrated GPUI BDD test must
  be split into `main.rs` + `steps.rs` (a directory module) if a single file
  would approach that limit; the i18n example fits in one file, so a single
  file is acceptable if it stays well under 400 lines.
- Dependencies use SemVer caret requirements; no `*` or open-ended `>=`. The
  pre-release pin is the explicit exception documented in the Decision Log.
- Do not restore any root `[patch.crates-io]` table. The upstream guide
  forbids it for v0.6.0 (migration guide "Workspace dependency migration for
  contributors"). `gauss` is a *downstream consumer*, so it depends on
  published crate versions only.
- Do not modify production code in `src/` to make the test pass unless a
  genuine bug is found; the migration is test-infrastructure work. Any required
  `src/` change is escalated first.

## Tolerances (exception triggers)

Thresholds that trigger escalation when breached.

- Scope: if making the single migrated scenario pass requires changing more
  than 6 files (excluding this plan and the migration note), or more than ~250
  net lines, stop and escalate.
- Dependencies: adding `rstest-bdd-harness-gpui`, `serial_test`, and (if
  required) `rstest-bdd-harness` as dev-dependencies is in scope. Any *other*
  new dependency triggers escalation.
- Interface: if the migration appears to require a change to a public
  `Phase0Shell` signature or any `src/` public API, stop and escalate.
- Availability: if `rstest-bdd` 0.6.0-beta2 (and the harness crates) cannot be
  resolved from the configured registry/source, stop and escalate (see Risks).
- Iterations: if `make test` still fails on the migrated scenario after 4
  focused fix attempts, stop and escalate with the captured diagnostics.
- Ambiguity: if the GPUI harness requires native platform libraries that are
  unavailable in this environment (so the scenario cannot run headless), stop
  and present options (skip-gating vs. environment change).

## Risks

- Risk: `rstest-bdd` 0.6.0-beta2 and the `-harness-gpui` crate may not be
  published to crates.io, or may need a path/git source pointing at
  `/data/leynos/Projects/rstest-bdd`. Severity: high Likelihood: medium
  Mitigation: Stage A resolves this empirically with `cargo search`/
  `cargo update` dry runs and by inspecting the sibling repo's published
  versions (`crates/rstest-bdd-harness-gpui/Cargo.toml` is `0.6.0-beta2`). If
  unpublished, escalate per the Dependencies tolerance with the two source
  options (git tag vs. path) and their trade-offs; do not silently add a path
  dependency.

- Risk: the GPUI harness `GpuiHarness::run` uses `gpui::run_test`, which may
  need native windowing/platform libraries not present in this headless WSL2
  environment, causing the scenario to fail to start rather than fail an
  assertion. Severity: high Likelihood: medium Mitigation: the existing
  `tests/gpui_*.rs` already run headless GPUI under `make test` in this
  environment, so the platform support is present. Confirm in Stage A by
  running the existing `gpui_shell_mode_indicator` test in isolation. If the
  harness path needs extra libraries the raw tests do not, escalate.

- Risk: the mode-indicator scenario needs both `&mut TestAppContext` (to
  dispatch keystrokes and to run `view.update`) and durable handles to the
  `Phase0Shell` entity and window across steps. This is exactly the "two
  mutable fixtures" borrow constraint (`E0499`/`E0502`) described in the
  migration guide. Severity: medium Likelihood: high Mitigation: adopt the v0.6
  interim thread-local durable-handle pattern from the start (the stateful-GPUI
  playbook), storing `Option<gpui::Entity<Phase0Shell>>` and
  `Option<gpui::AnyWindowHandle>` in a `thread_local!` `RefCell`, rebuilding
  `VisualTestContext::from_window` per step, requesting only
  `&mut gpui::TestAppContext` from `StepContext`, marking the scenario
  `#[serial]`, and wiring a `Drop`-based reset fixture. This is the documented,
  supported shape; treat it as the design baseline, not a fallback.

- Risk: `Phase0Shell::new` takes a `gpui::Context<Phase0Shell>`; the existing
  raw test uses
  `cx.add_window_view(|_window, view_cx| Phase0Shell::new(view_cx))`. The
  harness gives steps a `&mut TestAppContext`, so the window-open step must
  reproduce that call shape against the harness context. Severity: low
  Likelihood: medium Mitigation: the stateful regression suite calls
  `context.add_window_view(|_context| CounterView::default())`; mirror that and
  confirm the `add_window_view` closure arity matches gpui 0.2.2 in this repo
  (the raw test uses the two-argument closure form). Resolve the exact arity in
  Stage A by reading gpui's `TestAppContext::add_window_view` signature.

- Risk: nextest may run BDD scenarios in parallel across test binaries, racing
  on GPUI's single-threaded, `Rc`-backed state and the shared thread-local
  scenario state. Severity: medium Likelihood: medium Mitigation: `#[serial]`
  from `serial_test` serialises within a binary; GPUI's own single-thread
  requirement is satisfied because each `#[gpui::test]`/ harness run builds its
  own `TestAppContext`. Keep the migrated scenario in its own integration-test
  binary (its own `tests/<name>.rs` or `tests/<name>/` directory) so
  cross-binary parallelism does not share its thread-local.

## Progress

- [x] (2026-06-09T00:30Z) Stage A research: read migration guide, user's guide,
  upstream roadmap, harness crate, gpui-counter example, stateful regression
  suite, base harness API, ADR-007, and design §2.7.6.x; confirmed baseline
  `cargo check --workspace --all-targets --all-features` is green and that the
  reported LSP diagnostics are false positives (the `test-support`-gated
  `Phase0Shell` test helpers compile under `--all-features`).
- [x] (2026-06-09T01:10Z) Stage A (remaining): confirmed all four crates are
  resolvable from crates.io at `0.6.0-beta2`/`serial_test 3`; ran the existing
  `gpui_shell_mode_indicator` test in isolation (passes, headless GPUI works);
  resolved exact gpui 0.2.2 signatures (recorded in Surprises & Discoveries);
  confirmed the 0.5→0.6 breaking changes (underscore fixtures, custom harness
  adapters) do not affect any existing in-repo BDD test.
- [x] (2026-06-09T01:40Z) Stage B: bumped dev-dependencies to 0.6.0-beta2, added
  `rstest-bdd-harness-gpui` and `serial_test`; created
  `tests/features/shell_mode_indicator.feature` and the directory test target
  `tests/gpui_shell_mode_indicator_bdd/{main.rs,steps.rs}`. The harness macro
  wiring compiled on the first attempt after adding `use gpui::AppContext`.
- [x] (2026-06-09T01:50Z) Stage C: implemented the thread-local durable-handle
  steps; scenario passes (green). Verified the assertion bites by corrupting an
  expected value (clear named failure) and restoring it. Removed the superseded
  `tests/gpui_shell_mode_indicator.rs`.
- [x] (2026-06-09T02:20Z) Stage D: all gates green — `make check-fmt` (cargo
  fmt + markdownlint, 0 errors), `make lint` (clippy + whitaker dylint suite),
  and `make test` (909 passed, 1 skipped, including
  `gpui_shell_mode_indicator_bdd::scenario_mode_indicator_reflects_tool_and_edge_mode`).
  Resolved the workspace lint constraints recorded in Surprises & Discoveries
  (`shadow_reuse`, `no_unwrap_or_else_panic` vs `expect_used`). Finalised the
  proposals and retrospective.

## Surprises & discoveries

- Observation: every existing `tests/gpui_*.rs` test is inherently *stateful* in
  rstest-bdd terms — each opens a window and keeps the `Entity<Phase0Shell>` and
  `VisualTestContext` alive across what would be separate Given/When/Then
  steps, while also needing `&mut TestAppContext`. Evidence:
  `tests/gpui_shell_mode_indicator.rs` lines 19–40 hold `view` and `visual_cx`
  across the keystroke and the manipulate-mode switch; `tests/common/mod.rs`
  helpers take `&mut VisualTestContext` plus `&Entity<Phase0Shell>`. Impact:
  even the simplest GPUI scenario must use the v0.6 interim thread-local
  durable-handle pattern; there is no "trivial stateless" GPUI test that avoids
  it. This is the single biggest source of friction and shapes the migration
  ordering proposal.

- Observation: gpui 0.2.2's test API differs materially from the API the
  user's-guide stateful playbook is written against, in four ways that each
  break a verbatim copy of the playbook snippets. Evidence (all from
  `~/.cargo/registry/src/index.crates.io-*/gpui-0.2.2/src/app/test_context.rs`
  and `src/window.rs`):
  1. `TestAppContext::add_window_view<F, V>(&mut self, F) -> (Entity<V>, &mut
     VisualTestContext)` where `F: FnOnce(&mut Window, &mut Context<V>) -> V`
     — a **two-argument** closure returning a **`&mut VisualTestContext`**
     borrow. The playbook uses a one-argument closure
     (`|_context| CounterView::default()`) and binds the result by value.
  2. `VisualTestContext.window` is a **private** field and there is **no**
     `VisualTestContext::window_handle()` accessor in 0.2.2. The playbook calls
     `visual_context.window_handle()`. The 0.2.2 way to obtain the
     `AnyWindowHandle` is `Window::window_handle()` (public, `window.rs:1362`),
     reached inside an update: `vcx.update(|window, _app| window.window_handle())`.
  3. `VisualTestContext::from_window(window: AnyWindowHandle, cx: &TestAppContext)
     -> Self` returns `Self` directly (it clones the `TestAppContext`), **not**
     `Option<VisualTestContext>`. The playbook unwraps an `Option`.
  4. Both `TestAppContext` and `VisualTestContext` define `type Result<T> = T`
     (identity), so `read_entity`/`update_entity` return `R` directly. The
     playbook expects `Ok(())`/`Some(1)` wrappers.
  Impact: the harness itself (which only deals in `TestAppContext`) is
  unaffected, but every window/entity helper call in the migrated steps must be
  adapted to gpui 0.2.2 rather than copied. This sharpens Proposal 3's "closure
  arity" gap into a broader "the playbook assumes a newer gpui than the
  harness's own MSRV-compatible 0.2.2; the guide should pin which gpui version
  its snippets target". The step design below uses the 0.2.2 shapes.

- Observation: bumping the workspace to `rstest-bdd` 0.6.0-beta2 does not
  require touching any existing BDD test. Evidence: the only two breaking
  changes are the leading-underscore implicit fixture normalization and the
  `HarnessAdapter::run -> HarnessResult<T>` signature. No scenario/step
  parameter in `tests/` is underscore-prefixed (grep for `^\s*_[a-z]\w*\s*:` and
  `fn …(_…:` both empty), and `gauss` defines no custom `HarnessAdapter`.
  Impact: the dependency bump is low-risk for the existing `i18n_bdd`,
  `undo_entry_count_bdd`, `widget_capability_audit_bdd`, and `a11y_service_bdd`
  suites; only additive harness configuration is new.

- Observation: the migrated step file collides with two workspace lint policies
  that the user's-guide playbook snippets ignore. Evidence: `make lint` failed
  first on `clippy::shadow_reuse` (the playbook re-binds
  `let key = key.trim_matches(...)` and `let mut state = state.borrow_mut()`),
  then on the in-house `whitaker` dylint `no_unwrap_or_else_panic`
  (`state.entity.unwrap_or_else(|| panic!(...))`). The obvious remedy the
  latter lint suggests — `.expect("…")` — is itself denied by
  `clippy::expect_used`. Impact: durable-handle steps in a crate with a strict
  lint profile must (a) rename trimmed/`borrow` bindings rather than shadow,
  and (b) reach for a `let … else { panic!(…) }` accessor rather than
  `unwrap_or_else`/`expect`. This feeds Proposal 3: the playbook should offer a
  lint-clean variant (no shadowing, no `unwrap_or_else`-panic) so it copies
  cleanly into pedantic crates, and Proposal 2's "scenario-local state helper"
  (roadmap 11.1.3) would remove the hand-rolled `borrow`/`panic!` accessor
  entirely.

- Observation: the reported rust-analyzer diagnostics ("no method named
  `document`/`selection`/`last_canvas_click_screen`/
  `document_history_len_for_tests` on `&Phase0Shell`") are false positives.
  Evidence: those methods exist in `src/ui/phase0_shell/test_helpers.rs`, gated
  by `#[cfg(any(test, feature = "test-support", coverage, coverage_nightly))]`
  (`src/ui/phase0_shell/mod.rs` lines 27–28).
  `cargo check --workspace --all-targets --all-features` finishes clean.
  Impact: no production fix is needed; the LSP simply is not building with
  `--all-features`. Do not "fix" these by adding methods.

## Decision log

- Decision: Select `tests/gpui_shell_mode_indicator.rs` as the single GPUI test
  to migrate to the harness in this iteration. Rationale: it is small (41
  lines) yet representative — it opens a window, draws, dispatches a keystroke
  (`tab`), mutates state through `view.update` (manipulate mode), and makes
  three distinct assertions on `mode_status_line_for_tests()`. This exercises
  window open, harness-context access, keystroke dispatch, and durable-handle
  reconstruction in one scenario, making it a stronger proof point than the
  trivial `gpui_shell_canvas_layout` (27 lines, single assertion).
  `canvas_layout` is recorded as the fallback if keystroke dispatch through the
  harness proves blocking. Date/Author: 2026-06-09 / Claude (plan author).

- Decision: Adopt the thread-local durable-handle ("stateful GPUI") pattern as
  the baseline rather than attempting a stateless shape. Rationale: the test
  needs `&mut TestAppContext` in multiple steps plus durable `Entity`/window
  handles; this is the documented two-mutable-fixture constraint. The interim
  pattern is the supported v0.6 answer. Date/Author: 2026-06-09 / Claude.

- Decision: Pin the pre-release with explicit caret-incompatible pre-release
  requirements (`= "0.6.0-beta2"` form) as an intentional, documented exception
  to the caret-only policy in `AGENTS.md`. Rationale: pre-releases are not
  matched by caret ranges of the prior stable; pinning the exact beta is the
  correct way to consume it. Revisit when 0.6.0 final ships. Date/Author:
  2026-06-09 / Claude. (Escalate in Stage A if the crates are not published,
  before editing `Cargo.toml`.)

## Context and orientation

`gauss` is a single desktop drawing application built on GPUI (Zed's UI
framework). The Phase 0 UI shell is `gauss::ui::Phase0Shell`, defined in
`src/ui/phase0_shell/mod.rs`. Test-only accessors live in
`src/ui/phase0_shell/test_helpers.rs`, gated behind
`#[cfg(any(test, feature = "test-support", ...))]`; the relevant one here is
`mode_status_line_for_tests(&self) -> String` and
`enter_manipulate_mode_for_tests(&mut self)`.

Integration tests live in `tests/`. There are two flavours:

- Raw GPUI tests using `#[gpui::test] fn name(cx: &mut TestAppContext)` plus the
  shared `mod common;` helper module (`tests/common/mod.rs`). Example:
  `tests/gpui_shell_mode_indicator.rs`.
- BDD tests using `rstest-bdd` 0.5.0 (`#[scenario(path = "...")]` with
  `#[given]`/`#[when]`/`#[then]` and an `rstest` `#[fixture]`). Example:
  `tests/i18n_bdd.rs`, with its feature file `tests/features/i18n.feature`.

The dependency surface is in `Cargo.toml`: `gpui = "0.2.2"`, dev-dependencies
`rstest = "0.26.1"`, `rstest-bdd = "0.5.0"`, `rstest-bdd-macros = "0.5.0"`,
`gpui` with `test-support`. There is a `test-support` feature on `gauss`.

Key terms:

- **Harness**: a type implementing `rstest_bdd_harness::HarnessAdapter` that
  wraps how a generated scenario test body executes. `GpuiHarness` runs it
  inside `gpui::run_test` and injects a `gpui::TestAppContext`.
- **Harness context**: the `HarnessAdapter::Context` value (here
  `gpui::TestAppContext`) made available to steps under the reserved fixture key
  `rstest_bdd_harness_context`; a step requests it with
  `#[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext`.
- **Attribute policy**: which test attributes the macro emits. The GPUI policy
  emits `#[rstest::rstest]` then `#[gpui::test]`. It is inferred automatically
  when `harness = rstest_bdd_harness_gpui::GpuiHarness` is given and
  `attributes = ...` is omitted.
- **Durable handles**: `gpui::Entity<T>` and `gpui::AnyWindowHandle`, both cheap
  to copy and valid across steps. `VisualTestContext` is *not* durable and must
  be rebuilt per step via `gpui::VisualTestContext::from_window(window, cx)`.
- **Two-sided reset protocol**: clearing the thread-local scenario state both
  before assigning fresh handles (in the opening `#[given]`) and after the
  scenario (via a `Drop` fixture), to prevent handle leakage across serial
  scenarios reusing a test thread.

## Plan of work

### Stage A — Understand and verify (no production code changes)

1. Confirm dependency availability. From the worktree root, inspect what
   versions of `rstest-bdd`, `rstest-bdd-macros`, `rstest-bdd-harness-gpui`, and
   `serial_test` are resolvable. Cross-check the sibling repo's
   `crates/rstest-bdd-harness-gpui/Cargo.toml` (`version = "0.6.0-beta2"`). If
   the crates are unpublished, escalate with the path-vs-git source options.
2. Confirm the headless GPUI runtime works here by running the existing test in
   isolation: expect it to pass.
3. Resolve exact gpui 0.2.2 signatures with `leta`/`cargo doc`:
   `TestAppContext::add_window_view`, `VisualTestContext::from_window`,
   `VisualTestContext::window_handle`, `TestAppContext::simulate_keystrokes`,
   and how to dispatch keystrokes through a `VisualTestContext` rebuilt from a
   window handle. Record the closure arity used by `add_window_view`.
4. Record findings in Surprises & Discoveries and adjust the step design below
   if signatures differ from the sibling-repo example.

Validation for Stage A: existing `gpui_shell_mode_indicator` passes in
isolation; dependency source decided (or escalated); signatures recorded.

### Stage B — Scaffolding and a failing scenario (red)

1. Edit `Cargo.toml` `[dev-dependencies]`: bump `rstest-bdd` and
   `rstest-bdd-macros` to `"0.6.0-beta2"`; add
   `rstest-bdd-harness-gpui = "0.6.0-beta2"` and `serial_test = "3"` (resolve
   the exact published `serial_test` major in Stage A). Per the harness
   dependency matrix, do *not* add a direct `rstest-bdd-harness` entry unless
   Stage A shows the macro path is not recognised as first-party; the example
   `examples/gpui-counter` compiles without it.
2. Create the feature file `tests/features/shell_mode_indicator.feature`
   capturing the three behaviours as one scenario (initial state, Tab toggles
   the edge-mode label, manipulate mode drops the suffix). Keep step wording
   declarative and reusable.
3. Create the test target. Prefer a directory module to respect the 400-line
   limit and keep the thread-local state isolated to its own binary:
   `tests/gpui_shell_mode_indicator_bdd/main.rs` (scenario binding + module
   wiring) and `tests/gpui_shell_mode_indicator_bdd/steps.rs` (steps, fixtures,
   thread-local state). Add the `[[test]]` entry only if Cargo does not
   auto-discover the directory form; confirm in Stage A how the existing
   `tests/widget_capability_audit_bdd/` directory test is registered (it uses a
   `main.rs`, so Cargo auto-discovers it as a test named after the directory).
4. Write the scenario binding and empty/`todo!`-free step skeleton so the suite
   compiles and the scenario *fails* (for example, the `#[then]` assertions are
   present but the `#[given]`/`#[when]` are stubs that do not yet open a
   window).

Validation for Stage B:
`cargo test --test gpui_shell_mode_indicator_bdd --all-features` compiles and
the scenario fails for the intended reason (a clear assertion or missing-state
failure, not a macro/compile error). Capture the transcript.

### Stage C — Implement steps and pass (green)

1. In `steps.rs`, define the thread-local scenario state holding
   `Option<gpui::Entity<Phase0Shell>>` and `Option<gpui::AnyWindowHandle>`, the
   `reset_state_before_assignment`/`reset_state_after_scenario` helpers, the
   `ScenarioStateCleanup` `Drop` guard, and the
   `#[fixture] fn scenario_state_cleanup()` that resets before returning the
   guard — mirroring `crates/rstest-bdd-harness-gpui/tests/stateful_window.rs`.
2. Implement steps:
   - `#[given]` "a Phase 0 shell window is open": request
     `#[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext`, call
     `reset_state_before_assignment()`, open the window via
     `cx.add_window_view(...)` constructing `Phase0Shell::new`, store the
     `Entity` and `window_handle()` in the thread-local, rebuild a
     `VisualTestContext` to run the initial draw to parked.
   - `#[when]` "the user presses {key}": rebuild
     `VisualTestContext::from_window`
     from the stored handle and `cx`, dispatch the keystroke, run to parked.
   - `#[when]` "the shell switches to manipulate mode": rebuild the visual
     context, `update_entity`/`view.update` to call
     `enter_manipulate_mode_for_tests()` and notify, run to parked.
   - `#[then]` "the mode indicator reads {expected}": rebuild the visual
     context,
     read `mode_status_line_for_tests()`, assert equality (strip surrounding
     quotes from the captured placeholder as `tests/i18n_bdd.rs` does).
3. Bind the scenario with `#[scenario(path = "...", name = "...",`
   `harness = rstest_bdd_harness_gpui::GpuiHarness)]` against
   `tests/features/shell_mode_indicator.feature`, apply `#[serial]`, and take
   `#[from(scenario_state_cleanup)] _cleanup: ScenarioStateCleanup`. Omit
   `attributes = ...` to exercise the inferred GPUI policy.
4. Remove `tests/gpui_shell_mode_indicator.rs` only after the BDD scenario
   passes and covers the same three assertions.

Validation for Stage C:
`cargo test --test gpui_shell_mode_indicator_bdd --all-features` passes; the
scenario fails again if a `#[then]` expected value is deliberately corrupted
(sanity check), then restored.

### Stage D — Hardening, gates, documentation, proposals

1. Run the full gates in sequence (never in parallel; respect build caching):
   `make check-fmt`, then `make lint`, then `make test`. Fix any issue that
   arises anywhere in the workspace, even if not obviously caused by this
   change (the task explicitly requires this). Capture transcripts.
2. Write the migration note (see "Artifacts and notes") documenting the adoption
   steps, the friction encountered, and the gaps found in the upstream
   migration guide and user's guide.
3. Finalise the three proposals in this plan (migration order, roadmap gaps,
   harness/doc improvements) with anything learned during implementation.
4. Run `make markdownlint` and `make nixie` for the new/edited Markdown.
5. Complete the Outcomes & Retrospective section.

Validation for Stage D: all of `make check-fmt`, `make lint`, `make test`,
`make markdownlint` pass; the three proposals are concrete; the migration note
exists.

## Concrete steps

All commands run from the worktree root
`/data/leynos/Projects/gauss.worktrees/adopt-rstest-bdd-0-6-0-beta2`.

Stage A:

```bash
# Confirm the headless GPUI test runs in this environment.
cargo test --test gpui_shell_mode_indicator --all-features
# Inspect resolvable rstest-bdd / harness versions (decide source).
cargo update --dry-run -p rstest-bdd 2>&1 | head
```

Expected: the first command reports `test ... ok` for
`mode_indicator_reflects_tool_and_edge_mode`.

Stage B/C iteration:

```bash
cargo test --test gpui_shell_mode_indicator_bdd --all-features
```

Expected (B): a failing assertion or stubbed-state failure, not a compile
error. Expected (C):

```plaintext
test scenario_mode_indicator_reflects_tool_and_edge_mode ... ok
```

Stage D gates (run sequentially):

```bash
make check-fmt
make lint
make test
make markdownlint
```

Expected: each exits zero. `make test` reports all integration and unit tests
passing, including `gpui_shell_mode_indicator_bdd`.

## Validation and acceptance

Acceptance is behavioural:

- Running `make test` passes, and includes a new GPUI BDD scenario test
  `scenario_mode_indicator_reflects_tool_and_edge_mode` (driven by a Gherkin
  feature file through `GpuiHarness`) that asserts the mode indicator reads
  `Mode: Draw (Line)` initially, `Mode: Draw (Bezier (auto))` after `Tab`, and
  `Mode: Manipulate` after switching to manipulate mode. Corrupting any
  expected value makes the scenario fail; restoring it makes it pass.
- The superseded `tests/gpui_shell_mode_indicator.rs` is removed; behaviour
  coverage is preserved by the BDD scenario.
- `Cargo.toml` depends on `rstest-bdd`/`rstest-bdd-macros` `0.6.0-beta2` and
  `rstest-bdd-harness-gpui` `0.6.0-beta2` (dev-dependencies).

Quality criteria ("done"):

- Tests: `make test` green (workspace, all targets, all features).
- Lint/typecheck: `make lint` green (Clippy `-D warnings`, `cargo doc`,
  Whitaker); `make check-fmt` green.
- Docs: `make markdownlint` green for the plan and migration note; `make nixie`
  green if any Mermaid is added (none planned).

Quality method: run the four `make` targets sequentially and compare against
the expected transcripts above.

## Idempotence and recovery

- Editing `Cargo.toml` and adding test files is idempotent; re-running the gates
  is safe. `cargo` reuses the shared cache; do not create an isolated cache.
- If Stage C cannot pass within the iteration tolerance, the recovery path is to
  fall back to migrating `tests/gpui_shell_canvas_layout.rs` (single assertion,
  no keystroke dispatch) using the same thread-local pattern, and record the
  reason in the Decision Log. The mode-indicator file is restored from git if
  it was already removed.
- Removal of the raw test is the only mildly destructive step; it is recoverable
  via `git checkout -- tests/gpui_shell_mode_indicator.rs` until the commit
  lands, and via git history afterwards.

## Artifacts and notes

The migration note is delivered as a new section appended to this plan's
"Outcomes & retrospective" (or, if it grows large, a sibling doc
`docs/rstest-bdd-gpui-harness-adoption-notes.md` linked from here). It must
record, with evidence:

- the exact dependency edits and the source decision (published vs. path/git);
- the precise step shapes that worked, including the `add_window_view` closure
  arity and the `VisualTestContext::from_window` reconstruction;
- each friction point and how it was resolved;
- the documentation gaps below, confirmed or revised against what actually
  happened.

## Proposals (deliverables)

These proposals are part of the required output. They are refined during
implementation but stated here so the plan is self-contained.

### Proposal 1 — Order to migrate the remaining GPUI behavioural tests

Guiding principle: every GPUI test needs the thread-local durable-handle
pattern, so migrate in increasing order of *interaction complexity and number
of distinct gestures*, building a shared step library as we go. Group by the
seam each test exercises so step definitions can be reused across a group
before moving on.

1. **Layout/static-render group (warm-up, simplest reconstruction):**
   `gpui_shell_canvas_layout`, `gpui_shell_chrome_layout`,
   `gpui_shell_resize_borders`, `gpui_shell_window_controls`,
   `gpui_shell_quit_button`. These open a window, draw, and assert on debug
   bounds or a single flag. They establish the shared "a shell window is open"
   `#[given]` and a "debug bounds for {selector}" helper.
2. **Mode/tool-rail group (keystroke + simple state):**
   `gpui_shell_mode_indicator` (this iteration), `gpui_shell_tool_rail`,
   `gpui_tooling_escape_returns_to_draw`,
   `gpui_tooling_draw_escape_commits_open_path`, `gpui_shell_style_controls`,
   `gpui_shell_navigation_buttons`. These add keystroke/click dispatch and read
   tool/edge mode. Reuse the keystroke and "mode indicator reads" steps.
3. **Selection group (clicks + selection state):**
   `gpui_selection_clear_selection`, `gpui_selection_multi_select`,
   `gpui_selection_select_shape_by_bbox`,
   `gpui_selection_select_tool_noop_paths`,
   `gpui_selection_bbox_drag_requires_selection`. These need
   `replace_document_for_tests`/`replace_selection_for_tests` seam steps and
   selection-reading steps.
4. **Drag/manipulate group (multi-step drag gestures):**
   `gpui_selection_multi_shape_drag`, `gpui_tooling_toggle_segment_kind`,
   `gpui_tooling_close_path`, `gpui_tooling_draw_bezier_auto`,
   `gpui_tooling_hit_test_service`, `gpui_tooling_keybinding_integration`.
   These exercise mouse-down/move/up sequences across steps — the strongest
   test of durable-handle reconstruction.
5. **History/undo group (longest, most stateful):**
   `gpui_history_draw_undo`, `gpui_history_close_path_undo`,
   `gpui_history_drag_shape_undo`, `gpui_history_drag_anchor_undo`,
   `gpui_history_drag_handle_undo`, `gpui_history_multi_shape_drag_undo`,
   `gpui_history_reorder_undo`, `gpui_history_anchor_edit_undo`,
   `gpui_history_command_grouping_undo`, `gpui_history_selection_history`,
   `gpui_history_open_history_reset`. Largest files (up to 418 lines), multiple
   gestures plus undo/redo dispatch; migrate last when the step library is
   mature.
6. **File-IO and a11y group (dialogs and services, may need extra fixtures):**
   `gpui_file_io_*` (4 tests), `gpui_shell_a11y_service`,
   `gpui_shell_viewport_input`, `gpui_widget_audit_shell_seam`,
   `gpui_i18n_module` (overlaps the existing `i18n_bdd`). These touch platform
   dialogs and services; some may stay as raw tests if BDD adds no clarity —
   decide per test.

Rationale notes: tests already covered by a non-GPUI BDD suite (i18n,
widget-capability-audit, a11y-service, undo-entry-count) should not be
duplicated as GPUI BDD scenarios unless the GPUI path adds distinct coverage.

### Proposal 2 — Gaps in the rstest-bdd v0.6.0 roadmap to resolve

Grounded in `/data/leynos/Projects/rstest-bdd/docs/roadmap.md`:

- The v0.6.1 borrow/state helpers (items 11.1.1–11.1.4) and ergonomics
  (11.2.1–11.2.4) are all unchecked. The single most impactful gap for *real*
  GPUI adoption is **11.1.3 (scenario-local state helper with `set`/`with`/
  `with_mut`/`take`/`reset`)** and **11.1.4 (per-scenario cleanup
  registration)**: without them every GPUI scenario re-implements the
  thread-local `RefCell` + `Drop` reset boilerplate by hand, which is the
  largest friction this migration hits. These should be pulled forward, ideally
  before the v0.6.0 *final* tag rather than deferred to 0.6.1, because they
  materially change the recommended adoption shape.
- **11.2.1 (`#[harness_context]` marker)** is a small, non-breaking ergonomic
  win that removes the `#[from(rstest_bdd_harness_context)]` string-key
  spelling from every step. Resolving it early reduces copy-paste errors during
  a bulk GPUI migration like Proposal 1.
- **10.1.4** is marked done as "scenario name in logs *or* documented upstream
  limitation". The roadmap does not record *which* outcome shipped. The gap is
  a definitive statement (and a linked regression or a documented limitation)
  so downstream users know whether failing GPUI scenarios self-identify by name.
- The roadmap has no explicit item for a **downstream "bulk migration"
  cookbook** — guidance on sharing step libraries across many GPUI scenarios in
  one consuming crate, and on how `#[serial]` + thread-local state interact
  with nextest's per-binary parallelism. This is exactly what a real adopter
  (this repo) needs and is currently only implicit in the single-scenario
  examples.

### Proposal 3 — Improvements to the harness, its interface, or its documentation

For the v0.6.0 or v1.0.0 roadmap:

- **Harness interface (v0.7.0, aligns with 12.1.1):** the
  `StepContext::borrow_mut(&mut self, ...)` contract is the root cause of the
  whole thread-local workaround. The guard-based concurrent-borrow redesign
  (12.1.1) should be treated as the headline v0.7.0 deliverable; until it
  lands, the harness cannot offer an ergonomic "mutable harness context +
  mutable world" step, and every adopter pays the thread-local tax. Recommend
  an explicit ADR amendment confirming 12.1.1 is a v0.7.0 *commitment*, not an
  ambition.
- **Harness ergonomics (v0.6.1):** ship a first-party
  `GpuiScenarioState<T>`-style helper (the generic form of 11.1.3) *in the
  `rstest-bdd-harness-gpui` crate*, plus a `#[fixture]`-generating macro for
  the cleanup guard, so adopters get the durable-handle/reset pattern for free
  instead of copying ~50 lines from `stateful_window.rs`. This is the single
  documentation-to-API gap most visible during this migration.
- **Documentation:** the user's guide *does* carry a complete, copy-pasteable
  playbook under "Stateful GPUI scenarios with durable handles"
  (`docs/rstest-bdd-users-guide.md` from line 1088), and the migration guide's
  "Migrate a stateful GPUI test" subsection links it. The remaining gap is that
  the playbook does not document how nextest's per-binary parallelism interacts
  with `#[serial]` and the shared thread-local scenario state, which matters as
  soon as one consuming crate has many GPUI scenarios (Proposal 1). Recommend
  adding that interaction note to the playbook.
- **Documentation:** clarify the `add_window_view` closure arity expected by the
  harness against a given gpui version. The upstream example uses a
  single-argument closure (`|_context| ...`) while this repo's gpui 0.2.2 raw
  tests use a two-argument closure (`|_window, view_cx| ...`); the guide should
  state which gpui versions map to which arity, or the discrepancy costs every
  adopter a compile-error round trip.
- **Build invalidation / `#[scenario]` macro (v0.6.x, high priority):** make
  feature-file edits trigger a rebuild of the scenario binary. Cargo's
  fingerprinting tracks Rust sources and declared inputs, but the
  `#[scenario(path = "…")]` macro reads the `.feature` file with ordinary
  filesystem I/O, so cargo cannot see the dependency. As observed in Stage C
  (Surprises & Discoveries and Outcomes), a corrupted expectation can appear to
  *pass* from cache until an unrelated `.rs` file is touched — a silent, severe
  foot-gun for a testing framework. Prefer making the fix invisible to
  consumers rather than a per-crate obligation:
  1. *Macro-emitted `include_str!` (preferred).* Have the macro emit an
     `include_str!("…/foo.feature")` (even to a discarded `const`); cargo tracks
     `include_str!` paths for rebuild purposes, so this closes the loop with no
     consumer action and cannot be forgotten.
  2. *Shipped `build.rs` helper (fallback / complement).* Offer a build-script
     helper that scans the features directory and emits
     `cargo::rerun-if-changed=<dir>` plus a per-file line for each `.feature`.
  The sibling project [`theoremc`](https://github.com/leynos/theoremc) already
  solves the identical problem for its `.theorem` files and is a ready
  reference: its `build.rs` always emits `cargo::rerun-if-changed=theorems`
  (even when the directory is absent), adds nested directories and every
  discovered file "for robustness across Cargo versions and edge cases", and
  pairs that with a generated `OUT_DIR` suite compiled via `include!()` so the
  macro never reads an untracked file behind cargo's back. Notably `theoremc`
  treats invalidation as a *tested contract* (`tests/build_discovery_bdd.rs`
  asserts the exact `rerun-if-changed=` lines); rstest-bdd should adopt the same
  pattern and add a regression test for it.

## Interfaces and dependencies

Dependencies to add to `Cargo.toml` `[dev-dependencies]` (exact versions
confirmed in Stage A):

- `rstest-bdd = "0.6.0-beta2"`, `rstest-bdd-macros = "0.6.0-beta2"` (bump).
- `rstest-bdd-harness-gpui = "0.6.0-beta2"` (new).
- `serial_test = "<major from Stage A>"` (new).

Types/attributes that must exist at the end of the milestone, in the new test
target `tests/gpui_shell_mode_indicator_bdd/`:

```rust
// steps.rs
use rstest::fixture;
use rstest_bdd_macros::{given, then, when};

#[derive(Default)]
struct ScenarioState {
    entity: Option<gpui::Entity<gauss::ui::Phase0Shell>>,
    window: Option<gpui::AnyWindowHandle>,
}

thread_local! {
    static SCENARIO_STATE: std::cell::RefCell<ScenarioState> =
        std::cell::RefCell::new(ScenarioState::default());
}

struct ScenarioStateCleanup;
impl Drop for ScenarioStateCleanup { /* reset_state_after_scenario() */ }

#[fixture]
fn scenario_state_cleanup() -> ScenarioStateCleanup { /* reset, return guard */ }

#[given("a Phase 0 shell window is open")]
fn shell_window_is_open(
    #[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext,
) { /* reset, add_window_view, store handles, initial draw */ }

#[when("the user presses {key}")]
fn user_presses_key(
    #[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext,
    key: String,
) { /* rebuild VisualTestContext, dispatch, park */ }

#[then("the mode indicator reads {expected}")]
fn mode_indicator_reads(
    #[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext,
    expected: String,
) { /* rebuild visual context, read mode_status_line_for_tests, assert */ }
```

```rust
// main.rs
mod steps;

#[rstest_bdd_macros::scenario(
    path = "tests/features/shell_mode_indicator.feature",
    name = "Mode indicator reflects tool and edge mode",
    harness = rstest_bdd_harness_gpui::GpuiHarness,
)]
#[serial_test::serial]
fn scenario_mode_indicator_reflects_tool_and_edge_mode(
    #[from(steps::scenario_state_cleanup)] _cleanup: steps::ScenarioStateCleanup,
) {
}
```

The reserved fixture key `rstest_bdd_harness_context` resolves to
`gpui::TestAppContext` because `GpuiHarness` declares
`type Context = TestAppContext`. The GPUI attribute policy (`#[rstest::rstest]`
plus `#[gpui::test]`) is inferred from the canonical
`rstest_bdd_harness_gpui::GpuiHarness` path, so `attributes = ...` is omitted.

## Outcomes & retrospective

**What was achieved.** The Purpose is met: `gauss` now pins `rstest-bdd`,
`rstest-bdd-macros`, and `rstest-bdd-harness-gpui` at `0.6.0-beta2` (plus
`serial_test`), and the first GPUI behavioural test runs as a Gherkin scenario
on the first-party GPUI harness. The raw `#[gpui::test]` form in
`tests/gpui_shell_mode_indicator.rs` was removed and replaced by
`tests/features/shell_mode_indicator.feature` driving
`tests/gpui_shell_mode_indicator_bdd/{main.rs,steps.rs}`. All three required
gates pass: `make check-fmt`, `make lint`, and `make test` (909 passed, 1
skipped). The migrated scenario
`scenario_mode_indicator_reflects_tool_and_edge_mode` was observed both passing
and — when its expected value was deliberately corrupted — failing with a
clear, scenario-named assertion, confirming the assertion bites.

**Realised step shapes.** Each step requests only
`#[from(rstest_bdd_harness_context)] cx: &mut gpui::TestAppContext`. Durable
handles (`Entity<Phase0Shell>` plus `AnyWindowHandle`) live in a thread-local
`RefCell`, rebuilt into a `VisualTestContext` per step via
`VisualTestContext::from_window(window, cx)`. The scenario is `#[serial]` and
uses the two-sided reset protocol (a reset-before-assignment fixture returning a
`Drop` guard). The gpui 0.2.2 API shapes (two-argument `add_window_view`
closure, `Window::window_handle()`, by-value `from_window`, identity
`Result<T> = T`) differ from the user's-guide playbook and are documented
inline in `steps.rs` and in Surprises & Discoveries.

**Confirmed documentation gaps (feeding Proposals 2 and 3).** (1) The
user's-guide playbook targets a newer gpui than the harness's MSRV-compatible
0.2.2; four snippet shapes do not copy verbatim. (2) The playbook snippets
violate a pedantic lint profile (`shadow_reuse`, and
`unwrap_or_else(|| panic!)` against `no_unwrap_or_else_panic` whilst
`expect_used` is also denied), so a lint-clean variant is needed. (3) Cargo
does not rebuild a `#[scenario]` binary on `.feature`-only edits, so a
corrupted expectation can appear to pass until a `.rs` file is touched — a real
foot-gun worth a documented caveat.

**What would be done differently for the bulk migration (Proposal 1).** The
thread-local durable-handle boilerplate is the dominant cost and is identical
across every GPUI scenario; landing roadmap 11.1.3/11.1.4 (a scenario-local
state helper plus per-scenario cleanup registration) before the bulk migration
would remove the hand-rolled `RefCell`, `Drop` guard, and
`let … else { panic! }` accessor from every file. Until then, the migration
should factor the handle helpers into one shared steps module per consuming
crate rather than copying them per test, and each migrated scenario must be
touched (not just its feature file) when adjusting expectations.
